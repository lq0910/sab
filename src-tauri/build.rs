// 功能：生成 Tauri 配置与受限命令 ACL；时间：2026-11-08；作者：lq。
fn main() {
    // 功能：为受限 IPC 注册收藏导入、分类管理和自动整理命令；时间：2026-11-08；作者：lq。
    const COMMANDS: &[&str] = &["bootstrap", "create_folder", "rename_folder", "delete_folder", "list_bookmarks", "save_bookmark", "import_bookmarks", "classification_preview", "apply_classification", "undo_classification", "delete_bookmark", "save_settings", "list_recent", "list_history", "record_visit", "delete_history", "clear_history", "hide_recent", "browser_view"];
    tauri_build::try_build(
        tauri_build::Attributes::new()
            .app_manifest(tauri_build::AppManifest::new().commands(COMMANDS)),
    ).expect("无法生成轻页构建配置");
}
