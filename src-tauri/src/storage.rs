// 功能：管理 SQLite 迁移、收藏、偏好和最近记录；时间：2026-11-08；作者：lq。
use crate::{models::*, security::website_url};
use rusqlite::{params, Connection, OptionalExtension};
use std::{path::Path, time::{SystemTime, UNIX_EPOCH}};
use uuid::Uuid;

pub struct Store { connection: Connection }
type Result<T> = std::result::Result<T, String>;

impl Store {
    pub fn open(path: &Path) -> Result<Self> {
        let connection = Connection::open(path).map_err(error)?;
        Self::initialize(connection)
    }

    fn initialize(connection: Connection) -> Result<Self> {
        connection.busy_timeout(std::time::Duration::from_secs(5)).map_err(error)?;
        connection.execute_batch("PRAGMA foreign_keys=ON; PRAGMA journal_mode=WAL;").map_err(error)?;
        let version: u32 = connection.query_row("PRAGMA user_version", [], |row| row.get(0)).map_err(error)?;
        if version > 1 { return Err("数据版本高于当前应用，请使用较新的轻页版本".into()); }
        if version == 0 {
            let transaction = connection.unchecked_transaction().map_err(error)?;
            transaction.execute_batch(include_str!("../migrations/001_initial.sql")).map_err(error)?;
            transaction.commit().map_err(error)?;
        }
        Ok(Self { connection })
    }

    pub fn bootstrap(&self) -> Result<Bootstrap> {
        let value: Option<String> = self.connection.query_row("SELECT value FROM settings WHERE id=1", [], |r| r.get(0)).optional().map_err(error)?;
        let settings = value.map(|s| serde_json::from_str(&s).map_err(error)).transpose()?.unwrap_or_default();
        let mut statement = self.connection.prepare(
            "SELECT f.id, f.name, COUNT(b.id) FROM folders f LEFT JOIN bookmarks b ON b.folder_id=f.id AND b.deleted_at IS NULL GROUP BY f.id ORDER BY f.position"
        ).map_err(error)?;
        let folders = statement.query_map([], |row| Ok(Folder { id: row.get(0)?, name: row.get(1)?, count: row.get(2)? }))
            .map_err(error)?.collect::<rusqlite::Result<Vec<_>>>().map_err(error)?;
        Ok(Bootstrap { settings, folders })
    }

    pub fn list(&self, query: &str, folder: &str, offset: u32, pinned: bool) -> Result<BookmarkPage> {
        if query.len() > 512 || folder.len() > 128 { return Err("检索条件过长".into()); }
        // 功能：按字面量检索通配符，限定每页 60 条；时间：2026-11-08；作者：lq。
        let pattern = format!("%{}%", query.trim().replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_"));
        let filter = r"FROM bookmarks WHERE deleted_at IS NULL AND (?1='' OR folder_id=?1) AND (?2=0 OR pinned=1) AND (title LIKE ?3 ESCAPE '\' OR url LIKE ?3 ESCAPE '\' OR tags LIKE ?3 ESCAPE '\')";
        let total = self.connection.query_row(&format!("SELECT COUNT(*) {filter}"), params![folder, pinned, pattern], |r| r.get(0)).map_err(error)?;
        let mut statement = self.connection.prepare(&format!("SELECT id,url,title,folder_id,tags,pinned,updated_at {filter} ORDER BY updated_at DESC,id LIMIT 60 OFFSET ?4")).map_err(error)?;
        let items = statement.query_map(params![folder, pinned, pattern, offset], |row| {
            let tags: String = row.get(4)?;
            let tags = serde_json::from_str(&tags).map_err(|e| rusqlite::Error::FromSqlConversionFailure(4, rusqlite::types::Type::Text, Box::new(e)))?;
            Ok(Bookmark { id: row.get(0)?, url: row.get(1)?, title: row.get(2)?, folder_id: row.get(3)?, tags, pinned: row.get(5)?, updated_at: row.get(6)? })
        }).map_err(error)?.collect::<rusqlite::Result<Vec<_>>>().map_err(error)?;
        Ok(BookmarkPage { items, total })
    }

    pub fn save(&self, draft: BookmarkDraft) -> Result<()> {
        let url = website_url(&draft.url)?.to_string();
        let title = draft.title.trim();
        if title.is_empty() || title.len() > 2048 || draft.tags.len() > 32 || draft.tags.iter().any(|t| t.len() > 128) {
            return Err("标题或标签超出允许范围".into());
        }
        let tags = serde_json::to_string(&draft.tags).map_err(error)?;
        let timestamp = now();
        // 功能：重复 URL 更新原条目，修订递增且保留创建时间；时间：2026-11-08；作者：lq。
        self.connection.execute(
            "INSERT INTO bookmarks(id,url,title,folder_id,tags,pinned,created_at,updated_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?7)
            ON CONFLICT(url) WHERE deleted_at IS NULL DO UPDATE SET title=excluded.title,folder_id=excluded.folder_id,tags=excluded.tags,pinned=excluded.pinned,updated_at=excluded.updated_at,revision=bookmarks.revision+1,manual_lock=1",
            params![Uuid::new_v4().to_string(),url,title,draft.folder_id,tags,draft.pinned,timestamp],
        ).map_err(error)?;
        Ok(())
    }

    pub fn delete(&self, id: &str) -> Result<()> {
        self.connection.execute("UPDATE bookmarks SET deleted_at=?1,updated_at=?1,revision=revision+1 WHERE id=?2 AND deleted_at IS NULL", params![now(), id]).map_err(error)?;
        Ok(())
    }

    pub fn save_settings(&self, settings: &Settings) -> Result<()> {
        if !settings.startup_url.is_empty() { website_url(&settings.startup_url)?; }
        let value = serde_json::to_string(settings).map_err(error)?;
        self.connection.execute("INSERT INTO settings(id,value) VALUES(1,?1) ON CONFLICT(id) DO UPDATE SET value=excluded.value", [value]).map_err(error)?;
        Ok(())
    }

    pub fn recent(&self) -> Result<Vec<RecentVisit>> {
        let mut statement = self.connection.prepare("SELECT url,title,visited_at FROM history WHERE hidden=0 ORDER BY visited_at DESC LIMIT 20").map_err(error)?;
        let rows = statement.query_map([], |row| Ok(RecentVisit { url: row.get(0)?, title: row.get(1)?, visited_at: row.get(2)? }))
            .map_err(error)?.collect::<rusqlite::Result<Vec<_>>>().map_err(error)?;
        Ok(rows)
    }

    pub fn clear_history(&self) -> Result<()> {
        self.connection.execute("DELETE FROM history", []).map_err(error)?;
        Ok(())
    }

    pub fn hide_recent(&self, url: &str) -> Result<()> {
        self.connection.execute("UPDATE history SET hidden=1 WHERE url=?1", [url]).map_err(error)?;
        Ok(())
    }
}

fn error(value: impl std::fmt::Display) -> String { value.to_string() }
fn now() -> i64 { SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as i64 }

#[cfg(test)]
mod tests {
    use super::*;
    fn store() -> Store { Store::initialize(Connection::open_in_memory().unwrap()).unwrap() }
    fn draft(url: &str) -> BookmarkDraft {
        BookmarkDraft { url: url.into(), title: "Rust 中文文档".into(), folder_id: "development".into(), tags: vec!["rust".into()], pinned: false }
    }
    // 功能：验证迁移可重复执行、默认显示最近访问区；时间：2026-11-08；作者：lq。
    #[test]
    fn migration_and_defaults() {
        let database = store();
        let bootstrap = database.bootstrap().unwrap();
        assert!(bootstrap.settings.show_recent);
        assert_eq!(bootstrap.folders.len(), 9);
        let database = Store::initialize(database.connection).unwrap();
        assert_eq!(database.bootstrap().unwrap().folders.len(), 9);
    }
    // 功能：验证去重、分页、检索、软删除及手动锁定；时间：2026-11-08；作者：lq。
    #[test]
    fn bookmark_lifecycle() {
        let database = store();
        database.save(draft("https://example.com/?q=1#part")).unwrap();
        let first = database.list("中文", "", 0, false).unwrap().items.remove(0);
        database.save(draft("https://example.com/?q=1#part")).unwrap();
        assert_eq!(database.list("rust", "development", 0, false).unwrap().total, 1);
        assert_eq!(database.list("", "", 0, false).unwrap().items[0].id, first.id);
        database.save(draft("http://example.com/?q=1#part")).unwrap();
        database.save(draft("https://example.com/?q=2#part")).unwrap();
        assert_eq!(database.list("", "", 0, false).unwrap().total, 3);
        assert!(database.list("%", "", 0, false).unwrap().items.is_empty());
        database.delete(&first.id).unwrap();
        assert_eq!(database.list("", "", 0, false).unwrap().total, 2);
        assert_eq!(database.list("", "", 2, false).unwrap().items.len(), 0);
    }
    // 功能：验证非法输入不能进入数据库；时间：2026-11-08；作者：lq。
    #[test]
    fn invalid_writes_are_rejected() {
        let database = store();
        assert!(database.save(draft("javascript:alert(1)")).is_err());
        let mut invalid = draft("https://example.com"); invalid.folder_id = "missing".into();
        assert!(database.save(invalid).is_err());
        let settings = Settings { startup_url: "file:///tmp/x".into(), ..Settings::default() };
        assert!(database.save_settings(&settings).is_err());
        assert_eq!(database.list("", "", 0, false).unwrap().total, 0);
    }
    // 功能：验证设置跨连接保存，隐藏最近记录不删除历史；时间：2026-11-08；作者：lq。
    #[test]
    fn settings_and_history_boundaries() {
        let database = store();
        let settings = Settings { home_mode: HomeMode::Bookmarks, show_recent: true, ..Settings::default() };
        database.save_settings(&settings).unwrap();
        assert_eq!(database.bootstrap().unwrap().settings.home_mode, HomeMode::Bookmarks);
        database.connection.execute("INSERT INTO history VALUES('https://example.com/','Example',1,0)", []).unwrap();
        database.hide_recent("https://example.com/").unwrap();
        assert!(database.recent().unwrap().is_empty());
        let count: u32 = database.connection.query_row("SELECT COUNT(*) FROM history", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 1);
        database.clear_history().unwrap();
        let count: u32 = database.connection.query_row("SELECT COUNT(*) FROM history", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 0);
    }
}
