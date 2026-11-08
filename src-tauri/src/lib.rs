// 功能：初始化数据库、受限命令和无边框本地窗口，由前端标题栏提供窗口控制；时间：2026-11-08；作者：lq。
mod browser;
mod commands;
mod models;
mod security;
mod storage;

use commands::*;
use std::sync::{Arc, Mutex};
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let directory = app.path().app_data_dir()?;
            std::fs::create_dir_all(&directory)?;
            let store = storage::Store::open(&directory.join("sab.sqlite3")).map_err(std::io::Error::other)?;
            app.manage(AppState(Arc::new(Mutex::new(store))));
            WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
                .title("轻页 SAB")
                .decorations(false)
                .transparent(true)
                .inner_size(1280.0, 820.0)
                .min_inner_size(720.0, 520.0)
                .on_navigation(security::is_shell_url)
                .build()?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![bootstrap, list_bookmarks, save_bookmark, delete_bookmark, save_settings, list_recent, clear_history, hide_recent, browser::browser_view])
        .run(tauri::generate_context!())
        .expect("轻页 SAB 启动失败");
}
