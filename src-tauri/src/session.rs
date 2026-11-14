// 功能：在现有 SQLite 会话表中原子保存标签快照，拒绝无效或超限数据；时间：2026-11-14；作者：lq。
use crate::{models::{SessionSnapshot, Settings}, security::website_url, storage::Store};
use rusqlite::{params, OptionalExtension};
use std::{collections::HashSet, time::{SystemTime, UNIX_EPOCH}};

fn error(e: impl std::fmt::Display) -> String { e.to_string() }

// 功能：恢复时重新验证网址、标签标识和缩放，不接受可执行协议；时间：2026-11-14；作者：lq。
fn validate(snapshot: &SessionSnapshot) -> Result<(), String> {
    if snapshot.version != 1 || snapshot.tabs.is_empty() || snapshot.tabs.len() > 200 || snapshot.closed.len() > 20 { return Err("会话格式或标签数量不合法".into()); }
    let mut ids = HashSet::new();
    for tab in snapshot.tabs.iter().chain(snapshot.closed.iter()) {
        uuid::Uuid::parse_str(&tab.id).map_err(|_| "会话标签标识不合法")?;
        if !ids.insert(&tab.id) || tab.title.chars().count() > 512 || !tab.zoom.is_finite() || !(0.5..=2.0).contains(&tab.zoom) { return Err("会话标签内容不合法".into()); }
        if !["sab://home", "sab://settings", "sab://history", "sab://bookmarks"].contains(&tab.url.as_str()) { website_url(&tab.url)?; }
    }
    if !snapshot.tabs.iter().any(|tab| tab.id == snapshot.active_id) { return Err("会话活动标签不存在".into()); }
    Ok(())
}

impl Store {
    // 功能：关闭会话恢复后不再写入会话，避免偏好关闭后仍保留标签网址；时间：2026-11-14；作者：lq。
    fn session_enabled(&self) -> Result<bool, String> {
        let value: Option<String> = self.connection.query_row("SELECT value FROM settings WHERE id=1", [], |r| r.get(0)).optional().map_err(error)?;
        let settings: Settings = value.map(|s| serde_json::from_str(&s).map_err(error)).transpose()?.unwrap_or_default();
        Ok(settings.restore_session)
    }
    pub fn save_session(&self, snapshot: SessionSnapshot) -> Result<(), String> {
        if !self.session_enabled()? { return Ok(()); }
        validate(&snapshot)?;
        let value = serde_json::to_string(&snapshot).map_err(error)?;
        if value.len() > 1_048_576 { return Err("会话数据超过 1 MB".into()); }
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as i64;
        self.connection.execute("INSERT INTO sessions(id,value,updated_at) VALUES('workspace',?1,?2) ON CONFLICT(id) DO UPDATE SET value=excluded.value,updated_at=excluded.updated_at", params![value, now]).map_err(error)?;
        Ok(())
    }
    pub fn load_session(&self) -> Result<Option<SessionSnapshot>, String> {
        if !self.session_enabled()? { return Ok(None); }
        let value: Option<String> = self.connection.query_row("SELECT value FROM sessions WHERE id='workspace'", [], |r| r.get(0)).optional().map_err(error)?;
        value.map(|value| {
            if value.len() > 1_048_576 { return Err("上次会话数据过大，请从主页继续".into()); }
            let snapshot: SessionSnapshot = serde_json::from_str(&value).map_err(|_| "上次会话数据损坏，请从主页继续")?;
            validate(&snapshot)?;
            Ok(snapshot)
        }).transpose()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::SessionTab;
    // 功能：隔离快照包含网页与内置页，测试不接触用户数据库；时间：2026-11-14；作者：lq。
    fn snapshot() -> SessionSnapshot {
        let id = uuid::Uuid::new_v4().to_string();
        SessionSnapshot { version: 1, active_id: id.clone(), tabs: vec![SessionTab { id, url: "https://example.com/?q=1#part".into(), title: "恢复页面".into(), zoom: 1.2 }], closed: vec![] }
    }
    #[test]
    fn session_survives_reopen_and_invalid_write_preserves_previous() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("test.sqlite3");
        let database = Store::open(&path).unwrap();
        let data = snapshot();
        database.save_session(data.clone()).unwrap();
        let mut invalid = data.clone();
        invalid.tabs[0].url = "javascript:alert(1)".into();
        assert!(database.save_session(invalid).is_err());
        drop(database);
        let restored = Store::open(&path).unwrap().load_session().unwrap().unwrap();
        assert_eq!(restored.active_id, data.active_id);
        assert_eq!(restored.tabs[0].url, data.tabs[0].url);
        assert_eq!(restored.tabs[0].zoom, 1.2);
    }
    #[test]
    fn disabling_restore_clears_snapshot_and_blocks_late_save() {
        let directory = tempfile::tempdir().unwrap();
        let database = Store::open(&directory.path().join("test.sqlite3")).unwrap();
        database.save_session(snapshot()).unwrap();
        let settings = Settings { restore_session: false, ..Settings::default() };
        database.save_settings(&settings).unwrap();
        database.save_session(snapshot()).unwrap();
        let count: u32 = database.connection.query_row("SELECT COUNT(*) FROM sessions", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 0);
        assert!(database.load_session().unwrap().is_none());
    }
    #[test]
    fn malformed_active_duplicate_and_oversized_sessions_are_rejected() {
        let mut data = snapshot();
        data.active_id = uuid::Uuid::new_v4().to_string();
        assert!(validate(&data).is_err());
        let mut data = snapshot();
        data.closed.push(data.tabs[0].clone());
        assert!(validate(&data).is_err());
        let mut data = snapshot();
        data.tabs[0].zoom = f64::NAN;
        assert!(validate(&data).is_err());
        let mut data = snapshot();
        data.tabs[0].url = "sab://unknown".into();
        assert!(validate(&data).is_err());
    }
}
