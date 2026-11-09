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
// 功能：创建用户分类并返回分类 ID 和名称；时间：2026-11-08；作者：lq。
#[tauri::command]
pub async fn create_folder(caller: Webview, state: State<'_, AppState>, name: String) -> Result<Folder, String> {
    with_store(caller, state, move |s| s.create_folder(&name)).await
}
// 功能：更新用户分类名称并保持已有收藏关联不变；时间：2026-11-08；作者：lq。
#[tauri::command]
pub async fn rename_folder(caller: Webview, state: State<'_, AppState>, id: String, name: String) -> Result<(), String> {
    with_store(caller, state, move |s| s.rename_folder(&id, &name)).await
}
// 功能：删除分类并将其中收藏移动到用户选择的目标分类；时间：2026-11-08；作者：lq。
#[tauri::command]
pub async fn delete_folder(caller: Webview, state: State<'_, AppState>, id: String, target_id: String) -> Result<u32, String> {
    with_store(caller, state, move |s| s.delete_folder(&id, &target_id)).await
}
#[tauri::command]
pub async fn list_bookmarks(caller: Webview, state: State<'_, AppState>, query: String, folder_id: String, offset: u32, pinned: bool) -> Result<BookmarkPage, String> {
    with_store(caller, state, move |s| s.list(&query, &folder_id, offset, pinned)).await
}
#[tauri::command]
pub async fn save_bookmark(caller: Webview, state: State<'_, AppState>, draft: BookmarkDraft) -> Result<(), String> {
    with_store(caller, state, move |s| s.save(draft)).await
}
// 功能：在可信本地界面批量导入浏览器书签并返回处理数量；时间：2026-11-08；作者：lq。
#[tauri::command]
pub async fn import_bookmarks(caller: Webview, state: State<'_, AppState>, drafts: Vec<BookmarkDraft>) -> Result<BookmarkImportResult, String> {
    with_store(caller, state, move |s| s.import_bookmarks(drafts)).await
}
// 功能：预览待整理收藏的本地规则分类建议，不修改收藏数据；时间：2026-11-08；作者：lq。
#[tauri::command]
pub async fn classification_preview(caller: Webview, state: State<'_, AppState>) -> Result<ClassificationPreview, String> {
    with_store(caller, state, |s| s.classification_preview()).await
}
// 功能：执行预览过的本地分类规则并生成可撤销批次；时间：2026-11-08；作者：lq。
#[tauri::command]
pub async fn apply_classification(caller: Webview, state: State<'_, AppState>) -> Result<ClassificationBatchResult, String> {
    with_store(caller, state, |s| s.apply_classification()).await
}
// 功能：撤销指定自动分类批次且跳过用户后来手动整理的收藏；时间：2026-11-08；作者：lq。
#[tauri::command]
pub async fn undo_classification(caller: Webview, state: State<'_, AppState>, batch_id: String) -> Result<ClassificationUndoResult, String> {
    with_store(caller, state, move |s| s.undo_classification(&batch_id).map(|restored| ClassificationUndoResult { restored })).await
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
// 功能：提供可信本地界面的历史搜索和分页接口；时间：2026-11-08；作者：lq。
#[tauri::command]
pub async fn list_history(caller: Webview, state: State<'_, AppState>, query: String, offset: u32, limit: u32) -> Result<HistoryPage, String> {
    with_store(caller, state, move |s| s.history(&query, offset, limit)).await
}
// 功能：记录可信网页视图完成加载的网址和标题；时间：2026-11-08；作者：lq。
#[tauri::command]
pub async fn record_visit(caller: Webview, state: State<'_, AppState>, url: String, title: String) -> Result<(), String> {
    with_store(caller, state, move |s| s.record_visit(&url, &title)).await
}
// 功能：删除历史列表中的单条网址记录；时间：2026-11-08；作者：lq。
#[tauri::command]
pub async fn delete_history(caller: Webview, state: State<'_, AppState>, url: String) -> Result<(), String> {
    with_store(caller, state, move |s| s.delete_history(&url)).await
}
#[tauri::command]
pub async fn clear_history(caller: Webview, state: State<'_, AppState>) -> Result<(), String> {
    with_store(caller, state, |s| s.clear_history()).await
}
#[tauri::command]
pub async fn hide_recent(caller: Webview, state: State<'_, AppState>, url: String) -> Result<(), String> {
    with_store(caller, state, move |s| s.hide_recent(&url)).await
}
