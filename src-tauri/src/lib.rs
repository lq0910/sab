// 功能：初始化数据库、受限命令和无边框本地窗口，由前端标题栏提供窗口控制；时间：2026-11-08；作者：lq。
mod browser;
mod commands;
// 功能：启用本地下载管理及完成文件操作；时间：2026-11-14；作者：lq。
mod downloads;
mod models;
mod security;
mod storage;
// 功能：注册网页焦点下的原生快捷键；时间：2026-11-14；作者：lq。
mod shortcuts;
// 功能：启用持久化会话恢复；时间：2026-11-14；作者：lq。
mod session;
// 功能：首版在数据库初始化前阻止重复进程占用；时间：2026-11-14；作者：lq。
mod instance;
// 功能：系统默认浏览器与外部链接接入；时间：2026-11-14；作者：lq。
mod system_browser;

use commands::*;
use std::sync::{Arc, Mutex};
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

pub fn run() {
    let initial_links = system_browser::ExternalLinks::default();
    initial_links.push(std::env::args().skip(1));
    let application = tauri::Builder::default()
        .manage(initial_links)
        // 功能：重复启动交给官方插件，转交链接并聚焦原窗口，不再丢弃系统URL；时间：2026-11-14；作者：lq。
        .plugin(tauri_plugin_single_instance::init(|app, args, _| {
            system_browser::receive(app, args.into_iter().skip(1));
        }))
        .setup(|app| {
            let directory = app.path().app_data_dir()?;
            std::fs::create_dir_all(&directory)?;
            // 功能：重复进程不打开数据库，避免将仍在传输的下载错误标成中断；时间：2026-11-14；作者：lq。
            let Some(guard) = instance::InstanceGuard::acquire(&directory)? else {
                rfd::MessageDialog::new().set_title("轻页 SAB 已在运行").set_description("请回到已经打开的轻页 SAB 窗口继续使用。").show();
                app.handle().exit(0);
                return Ok(());
            };
            app.manage(guard);
            let store = storage::Store::open(&directory.join("sab.sqlite3")).map_err(std::io::Error::other)?;
            app.manage(AppState(Arc::new(Mutex::new(store))));
            shortcuts::install(app.handle())?;
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
        // 功能：注册收藏、历史记录和网页视图的受限命令；时间：2026-11-08；作者：lq。
        .invoke_handler(tauri::generate_handler![bootstrap, create_folder, rename_folder, delete_folder, list_bookmarks, save_bookmark, import_bookmarks, classification_preview, apply_classification, undo_classification, delete_bookmark, save_settings, list_recent, list_history, record_visit, delete_history, clear_history, hide_recent, browser::browser_view, downloads::list_downloads, downloads::clear_downloads, downloads::open_download, load_session, save_session, system_browser::default_browser_status, system_browser::request_default_browser, system_browser::take_external_links])
        .build(tauri::generate_context!());
    match application {
        Ok(app) => app.run(|_app, event| {
            // 功能：macOS冷/热启动的系统HTTP/HTTPS事件先排队，前端就绪后消费；时间：2026-11-14；作者：lq。
            #[cfg(target_os = "macos")]
            if let tauri::RunEvent::Opened { urls } = event {
                system_browser::receive(_app, urls.into_iter().map(|url| url.to_string()));
            }
            #[cfg(not(target_os = "macos"))]
            let _ = event;
        }),
        // 功能：启动失败显示可读信息，不隐式删除或重建用户数据；时间：2026-11-14；作者：lq。
        Err(error) => {
            rfd::MessageDialog::new().set_title("轻页 SAB 无法启动")
                .set_description(format!("本地数据未被清除。请关闭其他实例后重试，或保留数据目录并联系支持。\n\n{error}"))
                .set_level(rfd::MessageLevel::Error).show();
        }
    }
}
