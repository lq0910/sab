// 功能：用应用原生菜单捕获网页焦点下的浏览器快捷键，保留系统编辑操作；时间：2026-11-14；作者：lq。
use tauri::{menu::{Menu, MenuItem, PredefinedMenuItem, Submenu}, AppHandle, Emitter};

pub const SHORTCUTS: &[(&str, &str, &str)] = &[
    ("new-tab", "新建标签", "CmdOrCtrl+T"),
    ("close-tab", "关闭标签", "CmdOrCtrl+W"),
    ("restore-tab", "恢复关闭的标签", "CmdOrCtrl+Shift+T"),
    ("focus-address", "地址或搜索", "CmdOrCtrl+L"),
    ("reload", "重新加载", "CmdOrCtrl+R"),
    ("next-tab", "下一个标签", "Ctrl+Tab"),
    ("previous-tab", "上一个标签", "Ctrl+Shift+Tab"),
    ("tab-picker", "搜索标签", "CmdOrCtrl+Shift+A"),
    ("save-bookmark", "收藏当前网页", "CmdOrCtrl+D"),
    ("bookmarks", "收藏中心", "CmdOrCtrl+Shift+O"),
    ("history", "浏览历史", "CmdOrCtrl+Y"),
    ("downloads", "下载管理", "CmdOrCtrl+Shift+J"),
    ("settings", "设置", "CmdOrCtrl+Comma"),
    ("zoom-in", "放大网页", "CmdOrCtrl+Equal"),
    ("zoom-out", "缩小网页", "CmdOrCtrl+Minus"),
    ("zoom-reset", "实际大小", "CmdOrCtrl+Digit0"),
    ("quit", "退出轻页 SAB", "CmdOrCtrl+Q"),
];

// 功能：限定菜单事件白名单，其他原生菜单事件不转发成浏览器命令；时间：2026-11-14；作者：lq。
pub fn action_from_menu(id: &str) -> Option<&'static str> {
    SHORTCUTS.iter().find(|(action, _, _)| id == format!("sab-{action}")).map(|(action, _, _)| *action)
}

pub fn install(app: &AppHandle) -> tauri::Result<()> {
    let menu = Menu::new(app)?;
    #[cfg(target_os = "macos")]
    menu.append(&Submenu::with_items(app, "轻页 SAB", true, &[
        &PredefinedMenuItem::about(app, Some("关于轻页 SAB"), None)?,
        &PredefinedMenuItem::separator(app)?,
        &PredefinedMenuItem::services(app, Some("服务"))?,
        &PredefinedMenuItem::hide(app, Some("隐藏轻页 SAB"))?,
        &PredefinedMenuItem::hide_others(app, Some("隐藏其他"))?,
    ])?)?;
    let browser = Submenu::new(app, "浏览", true)?;
    for (action, label, accelerator) in SHORTCUTS {
        browser.append(&MenuItem::with_id(app, format!("sab-{action}"), label, true, Some(accelerator))?)?;
    }
    menu.append(&browser)?;
    // 功能：复制、粘贴、撤销等继续由当前获得焦点的网页/输入框处理；时间：2026-11-14；作者：lq。
    menu.append(&Submenu::with_items(app, "编辑", true, &[
        &PredefinedMenuItem::undo(app, Some("撤销"))?, &PredefinedMenuItem::redo(app, Some("重做"))?,
        &PredefinedMenuItem::separator(app)?, &PredefinedMenuItem::cut(app, Some("剪切"))?,
        &PredefinedMenuItem::copy(app, Some("复制"))?, &PredefinedMenuItem::paste(app, Some("粘贴"))?,
        &PredefinedMenuItem::select_all(app, Some("全选"))?,
    ])?)?;
    app.set_menu(menu)?;
    app.on_menu_event(|app, event| {
        if let Some(action) = action_from_menu(event.id().as_ref()) { let _ = app.emit_to("main", "sab:shortcut", action); }
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    // 功能：菜单命令不能混入标签右键菜单或未注册动作；时间：2026-11-14；作者：lq。
    #[test]
    fn menu_actions_are_exact() {
        assert_eq!(action_from_menu("sab-focus-address"), Some("focus-address"));
        for value in ["close", "sab-close", "sab-quit-extra", "reload"] { assert_eq!(action_from_menu(value), None); }
        let actions: std::collections::HashSet<_> = SHORTCUTS.iter().map(|(id, _, _)| *id).collect();
        assert_eq!(actions.len(), SHORTCUTS.len());
    }
}
