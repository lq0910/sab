// 功能：用隔离数据库和回环 HTTP 服务验收真实 macOS/Windows WebView 下载；时间：2026-11-14；作者：lq。
// 运行：cargo run --manifest-path src-tauri/Cargo.toml --example download_smoke --offline。
// 文件选择器由测试替身选择临时目录；传输、Cookie、Blob、完成失败事件使用真实原生引擎。
#![allow(dead_code)]
#[path = "../src/models.rs"] mod models;
#[path = "../src/security.rs"] mod security;
#[path = "../src/storage.rs"] mod storage;
#[path = "../src/commands.rs"] mod commands;
#[path = "../src/downloads.rs"] mod downloads;
#[path = "../src/session.rs"] mod session;
use std::{io::{Read, Write}, net::TcpListener, sync::{Arc, Mutex}, time::{Duration, Instant}};
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

fn main() {
    let directory = tempfile::tempdir().expect("isolated directory");
    let store = Arc::new(Mutex::new(storage::Store::open(&directory.path().join("test.sqlite3")).unwrap()));
    let listener = TcpListener::bind("127.0.0.1:0").expect("loopback fixture");
    let origin = format!("http://{}", listener.local_addr().unwrap());
    // 功能：提供带 Cookie 的附件、断流附件与客户端 Blob，数据固定便于字节校验；时间：2026-11-14；作者：lq。
    std::thread::spawn(move || {
        for incoming in listener.incoming() {
            let Ok(mut stream) = incoming else { break; };
            std::thread::spawn(move || {
                let mut buffer = [0u8; 8192];
                let count = stream.read(&mut buffer).unwrap_or(0);
                let request = String::from_utf8_lossy(&buffer[..count]);
                let route = request.split_whitespace().nth(1).unwrap_or("/");
                if route == "/" {
                    let body = r#"<!doctype html><title>SAB download smoke</title><script>
                    function download(url, name) { const a=document.createElement('a'); a.href=url; if(name)a.download=name; document.body.append(a); a.click(); a.remove(); }
                    setTimeout(()=>download('/plain'),500);
                    setTimeout(()=>download('/cookie'),1500);
                    setTimeout(()=>download(URL.createObjectURL(new Blob(['blob-content'],{type:'text/plain'})), 'blob.txt'),2500);
                    setTimeout(()=>download('/broken'),3500);
                    </script><body>Isolated native download test</body>"#;
                    let _ = write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nSet-Cookie: sab_test=logged_in; Path=/\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", body.len(), body);
                } else if route == "/plain" || (route == "/cookie" && request.contains("sab_test=logged_in")) {
                    let content = if route == "/plain" { "plain-content" } else { "authenticated-content" };
                    let name = if route == "/plain" { "plain.txt" } else { "cookie.txt" };
                    let _ = write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Disposition: attachment; filename=\"{name}\"\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{content}", content.len());
                } else if route == "/broken" {
                    let _ = stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Disposition: attachment; filename=\"broken.txt\"\r\nContent-Length: 100000\r\nConnection: close\r\n\r\npartial");
                } else { let _ = stream.write_all(b"HTTP/1.1 403 Forbidden\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"); }
            });
        }
    });
    let setup_store = store.clone();
    let output = directory.path().to_path_buf();
    tauri::Builder::default().setup(move |app| {
        app.manage(commands::AppState(setup_store.clone()));
        let target_dir = output.clone();
        WebviewWindowBuilder::new(app, "main", WebviewUrl::External(origin.parse().unwrap()))
            .title("SAB isolated download smoke")
            .inner_size(600.0, 400.0)
            .visible(false)
            .on_download(move |view, event| {
                let suggested = match &event { tauri::webview::DownloadEvent::Requested { destination, .. } => destination.file_name().unwrap_or_default().to_os_string(), _ => "unused".into() };
                let target = target_dir.join(suggested);
                downloads::handle_download_with_picker(view, "smoke-tab", event, |_| Some(target))
            }).build()?;
        let app_handle = app.handle().clone();
        // 功能：等待真实回调，验证内容、认证继承、失败记录和原始文件保全；时间：2026-11-14；作者：lq。
        std::thread::spawn(move || {
            let started = Instant::now();
            loop {
                std::thread::sleep(Duration::from_millis(200));
                let records = setup_store.lock().unwrap().downloads().unwrap();
                if records.len() == 4 && records.iter().all(|d| d.status != "downloading") {
                    let success = records.iter().filter(|d| d.status == "completed").count() == 3
                        && records.iter().filter(|d| d.status == "failed").count() == 1
                        && std::fs::read(output.join("plain.txt")).ok().as_deref() == Some(b"plain-content".as_slice())
                        && std::fs::read(output.join("cookie.txt")).ok().as_deref() == Some(b"authenticated-content".as_slice())
                        && std::fs::read(output.join("blob.txt")).ok().as_deref() == Some(b"blob-content".as_slice())
                        && !output.join("broken.txt").exists();
                    for item in &records { println!("{}: {} ({} bytes)", item.name, item.status, item.received_bytes); }
                    println!("native download smoke: {}", if success { "PASS" } else { "FAIL" });
                    app_handle.exit(if success { 0 } else { 1 });
                    break;
                }
                if started.elapsed() > Duration::from_secs(25) {
                    for item in records { eprintln!("{}: {} {}", item.name, item.status, item.error); }
                    eprintln!("native download smoke: TIMEOUT");
                    app_handle.exit(2);
                    break;
                }
            }
        });
        Ok(())
    }).run(tauri::generate_context!()).expect("native test runtime");
    drop(directory);
}
