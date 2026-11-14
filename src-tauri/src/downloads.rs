// 功能：使用原生网页下载引擎保留登录态，管理保存位置、下载记录与文件操作；时间：2026-11-14；作者：lq。
use crate::{commands::AppState, security::require_shell, storage::Store};
use rusqlite::params;
use serde::Serialize;
use std::{path::{Path, PathBuf}, time::{SystemTime, UNIX_EPOCH}};
use tauri::{webview::DownloadEvent, AppHandle, Emitter, Manager, State, Webview};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Download {
    pub id: String, pub url: String, pub tab_id: String, pub source_url: String,
    pub name: String, pub path: String, pub status: String, pub received_bytes: u64,
    pub started_at: i64, pub finished_at: Option<i64>, pub error: String,
    #[serde(skip)]
    pub temp_path: String,
    #[serde(skip)]
    pub overwrite_allowed: bool,
}

fn error(e: impl std::fmt::Display) -> String { e.to_string() }
fn now() -> i64 { SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as i64 }

impl Store {
    // 功能：分页上限固定为最近 200 项，活动任务始终保留在前；时间：2026-11-14；作者：lq。
    pub fn downloads(&self) -> Result<Vec<Download>, String> {
        let mut statement = self.connection.prepare("SELECT id,url,tab_id,source_url,name,path,status,received_bytes,started_at,finished_at,error,temp_path,overwrite_allowed FROM downloads ORDER BY (status='downloading') DESC,started_at DESC LIMIT 200").map_err(error)?;
        let rows = statement.query_map([], |row| Ok(Download {
            id: row.get(0)?, url: row.get(1)?, tab_id: row.get(2)?, source_url: row.get(3)?,
            name: row.get(4)?, path: row.get(5)?, status: row.get(6)?, received_bytes: row.get(7)?,
            started_at: row.get(8)?, finished_at: row.get(9)?, error: row.get(10)?, temp_path: row.get(11)?, overwrite_allowed: row.get(12)?,
        })).map_err(error)?.collect::<rusqlite::Result<Vec<_>>>().map_err(error)?;
        Ok(rows.into_iter().map(|mut item| {
            // 功能：原生引擎未暴露总大小，仅显示实际落盘字节，不伪造百分比；时间：2026-11-14；作者：lq。
            if item.status == "downloading" { item.received_bytes = std::fs::metadata(&item.temp_path).map(|m| m.len()).unwrap_or(0); }
            item
        }).collect())
    }
    pub fn download_active(&self, tab: &str) -> Result<bool, String> {
        self.connection.query_row("SELECT EXISTS(SELECT 1 FROM downloads WHERE tab_id=?1 AND status='downloading')", [tab], |r| r.get(0)).map_err(error)
    }
    fn insert_download(&self, item: &Download) -> Result<(), String> {
        self.connection.execute("INSERT INTO downloads(id,url,tab_id,source_url,name,path,temp_path,status,started_at,overwrite_allowed) VALUES(?1,?2,?3,?4,?5,?6,?7,'downloading',?8,?9)", params![item.id,item.url,item.tab_id,item.source_url,item.name,item.path,item.temp_path,item.started_at,item.overwrite_allowed]).map_err(error)?;
        // 功能：限定下载记录数量，活动任务不会被自动清除；时间：2026-11-14；作者：lq。
        self.connection.execute("DELETE FROM downloads WHERE status!='downloading' AND id NOT IN (SELECT id FROM downloads ORDER BY started_at DESC LIMIT 190)", []).map_err(error)?;
        Ok(())
    }
    fn finish_download(&self, id: &str, status: &str, size: u64, message: &str) -> Result<(), String> {
        self.connection.execute("UPDATE downloads SET status=?2,received_bytes=?3,finished_at=?4,error=?5 WHERE id=?1 AND status='downloading'", params![id,status,size,now(),message]).map_err(error)?;
        Ok(())
    }
    pub fn clear_downloads(&self) -> Result<(), String> {
        self.connection.execute("DELETE FROM downloads WHERE status!='downloading'", []).map_err(error)?;
        Ok(())
    }
    fn download(&self, id: &str) -> Result<Download, String> {
        self.downloads()?.into_iter().find(|item| item.id == id).ok_or_else(|| "下载记录不存在或已超出最近 200 项".into())
    }
}

// 功能：下载变更只通知可信外壳，文件路径不向网页广播；时间：2026-11-14；作者：lq。
fn notify(app: &AppHandle) { let _ = app.emit_to("main", "sab:downloads", ()); }
fn notice(app: &AppHandle, message: &str) { let _ = app.emit_to("main", "sab:download-notice", message); }

// 功能：通过系统保存面板确认文件名，先写同目录临时文件，成功后才替换目标；时间：2026-11-14；作者：lq。
pub fn handle_download(view: Webview, tab_id: &str, event: DownloadEvent<'_>) -> bool {
    handle_download_with_picker(view, tab_id, event, |dialog| dialog.save_file())
}
// 功能：隔离文件选择入口，允许原生集成测试使用独立临时目录验证下载引擎；时间：2026-11-14；作者：lq。
pub(crate) fn handle_download_with_picker(view: Webview, tab_id: &str, event: DownloadEvent<'_>, pick: impl FnOnce(rfd::FileDialog) -> Option<PathBuf>) -> bool {
    let app = view.app_handle();
    let state = app.state::<AppState>();
    match event {
        DownloadEvent::Requested { url, destination } => {
            // 功能：同一来源同一地址串行下载，避免 macOS 完成事件无路径时关联错误；时间：2026-11-14；作者：lq。
            let existing = match state.0.lock().map_err(error).and_then(|s| s.downloads()) {
                Ok(items) => items,
                Err(e) => { notice(app, &format!("无法创建下载：{e}")); return false; }
            };
            if existing.iter().any(|d| d.status == "downloading" && d.tab_id == tab_id && d.url == url.as_str()) {
                notice(app, "该文件正在下载，请在下载面板查看"); return false;
            }
            if existing.iter().filter(|d| d.status == "downloading").count() >= 10 {
                notice(app, "最多同时下载 10 个文件，请等待已有任务完成"); return false;
            }
            let name = destination.file_name().and_then(|s| s.to_str()).unwrap_or("download").to_string();
            let directory = existing.first().and_then(|d| Path::new(&d.path).parent()).filter(|p| p.is_dir()).map(Path::to_path_buf)
                .or_else(|| app.path().download_dir().ok());
            let mut dialog = rfd::FileDialog::new().set_title("保存下载文件").set_file_name(name).set_parent(&view.window());
            if let Some(directory) = directory { dialog = dialog.set_directory(directory); }
            let Some(target) = pick(dialog) else { return false; };
            if !target.is_absolute() || target.is_dir() { notice(app, "请选择有效的文件保存位置"); return false; }
            let id = uuid::Uuid::new_v4().to_string();
            let temporary = target.with_file_name(format!(".sab-{id}.part"));
            let item = Download {
                id, url: url.to_string(), tab_id: tab_id.to_string(), source_url: view.url().map(|u| u.to_string()).unwrap_or_default(),
                name: target.file_name().unwrap_or_default().to_string_lossy().into(), path: target.to_string_lossy().into(),
                temp_path: temporary.to_string_lossy().into(), status: "downloading".into(), received_bytes: 0,
                started_at: now(), finished_at: None, error: String::new(), overwrite_allowed: target.exists(),
            };
            let result = state.0.lock().map_err(error).and_then(|s| s.insert_download(&item));
            if let Err(e) = result { notice(app, &format!("无法保存下载记录：{e}")); return false; }
            *destination = temporary;
            notify(app);
            true
        },
        DownloadEvent::Finished { url, success, .. } => {
            let items = match state.0.lock().map_err(error).and_then(|s| s.downloads()) {
                Ok(items) => items, Err(e) => { notice(app, &e); return true; }
            };
            let Some(item) = items.into_iter().find(|d| d.status == "downloading" && d.tab_id == tab_id && d.url == url.as_str()) else { return true; };
            let size = std::fs::metadata(&item.temp_path).map(|m| m.len()).unwrap_or(0);
            let result = if success { finalize_file(&item) } else { Err("网络连接中断或网站拒绝下载，请重新尝试".into()) };
            let (status, message) = match result { Ok(()) => ("completed", String::new()), Err(e) => ("failed", e) };
            if !success { let _ = std::fs::remove_file(&item.temp_path); }
            if let Err(e) = state.0.lock().map_err(error).and_then(|s| s.finish_download(&item.id, status, size, &message)) { notice(app, &e); }
            notify(app);
            true
        },
        _ => false,
    }
}

// 功能：成功下载后原子改名，目标后来新建时保留原文件和已下载的临时文件；时间：2026-11-14；作者：lq。
fn finalize_file(item: &Download) -> Result<(), String> {
    let temporary = tempfile::TempPath::try_from_path(&item.temp_path).map_err(error)?;
    let result = if item.overwrite_allowed { temporary.persist(&item.path) } else { temporary.persist_noclobber(&item.path) };
    result.map_err(|e| {
        let reason = e.error.to_string();
        // 功能：提交失败时保留完整临时文件，防止用户下载成果被自动删除；时间：2026-11-14；作者：lq。
        let _ = e.path.keep();
        format!("保存失败：{reason}。已下载文件保留在 {}", item.temp_path)
    })
}

// 功能：所有下载命令校验本地外壳，前端只能按记录 ID 操作文件；时间：2026-11-14；作者：lq。
#[tauri::command]
pub async fn list_downloads(caller: Webview, state: State<'_, AppState>) -> Result<Vec<Download>, String> {
    require_shell(&caller)?;
    let store = state.0.clone();
    tauri::async_runtime::spawn_blocking(move || store.lock().map_err(error)?.downloads()).await.map_err(error)?
}
#[tauri::command]
pub async fn clear_downloads(caller: Webview, state: State<'_, AppState>) -> Result<(), String> {
    require_shell(&caller)?;
    state.0.lock().map_err(error)?.clear_downloads()
}
#[tauri::command]
pub async fn open_download(caller: Webview, state: State<'_, AppState>, id: String, reveal: bool) -> Result<(), String> {
    require_shell(&caller)?;
    let item = state.0.lock().map_err(error)?.download(&id)?;
    if item.status != "completed" { return Err("文件尚未下载完成".into()); }
    let path = PathBuf::from(item.path);
    if !path.is_file() { return Err("文件已被移动或删除，请重新下载".into()); }
    tauri::async_runtime::spawn_blocking(move || {
        if reveal { tauri_plugin_opener::reveal_item_in_dir(path).map_err(error) }
        else { tauri_plugin_opener::open_path(path, None::<&str>).map_err(error) }
    }).await.map_err(error)?
}

#[cfg(test)]
mod tests {
    use super::*;
    // 功能：创建隔离下载记录，测试不接触用户文件与业务数据库；时间：2026-11-14；作者：lq。
    fn item(directory: &Path) -> Download {
        Download { id: uuid::Uuid::new_v4().to_string(), url: "https://example.com/file".into(), tab_id: "tab".into(), source_url: "https://example.com/".into(), name: "file.txt".into(), path: directory.join("file.txt").to_string_lossy().into(), temp_path: directory.join(".sab-test.part").to_string_lossy().into(), status: "downloading".into(), received_bytes: 0, started_at: now(), finished_at: None, error: String::new(), overwrite_allowed: false }
    }
    // 功能：下载完成前保持已有文件内容，确认覆盖后才提交完整文件；时间：2026-11-14；作者：lq。
    #[test]
    fn completed_download_replaces_only_after_success() {
        let directory = tempfile::tempdir().unwrap();
        let mut record = item(directory.path());
        std::fs::write(&record.path, b"original").unwrap();
        std::fs::write(&record.temp_path, b"complete download").unwrap();
        assert!(finalize_file(&record).is_err());
        assert_eq!(std::fs::read(&record.path).unwrap(), b"original");
        assert!(Path::new(&record.temp_path).exists());
        record.overwrite_allowed = true;
        finalize_file(&record).unwrap();
        assert_eq!(std::fs::read(&record.path).unwrap(), b"complete download");
        assert!(!Path::new(&record.temp_path).exists());
    }
    // 功能：重启后中断状态可见，清理记录不影响活动任务和已下载文件；时间：2026-11-14；作者：lq。
    #[test]
    fn restart_marks_interrupted_and_clear_keeps_files_and_active_tasks() {
        let directory = tempfile::tempdir().unwrap();
        let db_path = directory.path().join("sab.sqlite3");
        let database = Store::open(&db_path).unwrap();
        let record = item(directory.path());
        database.insert_download(&record).unwrap();
        assert!(database.download_active("tab").unwrap());
        database.clear_downloads().unwrap();
        assert_eq!(database.downloads().unwrap().len(), 1);
        drop(database);
        let database = Store::open(&db_path).unwrap();
        assert_eq!(database.downloads().unwrap()[0].status, "interrupted");
        assert!(!database.download_active("tab").unwrap());
        std::fs::write(&record.path, b"saved").unwrap();
        database.clear_downloads().unwrap();
        assert!(database.downloads().unwrap().is_empty());
        assert_eq!(std::fs::read(&record.path).unwrap(), b"saved");
    }
    // 功能：临时文件不存在时不得破坏用户原有目标；时间：2026-11-14；作者：lq。
    #[test]
    fn missing_download_never_removes_original_file() {
        let directory = tempfile::tempdir().unwrap();
        let mut record = item(directory.path());
        record.overwrite_allowed = true;
        std::fs::write(&record.path, b"original").unwrap();
        assert!(finalize_file(&record).is_err());
        assert_eq!(std::fs::read(&record.path).unwrap(), b"original");
    }
}
