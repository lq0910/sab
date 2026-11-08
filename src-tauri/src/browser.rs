// 功能：管理与 UI 隔离的原生网页视图，禁止 iframe 和远程 IPC；时间：2026-11-08；作者：lq。
use crate::security::{require_shell, website_url};
use serde::{Deserialize, Serialize};
use tauri::{webview::{NewWindowResponse, PageLoadEvent, WebviewBuilder}, AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, Rect, Webview, WebviewUrl};

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BrowserAction { Open, Activate, Resize, Hide, Close, Reload, Back, Forward }

#[derive(Clone, Deserialize)]
pub struct ViewBounds { x: f64, y: f64, width: f64, height: f64 }

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct PageEvent { tab_id: String, url: String, title: Option<String>, loading: Option<bool> }

fn bounds(value: ViewBounds) -> Result<Rect, String> {
    if [value.x, value.y, value.width, value.height].iter().any(|n| !n.is_finite())
        || value.x < 0.0 || value.y < 76.0 || value.width < 1.0 || value.height < 1.0
        || value.width > 32768.0 || value.height > 32768.0 {
        return Err("网页视图尺寸不合法".into());
    }
    Ok(Rect { position: LogicalPosition::new(value.x, value.y).into(), size: LogicalSize::new(value.width, value.height).into() })
}

// 功能：所有视图操作走异步命令，避免 Windows 同步创建 WebView 死锁；时间：2026-11-08；作者：lq。
#[tauri::command]
pub async fn browser_view(app: AppHandle, caller: Webview, action: BrowserAction, tab_id: String, url: Option<String>, bounds: Option<ViewBounds>) -> Result<(), String> {
    require_shell(&caller)?;
    uuid::Uuid::parse_str(&tab_id).map_err(|_| "标签标识不合法")?;
    let label = format!("page-{tab_id}");
    if matches!(action, BrowserAction::Open | BrowserAction::Activate | BrowserAction::Hide) {
        for (name, view) in app.webviews() {
            if name.starts_with("page-") { view.hide().map_err(|e| e.to_string())?; }
        }
    }
    if matches!(action, BrowserAction::Hide) { return Ok(()); }
    if matches!(action, BrowserAction::Open) {
        let url = website_url(url.as_deref().ok_or("缺少网址")?)?;
        let area = self::bounds(bounds.ok_or("缺少网页尺寸")?)?;
        if let Some(view) = app.get_webview(&label) {
            view.set_bounds(area).map_err(|e| e.to_string())?;
            view.navigate(url).map_err(|e| e.to_string())?;
            view.show().map_err(|e| e.to_string())?;
            return Ok(());
        }
        let id_load = tab_id.clone();
        let id_title = tab_id.clone();
        let id_new = tab_id.clone();
        let new_app = app.clone();
        let builder = WebviewBuilder::new(&label, WebviewUrl::External(url))
            .on_navigation(|url| website_url(url.as_str()).is_ok())
            .on_page_load(move |view, payload| {
                // 页面完成加载不等于 HTTP 成功；历史写入待导航错误核验后启用。
                let _ = view.app_handle().emit_to("main", "sab:page", PageEvent { tab_id: id_load.clone(), url: payload.url().to_string(), title: None, loading: Some(matches!(payload.event(), PageLoadEvent::Started)) });
            })
            .on_document_title_changed(move |view, title| {
                if let Ok(url) = view.url() {
                    let _ = view.app_handle().emit_to("main", "sab:page", PageEvent { tab_id: id_title.clone(), url: url.to_string(), title: Some(title), loading: None });
                }
            })
            .on_new_window(move |url, _| {
                if website_url(url.as_str()).is_ok() {
                    let _ = new_app.emit_to("main", "sab:new-tab", PageEvent { tab_id: id_new.clone(), url: url.to_string(), title: None, loading: None });
                }
                NewWindowResponse::Deny
            })
            // 下载专项尚未提供用户确认，骨架阶段拒绝自动落盘。
            .on_download(|_, _| false);
        let window = caller.window();
        window.add_child(builder, area.position, area.size).map_err(|e| e.to_string())?;
        return Ok(());
    }
    let Some(view) = app.get_webview(&label) else {
        return if matches!(action, BrowserAction::Close) { Ok(()) } else { Err("网页视图尚未创建".into()) };
    };
    let result = match action {
        BrowserAction::Activate => { view.set_bounds(self::bounds(bounds.ok_or("缺少网页尺寸")?)?).map_err(|e| e.to_string())?; view.show() },
        BrowserAction::Resize => view.set_bounds(self::bounds(bounds.ok_or("缺少网页尺寸")?)?),
        BrowserAction::Close => view.close(),
        BrowserAction::Reload => view.reload(),
        // 固定脚本没有插入网页输入；仅桥接原生视图当前历史导航。
        BrowserAction::Back => view.eval("history.back()"),
        BrowserAction::Forward => view.eval("history.forward()"),
        BrowserAction::Open | BrowserAction::Hide => unreachable!(),
    };
    result.map_err(|e| e.to_string())
}
