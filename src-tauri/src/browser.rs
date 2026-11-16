// 功能：管理与 UI 隔离的原生网页视图，禁止 iframe 和远程 IPC；时间：2026-11-08；作者：lq。
use crate::security::{require_shell, website_url};
use serde::{Deserialize, Serialize};
// 功能：按应用跟踪唯一可见网页，避免每次导航遍历并隐藏所有后台视图；时间：2026-11-16；作者：lq。
use std::sync::Mutex;
use tauri::{webview::{NewWindowResponse, PageLoadEvent, WebviewBuilder}, AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, Rect, Webview, WebviewUrl};

// 功能：仅记录原生视图可见性，不改变后台网页、登录态和下载任务；时间：2026-11-16；作者：lq。
#[derive(Default)]
struct PageVisibility(Mutex<Option<String>>);

// 功能：只隐藏上一个可见网页，同标签导航保持显示以减少闪白；时间：2026-11-16；作者：lq。
fn hide_previous(app: &AppHandle, visible: &mut Option<String>, next: Option<&str>) -> Result<(), String> {
    if visible.as_deref() == next { return Ok(()); }
    if let Some(label) = visible.as_deref() {
        if let Some(view) = app.get_webview(label) { view.hide().map_err(|e| e.to_string())?; }
    }
    *visible = None;
    Ok(())
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
// 功能：提供固定的停止与缩放命令，不接受网页脚本输入；时间：2026-11-14；作者：lq。
pub enum BrowserAction { Open, Activate, Resize, Hide, Close, Reload, Back, Forward, Stop, Zoom, FocusShell, FocusPage }

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
pub async fn browser_view(app: AppHandle, caller: Webview, action: BrowserAction, tab_id: String, url: Option<String>, bounds: Option<ViewBounds>, zoom: Option<f64>) -> Result<(), String> {
    require_shell(&caller)?;
    uuid::Uuid::parse_str(&tab_id).map_err(|_| "标签标识不合法")?;
    // 功能：禁止关闭或休眠仍在下载的来源标签，避免中断原生任务；时间：2026-11-14；作者：lq。
    if matches!(action, BrowserAction::Close) {
        let state = app.state::<crate::commands::AppState>();
        if state.0.lock().map_err(|e| e.to_string())?.download_active(&tab_id)? { return Err("该标签正在下载文件，请等待下载完成后关闭或休眠".into()); }
    }
    // 功能：地址栏接管焦点时只聚焦可信外壳，不销毁或导航网页；时间：2026-11-14；作者：lq。
    if matches!(action, BrowserAction::FocusShell) { return caller.set_focus().map_err(|e| e.to_string()); }
    let label = format!("page-{tab_id}");
    // 功能：状态随应用生命周期隔离，原生命令并发时按锁串行更新显示结果；时间：2026-11-16；作者：lq。
    app.manage(PageVisibility::default());
    let visibility = app.state::<PageVisibility>();
    let mut visible = visibility.0.lock().map_err(|e| e.to_string())?;
    if matches!(action, BrowserAction::Hide) {
        return hide_previous(&app, &mut visible, None);
    }
    if matches!(action, BrowserAction::Open) {
        let url = website_url(url.as_deref().ok_or("缺少网址")?)?;
        let area = self::bounds(bounds.ok_or("缺少网页尺寸")?)?;
        hide_previous(&app, &mut visible, Some(&label))?;
        if let Some(view) = app.get_webview(&label) {
            view.set_bounds(area).map_err(|e| e.to_string())?;
            view.navigate(url).map_err(|e| e.to_string())?;
            if visible.as_deref() != Some(&label) { view.show().map_err(|e| e.to_string())?; }
            *visible = Some(label);
            return Ok(());
        }
        let id_load = tab_id.clone();
        let id_title = tab_id.clone();
        let id_new = tab_id.clone();
        let new_app = app.clone();
        // 功能：下载事件绑定来源标签以保护下载中的网页视图；时间：2026-11-14；作者：lq。
        let id_download = tab_id.clone();
        let builder = WebviewBuilder::new(&label, WebviewUrl::External(url))
            // 功能：为浏览的网页提供窄轨道圆角滚动条，悬停时加深提示；时间：2026-11-16；作者：lq。
            .initialization_script(include_str!("pageScrollbars.js"))
            // 功能：允许网站生成的 Blob 下载，仍拒绝本地文件和脚本协议；时间：2026-11-14；作者：lq。
            .on_navigation(|url| website_url(url.as_str()).is_ok() || url.scheme() == "blob")
            .on_page_load(move |view, payload| {
                // 功能：向外壳报告页面加载状态和标题，供标签更新及浏览记录保存；时间：2026-11-08；作者：lq。
                if website_url(payload.url().as_str()).is_err() { return; }
                let finished = matches!(payload.event(), PageLoadEvent::Finished);
                let _ = view.app_handle().emit_to("main", "sab:page", PageEvent {
                    tab_id: id_load.clone(), url: payload.url().to_string(), title: None,
                    loading: Some(!finished),
                });
            })
            .on_document_title_changed(move |view, title| {
                if let Ok(url) = view.url() {
                    if website_url(url.as_str()).is_err() { return; }
                    // 功能：标题更新仅更新标题，不伪造加载完成；时间：2026-11-14；作者：lq。
                    let _ = view.app_handle().emit_to("main", "sab:page", PageEvent { tab_id: id_title.clone(), url: url.to_string(), title: Some(title), loading: None });
                }
            })
            .on_new_window(move |url, _| {
                if website_url(url.as_str()).is_ok() {
                    let _ = new_app.emit_to("main", "sab:new-tab", PageEvent { tab_id: id_new.clone(), url: url.to_string(), title: None, loading: None });
                }
                NewWindowResponse::Deny
            })
            // 功能：交由原生引擎下载，沿用网站登录态、响应头与 Blob 下载；时间：2026-11-14；作者：lq。
            .on_download(move |view, event| crate::downloads::handle_download(view, &id_download, event));
        let window = caller.window();
        window.add_child(builder, area.position, area.size).map_err(|e| e.to_string())?;
        *visible = Some(label);
        return Ok(());
    }
    let Some(view) = app.get_webview(&label) else {
        return if matches!(action, BrowserAction::Close) { Ok(()) } else { Err("网页视图尚未创建".into()) };
    };
    let result = match action {
        // 功能：切换只涉及前后两个视图，重复激活不再隐藏/显示当前网页；时间：2026-11-16；作者：lq。
        BrowserAction::Activate => {
            let area = self::bounds(bounds.ok_or("缺少网页尺寸")?)?;
            hide_previous(&app, &mut visible, Some(&label))?;
            view.set_bounds(area).map_err(|e| e.to_string())?;
            if visible.as_deref() != Some(&label) { view.show().map_err(|e| e.to_string())?; }
            *visible = Some(label);
            Ok(())
        },
        BrowserAction::Resize => view.set_bounds(self::bounds(bounds.ok_or("缺少网页尺寸")?)?),
        // 功能：关闭成功后清理可见标识，关闭后台标签保留前台网页；时间：2026-11-16；作者：lq。
        BrowserAction::Close => {
            view.close().map_err(|e| e.to_string())?;
            if visible.as_deref() == Some(&label) { *visible = None; }
            Ok(())
        },
        BrowserAction::Reload => view.reload(),
        BrowserAction::FocusPage => view.set_focus(),
        // 功能：停止网络加载，并将缩放限制在 50% 至 200%；时间：2026-11-14；作者：lq。
        BrowserAction::Stop => view.eval("window.stop()"),
        BrowserAction::Zoom => {
            let scale = zoom.ok_or("缺少缩放比例")?;
            if !scale.is_finite() || !(0.5..=2.0).contains(&scale) { return Err("缩放比例不合法".into()); }
            view.set_zoom(scale)
        },
        // 固定脚本没有插入网页输入；仅桥接原生视图当前历史导航。
        BrowserAction::Back => view.eval("history.back()"),
        BrowserAction::Forward => view.eval("history.forward()"),
        BrowserAction::Open | BrowserAction::Hide | BrowserAction::FocusShell => unreachable!(),
    };
    result.map_err(|e| e.to_string())
}
