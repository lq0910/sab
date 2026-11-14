// 功能：使用真实前端、受限 IPC 和独立数据库验收恢复、快捷键动作、离线提示及退出保存；时间：2026-11-14；作者：lq。
// 运行前执行 pnpm build；测试占用回环 1421 端口，临时测试窗口不会替换已安装应用。快捷键动作通过原生菜单事件通道注入，不模拟物理键盘。
#![allow(dead_code)]
#[path = "../src/models.rs"] mod models;
#[path = "../src/security.rs"] mod security;
#[path = "../src/storage.rs"] mod storage;
#[path = "../src/commands.rs"] mod commands;
#[path = "../src/downloads.rs"] mod downloads;
#[path = "../src/session.rs"] mod session;
#[path = "../src/shortcuts.rs"] mod shortcuts;
#[path = "../src/browser.rs"] mod browser;
// 功能：隔离验证系统链接接入与默认状态读取，不修改设备默认浏览器；时间：2026-11-14；作者：lq。
#[path = "../src/system_browser.rs"] mod system_browser;
use std::{io::{Read, Write}, net::TcpListener, sync::{Arc, Mutex, atomic::{AtomicBool, AtomicUsize, Ordering}}, time::Duration};
use tauri::{Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

// 功能：在真实外壳 DOM 上验证交互，标题仅用作隔离测试进程的结果通道；时间：2026-11-14；作者：lq。
const HARNESS: &str = r#"
(async()=>{
 document.title='SAB_SMOKE_STARTED';
 const pause=ms=>new Promise(resolve=>setTimeout(resolve,ms));
 const until=async(check,label)=>{for(let i=0;i<150;i++){if(check())return;await pause(50);}throw Error(label+'; '+document.body.innerText.slice(0,1500));};
 const tabs=()=>document.querySelectorAll('.browser-tab');
 let sequence=0;
 const action=async name=>{document.title='SAB_SMOKE_ACTION:'+(++sequence)+':'+name;await pause(250);};
 const assert=(value,label)=>{if(!value)throw Error(label);};
 await until(()=>!document.querySelector('.launch-screen')&&tabs().length===3,'restore 3 tabs');
 await pause(300);
 document.title='SAB_SMOKE_LAZY';await pause(250);
 await action('focus-address');
 await until(()=>document.activeElement===document.querySelector('[aria-label="地址或搜索"]'),'address focus');
 await action('new-tab');await until(()=>tabs().length===4,'new tab');
 await action('close-tab');await until(()=>tabs().length===3,'close tab');
 await action('restore-tab');await until(()=>tabs().length===4,'restore closed tab');
 await action('next-tab');await action('next-tab');
 window.dispatchEvent(new Event('offline'));
 await until(()=>document.querySelector('.empty-state h1')?.textContent==='设备当前离线','offline feedback');
 const waiting=[...document.querySelectorAll('.empty-state button')].find(button=>button.textContent==='继续等待');
 assert(waiting,'continue waiting');waiting.click();
 await until(()=>!document.querySelector('.empty-state'),'continue existing request');
 await pause(2300);
 await action('downloads');
 await until(()=>document.querySelector('.downloads-view'),'downloads panel');
 await action('close-tab');assert(tabs().length===4,'closing panel preserves tabs');
 // 功能：真实查询系统默认项，随后注入系统链接检验现有窗口的新标签接收；时间：2026-11-14；作者：lq。
 await action('settings');
 await until(()=>document.querySelector('#default .muted')?.textContent.includes('HTTP：'),'default browser status');
 document.title='SAB_SMOKE_EXTERNAL';
 await until(()=>tabs().length===5,'external link creates tab');
 await until(()=>document.querySelector('[aria-label="地址或搜索"]').value.includes('/external'),'external address');
 // 功能：在300毫秒自动保存之前退出，要求关闭流程主动写入最后的新标签；时间：2026-11-14；作者：lq。
 await action('new-tab');assert(tabs().length===6,'final unsaved tab');
 await action('quit');
})().catch(error=>{document.title='SAB_SMOKE_FAIL:'+error.message;});
"#;

fn main() {
    let directory = tempfile::tempdir().expect("isolated workspace");
    let store = Arc::new(Mutex::new(storage::Store::open(&directory.path().join("test.sqlite3")).unwrap()));
    let first = uuid::Uuid::new_v4().to_string();
    let second = uuid::Uuid::new_v4().to_string();
    let home = uuid::Uuid::new_v4().to_string();
    store.lock().unwrap().save_session(models::SessionSnapshot {
        version: 1, active_id: first.clone(), closed: vec![], tabs: vec![
            models::SessionTab { id: first, url: "http://127.0.0.1:1421/first".into(), title: "第一页".into(), zoom: 1.2 },
            models::SessionTab { id: second.clone(), url: "http://127.0.0.1:1421/slow".into(), title: "第二页".into(), zoom: 1.0 },
            models::SessionTab { id: home, url: "sab://home".into(), title: "主页".into(), zoom: 1.0 },
        ],
    }).unwrap();
    let listener = TcpListener::bind("127.0.0.1:1421").expect("test requires unused loopback port 1421");
    let assets = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../dist");
    let requests = Arc::new(AtomicUsize::new(0));
    let server_requests = requests.clone();
    // 功能：只提供构建产物和固定网页，慢响应验证继续等待没有重发请求；时间：2026-11-14；作者：lq。
    std::thread::spawn(move || {
        for incoming in listener.incoming() {
            let Ok(mut stream) = incoming else { break; };
            let assets = assets.clone(); let requests = server_requests.clone();
            std::thread::spawn(move || {
                let mut buffer = [0u8; 8192];
                let count = stream.read(&mut buffer).unwrap_or(0);
                let request = String::from_utf8_lossy(&buffer[..count]);
                let route = request.split_whitespace().nth(1).unwrap_or("/").split('?').next().unwrap_or("/");
                let (body, kind) = if ["/first", "/slow", "/external"].contains(&route) {
                    requests.fetch_add(1, Ordering::SeqCst);
                    if route == "/slow" { std::thread::sleep(Duration::from_secs(2)); }
                    (b"<!doctype html><title>Fixture page</title><input value='page input'>".to_vec(), "text/html")
                } else if route == "/" || route == "/index.html" {
                    (std::fs::read(assets.join("index.html")).unwrap(), "text/html")
                } else if route.starts_with("/assets/") && !route.contains("..") {
                    (std::fs::read(assets.join(&route[1..])).unwrap_or_default(), if route.ends_with(".js") { "text/javascript" } else if route.ends_with(".css") { "text/css" } else { "image/svg+xml" })
                } else { (vec![], "text/plain") };
                let _ = write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: {kind}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len());
                let _ = stream.write_all(&body);
            });
        }
    });
    let passed = Arc::new(AtomicBool::new(false));
    let setup_store = store.clone(); let setup_passed = passed.clone(); let setup_requests = requests.clone();
    let exit_code = tauri::Builder::default().setup(move |app| {
        app.manage(commands::AppState(setup_store.clone()));
        app.manage(system_browser::ExternalLinks::default());
        shortcuts::install(app.handle())?;
        let results = setup_passed.clone(); let requests = setup_requests.clone();
        WebviewWindowBuilder::new(app, "main", WebviewUrl::External("http://127.0.0.1:1421/index.html".parse().unwrap()))
            .title("SAB isolated workspace smoke").inner_size(1000.0, 700.0)
            .on_page_load(|view, event| { println!("shell page: {:?} {}", event.event(), event.url()); if matches!(event.event(), tauri::webview::PageLoadEvent::Finished) { view.eval(HARNESS).unwrap(); } })
            .on_document_title_changed(move |window, title| {
                println!("shell title: {title}");
                if let Some(request) = title.strip_prefix("SAB_SMOKE_ACTION:") {
                    let action = request.split_once(':').unwrap().1;
                    if action == "quit" { results.store(true, Ordering::SeqCst); }
                    window.app_handle().emit_to("main", "sab:shortcut", action).unwrap();
                }
                else if title == "SAB_SMOKE_EXTERNAL" {
                    system_browser::receive(window.app_handle(), ["file:///tmp/rejected".into(), "http://127.0.0.1:1421/external".into()]);
                }
                else if title == "SAB_SMOKE_LAZY" {
                    let views = window.app_handle().webviews().keys().filter(|name| name.starts_with("page-")).count();
                    if views != 1 || requests.load(Ordering::SeqCst) != 1 { eprintln!("lazy restore failed: views={views}"); window.app_handle().exit(1); }
                } else if title.starts_with("SAB_SMOKE_FAIL:") { eprintln!("{title}"); window.app_handle().exit(1); }
            }).build()?;
        let handle = app.handle().clone();
        std::thread::spawn(move || { std::thread::sleep(Duration::from_secs(20)); eprintln!("workspace smoke: TIMEOUT"); handle.exit(2); });
        Ok(())
    })
    .invoke_handler(tauri::generate_handler![commands::bootstrap, commands::list_recent, commands::list_history, commands::record_visit, commands::load_session, commands::save_session, browser::browser_view, downloads::list_downloads, system_browser::default_browser_status, system_browser::take_external_links])
    .build(tauri::generate_context!()).expect("native workspace runtime").run_return(|_, _| {});
    // 功能：只有正常退出已落盘且未重发慢请求才判定成功；时间：2026-11-14；作者：lq。
    assert_eq!(exit_code, 0, "native runtime exit");
    let snapshot = storage::Store::open(&directory.path().join("test.sqlite3")).unwrap().load_session().unwrap().unwrap();
    assert!(passed.load(Ordering::SeqCst), "frontend smoke must finish");
    assert_eq!(snapshot.tabs.len(), 6);
    assert_eq!(snapshot.active_id, snapshot.tabs.last().unwrap().id);
    assert_eq!(snapshot.tabs.last().unwrap().url, "sab://home");
    assert_eq!(requests.load(Ordering::SeqCst), 3);
    println!("native workspace smoke: PASS (lazy restore, focus action, tab actions, offline/continue, downloads panel, default status, system link, save before quit)");
}
