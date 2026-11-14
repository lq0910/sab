// 功能：注册浏览器候选、读取真实默认状态、响应用户设置请求并接收系统链接；时间：2026-11-14；作者：lq。
use crate::security::{require_shell, website_url};
use serde::Serialize;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, Webview};

#[derive(Default)]
pub struct ExternalLinks(Mutex<Vec<String>>);

impl ExternalLinks {
    // 功能：系统输入只接受无凭据HTTP(S)，最多排队32项，不允许命令或本地文件；时间：2026-11-14；作者：lq。
    pub fn push(&self, values: impl IntoIterator<Item = String>) {
        if let Ok(mut queue) = self.0.lock() {
            for value in values {
                if queue.len() >= 32 { break; }
                if let Ok(url) = website_url(&value) { queue.push(url.to_string()); }
            }
        }
    }
    fn take(&self) -> Result<Vec<String>, String> {
        Ok(std::mem::take(&mut *self.0.lock().map_err(|_| "系统链接队列暂不可用")?))
    }
}

// 功能：冷启动排队直到前端就绪，热启动通知现有窗口并带回前台；时间：2026-11-14；作者：lq。
pub fn receive(app: &AppHandle, values: impl IntoIterator<Item = String>) {
    app.state::<ExternalLinks>().push(values);
    let _ = app.emit_to("main", "sab:external-links", ());
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize(); let _ = window.show(); let _ = window.set_focus();
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DefaultBrowserStatus { supported: bool, http: bool, https: bool, message: String }

#[tauri::command]
pub fn take_external_links(caller: Webview, app: AppHandle) -> Result<Vec<String>, String> {
    require_shell(&caller)?;
    app.state::<ExternalLinks>().take()
}

#[tauri::command]
pub async fn default_browser_status(caller: Webview) -> Result<DefaultBrowserStatus, String> {
    require_shell(&caller)?;
    tauri::async_runtime::spawn_blocking(platform::status).await.map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn request_default_browser(caller: Webview) -> Result<DefaultBrowserStatus, String> {
    require_shell(&caller)?;
    // 功能：只有用户点击按钮才请求更改系统默认项，查询状态不会产生注册写入；时间：2026-11-14；作者：lq。
    tauri::async_runtime::spawn_blocking(platform::request).await.map_err(|e| e.to_string())?
}

#[cfg(target_os = "macos")]
mod platform {
    use super::*;
    use core_foundation::{base::{TCFType, CFRelease}, string::{CFString, CFStringRef}};
    #[link(name = "CoreServices", kind = "framework")]
    extern "C" {
        fn LSCopyDefaultHandlerForURLScheme(scheme: CFStringRef) -> CFStringRef;
        fn LSSetDefaultHandlerForURLScheme(scheme: CFStringRef, bundle: CFStringRef) -> i32;
    }
    fn handles(scheme: &str) -> bool {
        let scheme = CFString::new(scheme);
        // 功能：遵循Copy规则读取并释放LaunchServices返回的协议处理器；时间：2026-11-14；作者：lq。
        unsafe {
            let value = LSCopyDefaultHandlerForURLScheme(scheme.as_concrete_TypeRef());
            if value.is_null() { return false; }
            let name = CFString::wrap_under_get_rule(value).to_string();
            CFRelease(value.cast());
            name.eq_ignore_ascii_case("com.lq.sab")
        }
    }
    pub fn status() -> Result<DefaultBrowserStatus, String> {
        Ok(DefaultBrowserStatus { supported: true, http: handles("http"), https: handles("https"), message: "请先将轻页 SAB 安装到 Applications；系统可能要求确认更改默认浏览器。".into() })
    }
    pub fn request() -> Result<DefaultBrowserStatus, String> {
        // 功能：开发裸二进制不能冒用已安装应用身份请求系统关联；时间：2026-11-14；作者：lq。
        let executable = std::env::current_exe().map_err(|e| e.to_string())?;
        if !executable.ancestors().any(|path| path.extension().is_some_and(|ext| ext == "app")) { return Err("请安装并打开打包后的轻页 SAB.app，再设置默认浏览器".into()); }
        let bundle = CFString::new("com.lq.sab");
        for scheme in ["http", "https"] {
            let scheme = CFString::new(scheme);
            let result = unsafe { LSSetDefaultHandlerForURLScheme(scheme.as_concrete_TypeRef(), bundle.as_concrete_TypeRef()) };
            if result != 0 { return Err(format!("系统未完成默认浏览器设置（{result}）。请在系统设置的默认网页浏览器中选择轻页 SAB。")); }
        }
        let mut state = status()?;
        if !(state.http && state.https) { state.message = "设置请求已提交，请完成系统确认，再点击刷新状态。".into(); }
        Ok(state)
    }
}

#[cfg(target_os = "windows")]
mod platform {
    use super::*;
    use winreg::{enums::HKEY_CURRENT_USER, RegKey};
    const CAPABILITIES: &str = r"Software\Clients\StartMenuInternet\SAB\Capabilities";
    fn write(root: &RegKey, path: &str, name: &str, value: &str) -> Result<(), String> {
        root.create_subkey(path).and_then(|(key, _)| key.set_value(name, &value)).map_err(|e| e.to_string())
    }
    fn register() -> Result<(), String> {
        let executable = std::env::current_exe().map_err(|e| e.to_string())?;
        let executable = executable.to_str().ok_or("应用路径不是有效Unicode")?;
        let root = RegKey::predef(HKEY_CURRENT_USER);
        write(&root, r"Software\Classes\SAB.Url", "", "轻页 SAB 网页链接")?;
        write(&root, r"Software\Classes\SAB.Url", "URL Protocol", "")?;
        write(&root, r"Software\Classes\SAB.Url\DefaultIcon", "", &format!("\"{executable}\",0"))?;
        write(&root, r"Software\Classes\SAB.Url\shell\open\command", "", &format!("\"{executable}\" \"%1\""))?;
        write(&root, CAPABILITIES, "ApplicationName", "轻页 SAB")?;
        write(&root, CAPABILITIES, "ApplicationDescription", "轻量网页浏览与文件下载")?;
        write(&root, CAPABILITIES, "ApplicationIcon", &format!("\"{executable}\",0"))?;
        for scheme in ["http", "https"] { write(&root, &format!(r"{CAPABILITIES}\URLAssociations"), scheme, "SAB.Url")?; }
        write(&root, r"Software\RegisteredApplications", "轻页 SAB", CAPABILITIES)
    }
    fn handles(scheme: &str) -> Result<bool, String> {
        let path = format!(r"Software\Microsoft\Windows\Shell\Associations\UrlAssociations\{scheme}\UserChoice");
        match RegKey::predef(HKEY_CURRENT_USER).open_subkey(path).and_then(|key| key.get_value::<String, _>("ProgId")) {
            Ok(value) => Ok(value.eq_ignore_ascii_case("SAB.Url")),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(error.to_string()),
        }
    }
    pub fn status() -> Result<DefaultBrowserStatus, String> {
        Ok(DefaultBrowserStatus { supported: true, http: handles("http")?, https: handles("https")?, message: "在Windows默认应用中选择轻页 SAB，将HTTP和HTTPS设为默认；完成后返回刷新状态。".into() })
    }
    pub fn request() -> Result<DefaultBrowserStatus, String> {
        register()?;
        // 功能：使用受支持的设置入口，绝不写入Windows保护的默认选择哈希；时间：2026-11-14；作者：lq。
        tauri_plugin_opener::open_url("ms-settings:defaultapps?registeredAppUser=%E8%BD%BB%E9%A1%B5%20SAB", None::<&str>).map_err(|e| e.to_string())?;
        status()
    }
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod platform {
    use super::*;
    pub fn status() -> Result<DefaultBrowserStatus, String> { Ok(DefaultBrowserStatus { supported: false, http: false, https: false, message: "当前平台尚未接入默认浏览器设置".into() }) }
    pub fn request() -> Result<DefaultBrowserStatus, String> { Err("当前平台尚未接入默认浏览器设置".into()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    // 功能：外部链接不信任文件、命令参数、凭据与应用壳地址，排队有上限且只消费一次；时间：2026-11-14；作者：lq。
    #[test]
    fn external_urls_are_validated_bounded_and_drained_once() {
        let links = ExternalLinks::default();
        links.push(["file:///tmp/a", "--devtools", "https://a:b@example.com", "http://tauri.localhost/", "https://example.com/?q=你好#test"].map(str::to_string));
        assert_eq!(links.take().unwrap().len(), 1);
        assert!(links.take().unwrap().is_empty());
        links.push((0..100).map(|n| format!("https://example.com/{n}")));
        assert_eq!(links.take().unwrap().len(), 32);
    }
}
