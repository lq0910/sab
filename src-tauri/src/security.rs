// 功能：校验命令调用来源和 HTTP(S) 地址，阻止网页越权；时间：2026-11-08；作者：lq。
use tauri::Webview;
use url::Url;

pub fn is_shell_url(url: &Url) -> bool {
    let production = (url.scheme() == "tauri" && url.host_str() == Some("localhost"))
        || (url.scheme() == "http" && url.host_str() == Some("tauri.localhost"));
    let development = cfg!(debug_assertions) && url.scheme() == "http"
        && url.host_str() == Some("127.0.0.1") && url.port() == Some(1421);
    (production || development) && url.username().is_empty() && url.password().is_none()
}

pub fn require_shell(caller: &Webview) -> Result<(), String> {
    if caller.label() != "main" || caller.window().label() != "main"
        || !is_shell_url(&caller.url().map_err(|e| e.to_string())?) {
        return Err("此调用不属于可信本地界面".into());
    }
    Ok(())
}

pub fn website_url(input: &str) -> Result<Url, String> {
    if input.len() > 16384 { return Err("网址过长".into()); }
    let url = Url::parse(input).map_err(|_| "网址格式不正确")?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none()
        || !url.username().is_empty() || url.password().is_some()
        || url.host_str() == Some("tauri.localhost") {
        return Err("仅支持不含登录凭据的 HTTP 或 HTTPS 网址".into());
    }
    Ok(url)
}

#[cfg(test)]
mod tests {
    use super::*;
    // 功能：验证远程页面不能伪装成本地 UI；时间：2026-11-08；作者：lq。
    #[test]
    fn origins_are_exact() {
        assert!(is_shell_url(&Url::parse("tauri://localhost/index.html").unwrap()));
        for value in ["https://localhost/", "https://tauri.localhost.evil/", "http://127.0.0.1:1420/", "https://example.com/"] {
            assert!(!is_shell_url(&Url::parse(value).unwrap()));
        }
    }
    // 功能：验证危险协议、凭据和内部资源隔离；时间：2026-11-08；作者：lq。
    #[test]
    fn only_websites_are_accepted() {
        for value in ["javascript:alert(1)", "file:///tmp/x", "tauri://localhost/", "https://a:b@example.com", "http://tauri.localhost/"] {
            assert!(website_url(value).is_err());
        }
        assert_eq!(website_url("http://example.com/?utm_source=x#part").unwrap().as_str(), "http://example.com/?utm_source=x#part");
    }
}
