// 功能：在服务入口校验来源并将 SQLite 访问放到工作线程；时间：2026-11-08；作者：lq。
use crate::{models::*, security::require_shell, storage::Store};
use std::sync::{Arc, Mutex};
use tauri::{State, Webview};

pub struct AppState(pub Arc<Mutex<Store>>);

async fn with_store<T, F>(caller: Webview, state: State<'_, AppState>, operation: F) -> Result<T, String>
where T: Send + 'static, F: FnOnce(&Store) -> Result<T, String> + Send + 'static {
    require_shell(&caller)?;
    let store = state.0.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let guard = store.lock().map_err(|_| "本地存储暂不可用")?;
        operation(&guard)
    }).await.map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn bootstrap(caller: Webview, state: State<'_, AppState>) -> Result<Bootstrap, String> {
    with_store(caller, state, |s| s.bootstrap()).await
}
#[tauri::command]
pub async fn list_bookmarks(caller: Webview, state: State<'_, AppState>, query: String, folder_id: String, offset: u32, pinned: bool) -> Result<BookmarkPage, String> {
    with_store(caller, state, move |s| s.list(&query, &folder_id, offset, pinned)).await
}
#[tauri::command]
pub async fn save_bookmark(caller: Webview, state: State<'_, AppState>, draft: BookmarkDraft) -> Result<(), String> {
    with_store(caller, state, move |s| s.save(draft)).await
}
#[tauri::command]
pub async fn delete_bookmark(caller: Webview, state: State<'_, AppState>, id: String) -> Result<(), String> {
    with_store(caller, state, move |s| s.delete(&id)).await
}
#[tauri::command]
pub async fn save_settings(caller: Webview, state: State<'_, AppState>, settings: Settings) -> Result<(), String> {
    with_store(caller, state, move |s| s.save_settings(&settings)).await
}
#[tauri::command]
pub async fn list_recent(caller: Webview, state: State<'_, AppState>) -> Result<Vec<RecentVisit>, String> {
    with_store(caller, state, |s| s.recent()).await
}
#[tauri::command]
pub async fn clear_history(caller: Webview, state: State<'_, AppState>) -> Result<(), String> {
    with_store(caller, state, |s| s.clear_history()).await
}
#[tauri::command]
pub async fn hide_recent(caller: Webview, state: State<'_, AppState>, url: String) -> Result<(), String> {
    with_store(caller, state, move |s| s.hide_recent(&url)).await
}
