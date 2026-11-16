// 功能：迁移网站账号至系统凭据库，并限定来源手动填写普通登录表单；时间：2026-11-16；作者：lq。
use crate::{models::{CredentialAccount, CredentialDraft, CredentialImportResult}, security::website_url, storage::Store};
use rusqlite::{params, OptionalExtension};
use tauri::Manager;
use zeroize::Zeroizing;

const SERVICE: &str = "com.lq.sab.website-credentials";

// 功能：抽象系统秘密存储，逻辑测试使用隔离内存实现；时间：2026-11-16；作者：lq。
trait SecretStore {
    fn write(&self, id: &str, password: &str) -> Result<(), String>;
    fn read(&self, id: &str) -> Result<String, String>;
    fn delete(&self, id: &str) -> Result<(), String>;
}
struct SystemSecrets;

// 功能：macOS/Windows 显式使用真实系统密码库，其他平台拒绝回退到临时模拟存储；时间：2026-11-16；作者：lq。
impl SecretStore for SystemSecrets {
    fn write(&self, id: &str, password: &str) -> Result<(), String> {
        if !cfg!(any(target_os = "macos", target_os = "windows")) { return Err("此系统尚未支持安全密码存储".into()); }
        keyring::Entry::new(SERVICE, id).map_err(vault_error)?.set_password(password).map_err(vault_error)
    }
    fn read(&self, id: &str) -> Result<String, String> {
        if !cfg!(any(target_os = "macos", target_os = "windows")) { return Err("此系统尚未支持安全密码存储".into()); }
        keyring::Entry::new(SERVICE, id).map_err(vault_error)?.get_password().map_err(vault_error)
    }
    fn delete(&self, id: &str) -> Result<(), String> {
        if !cfg!(any(target_os = "macos", target_os = "windows")) { return Err("此系统尚未支持安全密码存储".into()); }
        match keyring::Entry::new(SERVICE, id).map_err(vault_error)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(error) => Err(vault_error(error)),
        }
    }
}

// 功能：系统存储错误仅反馈操作原因，不将平台原始错误中的数据带到界面或日志；时间：2026-11-16；作者：lq。
fn vault_error(error: keyring::Error) -> String {
    match error {
        keyring::Error::NoEntry => "系统密码库中找不到此条目，请删除后重新导入".into(),
        keyring::Error::TooLong(..) => "密码超过系统凭据库的长度限制".into(),
        _ => "无法访问系统密码库，请解锁并允许 SAB 访问后重试".into(),
    }
}

// 功能：规范化来源但原样保留用户名和密码，避免更改真实登录凭据；时间：2026-11-16；作者：lq。
fn validate_draft(draft: &CredentialDraft) -> Result<String, String> {
    if draft.username.len() > 1024 || draft.password.is_empty() || draft.password.len() > 4096 { return Err("账号或密码长度不符合导入限制".into()); }
    Ok(website_url(&draft.origin)?.origin().ascii_serialization())
}

// 功能：先校验整个批次，逐条提交索引和系统密码；重复项不覆盖，失败项明确统计；时间：2026-11-16；作者：lq。
fn import_with(store: &Store, drafts: Vec<CredentialDraft>, vault: &impl SecretStore) -> Result<CredentialImportResult, String> {
    if drafts.is_empty() || drafts.len() > 5000 { return Err("请选择 1 至 5000 个账号".into()); }
    let origins = drafts.iter().map(validate_draft).collect::<Result<Vec<_>, _>>()?;
    let mut result = CredentialImportResult { imported: 0, skipped: 0, failed: 0 };
    for (draft, origin) in drafts.iter().zip(origins) {
        let transaction = store.connection.unchecked_transaction().map_err(|_| "无法开始本地索引写入")?;
        let exists: bool = transaction.query_row("SELECT EXISTS(SELECT 1 FROM credentials WHERE origin=?1 AND username=?2)", params![origin, draft.username], |row| row.get(0)).map_err(|_| "无法检索本地账号索引")?;
        if exists { result.skipped += 1; continue; }
        let id = uuid::Uuid::new_v4().to_string();
        transaction.execute("INSERT INTO credentials(id,origin,username,created_at) VALUES(?1,?2,?3,unixepoch())", params![id, origin, draft.username]).map_err(|_| "无法写入本地账号索引")?;
        if vault.write(&id, &draft.password).is_err() { result.failed += 1; continue; }
        if transaction.commit().is_err() {
            let _ = vault.delete(&id);
            return Err(format!("本地索引写入失败，已导入 {} 个账号；可重新选择原 CSV 补入剩余项目", result.imported));
        }
        result.imported += 1;
    }
    Ok(result)
}

// 功能：正式导入始终使用操作系统密码库；时间：2026-11-16；作者：lq。
pub fn import(store: &Store, drafts: Vec<CredentialDraft>) -> Result<CredentialImportResult, String> { import_with(store, drafts, &SystemSecrets) }

// 功能：按网站或设置页检索不含密码的账号索引；时间：2026-11-16；作者：lq。
pub fn list(store: &Store, raw_origin: &str) -> Result<Vec<CredentialAccount>, String> {
    let origin = if raw_origin.is_empty() { String::new() } else { website_url(raw_origin)?.origin().ascii_serialization() };
    let mut statement = store.connection.prepare("SELECT id,origin,username FROM credentials WHERE ?1='' OR origin=?1 ORDER BY origin,username COLLATE NOCASE LIMIT 5000").map_err(|_| "无法检索账号索引")?;
    let accounts = statement.query_map([origin], |row| Ok(CredentialAccount { id: row.get(0)?, origin: row.get(1)?, username: row.get(2)? })).map_err(|_| "无法检索账号索引")?.collect::<rusqlite::Result<Vec<_>>>().map_err(|_| "无法读取账号索引")?;
    Ok(accounts)
}

// 功能：先移除系统密码再删除索引，访问被拒绝时保留条目以便重试；时间：2026-11-16；作者：lq。
fn delete_with(store: &Store, id: &str, vault: &impl SecretStore) -> Result<(), String> {
    uuid::Uuid::parse_str(id).map_err(|_| "账号标识不合法")?;
    let exists: bool = store.connection.query_row("SELECT EXISTS(SELECT 1 FROM credentials WHERE id=?1)", [id], |row| row.get(0)).map_err(|_| "无法读取本地索引")?;
    if !exists { return Ok(()); }
    vault.delete(id)?;
    store.connection.execute("DELETE FROM credentials WHERE id=?1", [id]).map_err(|_| "无法删除本地账号索引")?;
    Ok(())
}
// 功能：设置页删除入口始终同步清理系统秘密和本地索引；时间：2026-11-16；作者：lq。
pub fn delete(store: &Store, id: &str) -> Result<(), String> { delete_with(store, id, &SystemSecrets) }

// 功能：读取时复核精确来源，含协议、域名与端口；仅 HTTPS 账号允许填写；时间：2026-11-16；作者：lq。
fn account_for_origin(store: &Store, id: &str, current: &url::Url) -> Result<CredentialAccount, String> {
    if current.scheme() != "https" { return Err("请在 HTTPS 登录页填写已保存密码".into()); }
    let account = store.connection.query_row("SELECT id,origin,username FROM credentials WHERE id=?1", [id], |row| Ok(CredentialAccount { id: row.get(0)?, origin: row.get(1)?, username: row.get(2)? })).optional().map_err(|_| "无法读取账号索引")?.ok_or("找不到该网站账号")?;
    if current.origin().ascii_serialization() != account.origin { return Err("网页已切换到其他网站，请重新选择账号".into()); }
    Ok(account)
}

// 功能：脚本再次校验实时来源，结果通过原生回调返回，网页没有 IPC 权限；时间：2026-11-16；作者：lq。
pub fn fill(app: &tauri::AppHandle, tab_id: &str, id: &str) -> Result<(), String> {
    let uuid = uuid::Uuid::parse_str(tab_id).map_err(|_| "标签标识不合法")?;
    let view = app.get_webview(&format!("page-{uuid}")).ok_or("网页尚未打开")?;
    let current = view.url().map_err(|_| "无法读取当前网页")?;
    let account = {
        let state = app.state::<crate::commands::AppState>();
        let store = state.0.lock().map_err(|_| "本地存储暂不可用")?;
        account_for_origin(&store, id, &current)?
    };
    let password = Zeroizing::new(SystemSecrets.read(id)?);
    let payload = Zeroizing::new(serde_json::to_string(&serde_json::json!({ "username": account.username, "password": password.as_str() })).map_err(|_| "无法准备填表数据")?);
    let script = include_str!("credentialFill.js").replace("__SAB_ORIGIN__", &serde_json::to_string(&account.origin).map_err(|_| "无法准备网站校验")?).replace("__SAB_ACCOUNT__", &payload);
    let (send, receive) = std::sync::mpsc::channel();
    view.eval_with_callback(script, move |result| { let _ = send.send(result); }).map_err(|_| "无法访问当前网页表单")?;
    let result = receive.recv_timeout(std::time::Duration::from_secs(5)).map_err(|_| "网页未响应，请等待页面加载完成后重试")?;
    match serde_json::from_str::<String>(&result).unwrap_or_default().as_str() {
        "filled" => Ok(()),
        "origin-changed" => Err("网页已切换到其他网站，请重新选择账号".into()),
        _ => Err("没有找到唯一可见的登录密码框。暂不支持 iframe、注册页或分步登录表单，请手动填写".into()),
    }
}

// 功能：覆盖秘密不落库、重复保留、失败回滚和跨来源拒绝；时间：2026-11-16；作者：lq。
#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::RefCell, collections::HashMap};
    #[derive(Default)]
    struct MemorySecrets { values: RefCell<HashMap<String, String>>, fail: bool }
    impl SecretStore for MemorySecrets {
        fn write(&self, id: &str, password: &str) -> Result<(), String> { if self.fail { return Err("denied".into()); } self.values.borrow_mut().insert(id.into(), password.into()); Ok(()) }
        fn read(&self, id: &str) -> Result<String, String> { self.values.borrow().get(id).cloned().ok_or("missing".into()) }
        fn delete(&self, id: &str) -> Result<(), String> { self.values.borrow_mut().remove(id); Ok(()) }
    }
    fn draft(password: &str) -> CredentialDraft { CredentialDraft { origin: "https://example.com/login".into(), username: " person ".into(), password: password.into() } }
    #[test]
    fn import_duplicate_delete_and_failed_vault_are_consistent() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("test.sqlite3");
        let store = Store::open(&path).unwrap();
        let vault = MemorySecrets::default();
        let result = import_with(&store, vec![draft("unique-dummy-password"), draft("must-not-overwrite")], &vault).unwrap();
        assert_eq!((result.imported, result.skipped, result.failed), (1, 1, 0));
        let accounts = list(&store, "").unwrap();
        assert_eq!(accounts[0].username, " person ");
        assert_eq!(vault.read(&accounts[0].id).unwrap(), "unique-dummy-password");
        assert!(!serde_json::to_string(&accounts).unwrap().contains("unique-dummy-password"));
        for url in ["https://example.com.evil/", "http://example.com/", "https://example.com:444/", "https://sub.example.com/"] { assert!(account_for_origin(&store, &accounts[0].id, &url.parse().unwrap()).is_err()); }
        assert!(account_for_origin(&store, &accounts[0].id, &"https://example.com/other".parse().unwrap()).is_ok());
        delete_with(&store, &accounts[0].id, &vault).unwrap();
        assert!(list(&store, "").unwrap().is_empty()); assert!(vault.values.borrow().is_empty());
        let failing = MemorySecrets { fail: true, ..Default::default() };
        let result = import_with(&store, vec![draft("dummy")], &failing).unwrap();
        assert_eq!((result.imported, result.failed), (0, 1)); assert!(list(&store, "").unwrap().is_empty());
        let invalid = CredentialDraft { origin: "file:///tmp/x".into(), username: "person".into(), password: "dummy".into() };
        assert!(import_with(&store, vec![draft("dummy"), invalid], &vault).is_err()); assert!(vault.values.borrow().is_empty());
        // 功能：检查数据库及 WAL 的实际字节，确认测试口令从未落盘；时间：2026-11-16；作者：lq。
        for file in std::fs::read_dir(directory.path()).unwrap() {
            let bytes = std::fs::read(file.unwrap().path()).unwrap();
            assert!(!bytes.windows(b"unique-dummy-password".len()).any(|window| window == b"unique-dummy-password"));
        }
    }

    // 功能：确认应用在不支持的 OS 上绝不会退回内存或数据库明文存储；时间：2026-11-16；作者：lq。
    #[test]
    fn unsupported_os_never_accepts_secret_fallback() {
        if cfg!(any(target_os = "macos", target_os = "windows")) { return; }
        assert!(SystemSecrets.write("fixture", "dummy").is_err());
        assert!(SystemSecrets.read("fixture").is_err());
    }

    // 功能：显式运行时仅创建并清理随机测试钥匙串条目，不读取用户密码；时间：2026-11-16；作者：lq。
    #[test]
    #[ignore = "requires access to the operating system credential store"]
    fn native_keychain_roundtrip() {
        let id = uuid::Uuid::new_v4().to_string();
        SystemSecrets.write(&id, "sab-isolated-smoke-dummy").unwrap();
        let actual = SystemSecrets.read(&id);
        SystemSecrets.delete(&id).unwrap();
        assert_eq!(actual.unwrap(), "sab-isolated-smoke-dummy");
    }
}
