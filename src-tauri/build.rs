// 功能：生成 Tauri 配置与受限命令 ACL；时间：2026-11-08；作者：lq。
fn main() {
    const COMMANDS: &[&str] = &["bootstrap", "list_bookmarks", "save_bookmark", "delete_bookmark", "save_settings", "list_recent", "clear_history", "hide_recent", "browser_view"];
    tauri_build::try_build(
        tauri_build::Attributes::new()
            .app_manifest(tauri_build::AppManifest::new().commands(COMMANDS)),
    ).expect("无法生成轻页构建配置");
}
