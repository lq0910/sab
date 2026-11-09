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
        let mut version: u32 = connection.query_row("PRAGMA user_version", [], |row| row.get(0)).map_err(error)?;
        if version > 3 { return Err("数据版本高于当前应用，请使用较新的轻页版本".into()); }
        if version == 0 {
            let transaction = connection.unchecked_transaction().map_err(error)?;
            transaction.execute_batch(include_str!("../migrations/001_initial.sql")).map_err(error)?;
            transaction.commit().map_err(error)?;
            version = 1;
        }
        if version == 1 {
            let transaction = connection.unchecked_transaction().map_err(error)?;
            transaction.execute_batch(include_str!("../migrations/002_classification_batches.sql")).map_err(error)?;
            transaction.commit().map_err(error)?;
            version = 2;
        }
        if version == 2 {
            let transaction = connection.unchecked_transaction().map_err(error)?;
            transaction.execute_batch(include_str!("../migrations/003_import_folder.sql")).map_err(error)?;
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

    // 功能：创建用户自定义收藏分类并追加到分类列表；时间：2026-11-08；作者：lq。
    pub fn create_folder(&self, raw_name: &str) -> Result<Folder> {
        let name = validate_folder_name(raw_name)?;
        let duplicate: bool = self.connection.query_row("SELECT EXISTS(SELECT 1 FROM folders WHERE name=?1 COLLATE NOCASE)", [name], |row| row.get(0)).map_err(error)?;
        if duplicate { return Err("已有同名分类".into()); }
        let position: i64 = self.connection.query_row("SELECT COALESCE(MAX(position),-1)+1 FROM folders", [], |row| row.get(0)).map_err(error)?;
        let id = Uuid::new_v4().to_string();
        self.connection.execute("INSERT INTO folders(id,name,position) VALUES(?1,?2,?3)", params![id, name, position]).map_err(error)?;
        Ok(Folder { id, name: name.to_string(), count: 0 })
    }

    // 功能：重命名用户分类并阻止改名系统待整理分类；时间：2026-11-08；作者：lq。
    pub fn rename_folder(&self, id: &str, raw_name: &str) -> Result<()> {
        if id == "inbox" { return Err("“待整理”是系统分类，不能改名".into()); }
        let name = validate_folder_name(raw_name)?;
        let exists: bool = self.connection.query_row("SELECT EXISTS(SELECT 1 FROM folders WHERE id=?1)", [id], |row| row.get(0)).map_err(error)?;
        if !exists { return Err("分类不存在".into()); }
        let duplicate: bool = self.connection.query_row("SELECT EXISTS(SELECT 1 FROM folders WHERE name=?1 COLLATE NOCASE AND id<>?2)", params![name, id], |row| row.get(0)).map_err(error)?;
        if duplicate { return Err("已有同名分类".into()); }
        self.connection.execute("UPDATE folders SET name=?1 WHERE id=?2", params![name, id]).map_err(error)?;
        Ok(())
    }

    // 功能：删除分类前将其书签迁至指定分类，并清理依赖旧分类 ID 的规则记录；时间：2026-11-08；作者：lq。
    pub fn delete_folder(&self, id: &str, target_id: &str) -> Result<u32> {
        if id == "inbox" { return Err("不能删除系统“待整理”分类".into()); }
        if id == target_id { return Err("目标分类必须与待删除分类不同".into()); }
        let transaction = self.connection.unchecked_transaction().map_err(error)?;
        let source_exists: bool = transaction.query_row("SELECT EXISTS(SELECT 1 FROM folders WHERE id=?1)", [id], |row| row.get(0)).map_err(error)?;
        let target_exists: bool = transaction.query_row("SELECT EXISTS(SELECT 1 FROM folders WHERE id=?1)", [target_id], |row| row.get(0)).map_err(error)?;
        if !source_exists { return Err("分类不存在".into()); }
        if !target_exists { return Err("目标分类不存在".into()); }
        let moved: u32 = transaction.query_row("SELECT COUNT(*) FROM bookmarks WHERE folder_id=?1 AND deleted_at IS NULL", [id], |row| row.get(0)).map_err(error)?;
        transaction.execute("UPDATE bookmarks SET folder_id=?1,manual_lock=1,rule_reason=NULL,updated_at=?2,revision=revision+1 WHERE folder_id=?3", params![target_id, now(), id]).map_err(error)?;
        transaction.execute("UPDATE folders SET parent_id=?1 WHERE parent_id=?2", params![target_id, id]).map_err(error)?;
        transaction.execute("DELETE FROM classification_rules WHERE folder_id=?1", [id]).map_err(error)?;
        transaction.execute("DELETE FROM classification_batch_items WHERE from_folder=?1 OR to_folder=?1", [id]).map_err(error)?;
        transaction.execute("DELETE FROM folders WHERE id=?1", [id]).map_err(error)?;
        transaction.commit().map_err(error)?;
        Ok(moved)
    }

    pub fn list(&self, query: &str, folder: &str, offset: u32, pinned: bool) -> Result<BookmarkPage> {
        if query.len() > 512 || folder.len() > 128 { return Err("检索条件过长".into()); }
        // 功能：按字面量检索通配符，限定每页 60 条；时间：2026-11-08；作者：lq。
        let pattern = format!("%{}%", query.trim().replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_"));
        let filter = r"FROM bookmarks WHERE deleted_at IS NULL AND (?1='' OR folder_id=?1) AND (?2=0 OR pinned=1) AND (title LIKE ?3 ESCAPE '\' OR url LIKE ?3 ESCAPE '\' OR tags LIKE ?3 ESCAPE '\')";
        let total = self.connection.query_row(&format!("SELECT COUNT(*) {filter}"), params![folder, pinned, pattern], |r| r.get(0)).map_err(error)?;
        let mut statement = self.connection.prepare(&format!("SELECT id,url,title,folder_id,tags,pinned,updated_at,rule_reason {filter} ORDER BY updated_at DESC,id LIMIT 60 OFFSET ?4")).map_err(error)?;
        let items = statement.query_map(params![folder, pinned, pattern, offset], |row| {
            let tags: String = row.get(4)?;
            let tags = serde_json::from_str(&tags).map_err(|e| rusqlite::Error::FromSqlConversionFailure(4, rusqlite::types::Type::Text, Box::new(e)))?;
            Ok(Bookmark { id: row.get(0)?, url: row.get(1)?, title: row.get(2)?, folder_id: row.get(3)?, tags, pinned: row.get(5)?, updated_at: row.get(6)?, rule_reason: row.get(7)? })
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

    // 功能：原子导入最多 5000 条书签，统一放入待整理且跳过重复或无效网址；时间：2026-11-08；作者：lq。
    pub fn import_bookmarks(&self, drafts: Vec<BookmarkDraft>) -> Result<BookmarkImportResult> {
        if drafts.len() > 5000 { return Err("单次最多导入 5000 条书签".into()); }
        let timestamp = now();
        let batch_id = Uuid::new_v4().to_string();
        let transaction = self.connection.unchecked_transaction().map_err(error)?;
        transaction.execute("INSERT INTO import_batches(id,source,created_at) VALUES(?1,'browser-html',?2)", params![batch_id, timestamp]).map_err(error)?;
        let mut imported = 0u32;
        let mut skipped = 0u32;
        let mut folder_metadata_updated = 0u32;
        for draft in drafts {
            let Ok(url) = website_url(&draft.url) else { skipped += 1; continue; };
            let title = draft.title.trim();
            if title.is_empty() || title.len() > 2048 { skipped += 1; continue; }
            let tags = match serde_json::to_string(&draft.tags) { Ok(value) => value, Err(_) => { skipped += 1; continue; } };
            let import_folder = draft.import_folder.as_deref().map(str::trim).filter(|value| !value.is_empty() && value.len() <= 512);
            let changed = transaction.execute(
                "INSERT INTO bookmarks(id,url,title,folder_id,tags,pinned,created_at,updated_at,revision,manual_lock,import_batch_id,import_folder)
                 VALUES(?1,?2,?3,'inbox',?4,0,?5,?5,1,0,?6,?7)
                 ON CONFLICT(url) WHERE deleted_at IS NULL DO NOTHING",
                params![Uuid::new_v4().to_string(), url.as_str(), title, tags, timestamp, batch_id, import_folder],
            ).map_err(error)?;
            if changed == 1 { imported += 1; } else {
                skipped += 1;
                if let Some(folder) = import_folder {
                    folder_metadata_updated += transaction.execute("UPDATE bookmarks SET import_folder=?1 WHERE url=?2 AND deleted_at IS NULL AND manual_lock=0 AND import_folder IS NULL", params![folder, url.as_str()]).map_err(error)? as u32;
                }
            }
        }
        transaction.commit().map_err(error)?;
        Ok(BookmarkImportResult { imported, skipped, folder_metadata_updated })
    }

    // 功能：预览未手动锁定的待整理书签，并汇总本地规则可以确定的分类建议；时间：2026-11-08；作者：lq。
    pub fn classification_preview(&self) -> Result<ClassificationPreview> {
        let rows = self.connection.prepare(
            "SELECT title,url,import_folder FROM bookmarks WHERE deleted_at IS NULL AND folder_id='inbox' AND manual_lock=0 ORDER BY title COLLATE NOCASE LIMIT 10000",
        ).map_err(error)?.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, Option<String>>(2)?)))
            .map_err(error)?.collect::<rusqlite::Result<Vec<_>>>().map_err(error)?;
        let mut groups: std::collections::BTreeMap<String, ClassificationGroup> = std::collections::BTreeMap::new();
        let mut remaining = 0u32;
        // 功能：记录少量无法可靠分类的书签标题供预览检查；时间：2026-11-09；作者：lq。
        let mut remaining_samples = Vec::new();
        for (title, url, import_folder) in rows {
            if let Some((folder_id, reason)) = classify_with_import_folder(&self.connection, &title, &url, import_folder.as_deref())? {
                let folder_name: String = self.connection.query_row("SELECT name FROM folders WHERE id=?1", [&folder_id], |row| row.get(0)).map_err(error)?;
                let group = groups.entry(folder_id.clone()).or_insert_with(|| ClassificationGroup { folder_id, folder_name, count: 0, samples: Vec::new() });
                group.count += 1;
                if group.samples.len() < 3 { group.samples.push(format!("{title} — {reason}")); }
            } else {
                remaining += 1;
                if remaining_samples.len() < 5 { remaining_samples.push(title); }
            }
        }
        let suggested = groups.values().map(|group| group.count).sum();
        Ok(ClassificationPreview { groups: groups.into_values().collect(), suggested, remaining, remaining_samples })
    }

    // 功能：依据本地规则归类并记录每条变更，生成可安全撤销的整理批次；时间：2026-11-08；作者：lq。
    pub fn apply_classification(&self) -> Result<ClassificationBatchResult> {
        let transaction = self.connection.unchecked_transaction().map_err(error)?;
        let rows = transaction.prepare(
            "SELECT id,title,url,import_folder FROM bookmarks WHERE deleted_at IS NULL AND folder_id='inbox' AND manual_lock=0 ORDER BY id LIMIT 10000",
        ).map_err(error)?.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, Option<String>>(3)?)))
            .map_err(error)?.collect::<rusqlite::Result<Vec<_>>>().map_err(error)?;
        let batch_id = Uuid::new_v4().to_string();
        let timestamp = now();
        transaction.execute("INSERT INTO classification_batches(id,created_at) VALUES(?1,?2)", params![batch_id, timestamp]).map_err(error)?;
        let mut changed = 0u32;
        let mut remaining = 0u32;
        for (id, title, url, import_folder) in rows {
            if let Some((folder_id, reason)) = classify_with_import_folder(&transaction, &title, &url, import_folder.as_deref())? {
                transaction.execute(
                    "INSERT INTO classification_batch_items(batch_id,bookmark_id,from_folder,to_folder,reason) VALUES(?1,?2,'inbox',?3,?4)",
                    params![batch_id, id, folder_id, reason],
                ).map_err(error)?;
                transaction.execute(
                    "UPDATE bookmarks SET folder_id=?1,rule_reason=?2,updated_at=?3,revision=revision+1 WHERE id=?4 AND deleted_at IS NULL AND manual_lock=0 AND folder_id='inbox'",
                    params![folder_id, reason, timestamp, id],
                ).map_err(error)?;
                changed += 1;
            } else { remaining += 1; }
        }
        transaction.commit().map_err(error)?;
        Ok(ClassificationBatchResult { batch_id, changed, remaining })
    }

    // 功能：撤销最近一次未撤销的自动分类且保留之后的手动移动；时间：2026-11-08；作者：lq。
    pub fn undo_classification(&self, batch_id: &str) -> Result<u32> {
        let transaction = self.connection.unchecked_transaction().map_err(error)?;
        let mut statement = transaction.prepare(
            "SELECT bookmark_id,from_folder,to_folder FROM classification_batch_items WHERE batch_id=?1",
        ).map_err(error)?;
        let rows = statement.query_map([batch_id], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?)))
            .map_err(error)?.collect::<rusqlite::Result<Vec<_>>>().map_err(error)?;
        drop(statement);
        let mut changed = 0u32;
        for (id, from_folder, to_folder) in rows {
            changed += transaction.execute(
                "UPDATE bookmarks SET folder_id=?1,rule_reason=NULL,updated_at=?2,revision=revision+1 WHERE id=?3 AND folder_id=?4 AND manual_lock=0 AND deleted_at IS NULL",
                params![from_folder, now(), id, to_folder],
            ).map_err(error)? as u32;
        }
        transaction.execute("UPDATE classification_batches SET undone_at=?1 WHERE id=?2 AND undone_at IS NULL", params![now(), batch_id]).map_err(error)?;
        transaction.commit().map_err(error)?;
        Ok(changed)
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

    // 功能：搜索并分页返回本地历史，限制查询长度和单页条数；时间：2026-11-08；作者：lq。
    pub fn history(&self, query: &str, offset: u32, limit: u32) -> Result<HistoryPage> {
        if query.len() > 512 || limit == 0 || limit > 100 { return Err("历史检索条件不合法".into()); }
        let pattern = format!("%{}%", query.trim().replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_"));
        let total = self.connection.query_row(
            "SELECT COUNT(*) FROM history WHERE (?1='' OR title LIKE ?2 ESCAPE '\\' OR url LIKE ?2 ESCAPE '\\')",
            params![query.trim(), pattern], |row| row.get(0),
        ).map_err(error)?;
        let mut statement = self.connection.prepare(
            "SELECT url,title,visited_at FROM history WHERE (?1='' OR title LIKE ?2 ESCAPE '\\' OR url LIKE ?2 ESCAPE '\\') ORDER BY visited_at DESC LIMIT ?3 OFFSET ?4",
        ).map_err(error)?;
        let items = statement.query_map(params![query.trim(), pattern, limit, offset], |row| Ok(RecentVisit {
            url: row.get(0)?, title: row.get(1)?, visited_at: row.get(2)?,
        })).map_err(error)?.collect::<rusqlite::Result<Vec<_>>>().map_err(error)?;
        Ok(HistoryPage { items, total })
    }

    // 功能：网页加载完成后记录访问并刷新重复网址的时间与标题；时间：2026-11-08；作者：lq。
    pub fn record_visit(&self, raw_url: &str, raw_title: &str) -> Result<()> {
        let url = website_url(raw_url)?.to_string();
        let title = raw_title.trim();
        if title.len() > 2048 { return Err("网页标题过长".into()); }
        let title = if title.is_empty() { url::Url::parse(&url).map_err(error)?.host_str().unwrap_or(&url).to_string() } else { title.to_string() };
        self.connection.execute(
            "INSERT INTO history(url,title,visited_at,hidden) VALUES(?1,?2,?3,0) ON CONFLICT(url) DO UPDATE SET title=excluded.title,visited_at=excluded.visited_at,hidden=0",
            params![url, title, now()],
        ).map_err(error)?;
        Ok(())
    }

    // 功能：从浏览历史中删除指定网址记录；时间：2026-11-08；作者：lq。
    pub fn delete_history(&self, raw_url: &str) -> Result<()> {
        let url = website_url(raw_url)?.to_string();
        self.connection.execute("DELETE FROM history WHERE url=?1", [url]).map_err(error)?;
        Ok(())
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
// 功能：按明确域名映射和标题关键词评分返回高置信度本地分类及可读原因；时间：2026-11-08；作者：lq。
fn classify_bookmark(title: &str, raw_url: &str, import_folder: Option<&str>) -> Option<(String, String)> {
    let url = url::Url::parse(raw_url).ok()?;
    let host = url.host_str()?.trim_start_matches("www.").to_ascii_lowercase();
    if let Some(folder) = import_folder.and_then(folder_from_import_path) {
        return Some((folder.to_string(), format!("浏览器原目录：{}", import_folder.unwrap_or_default())));
    }
    let direct_rules: &[(&str, &[&str])] = &[
        // 功能：补充常见行业工具站点并统一归入通用大类；时间：2026-11-08；作者：lq。
        ("development", &["tympanus.net", "gdal3.js.org", "weui.io", "semantic-ui.cn", "semantic-ui-cn.com", "mapshaper.org", "devhints.io", "opencv.org", "remotion.dev", "adc4gis.com", "gwmodel.whu.edu.cn", "hxkj.vip", "rainbond.cc", "56idc.net", "chinageoss.cn", "datav.aliyun.com"]),
        ("design", &["gradientmagic.com", "iconfinder.com", "ailogo.qq.com", "logaster.com", "pkgif.net", "tiancode.cn"]),
        ("work", &["wxb.com", "faisco.cn", "wentu.io", "baimiaoapp.com", "wps.cn", "jsform.com", "beian.gov.cn", "innocom.gov.cn", "nmgp.gov.cn", "tongyi.aliyun.com", "gemini-ai.top"]),
        ("life", &["tenpay.com"]),
        ("reading", &["qiaomi.cn", "ruancang.net", "039m.com"]),
        ("development", &["github.com", "gitlab.com", "stackoverflow.com", "developer.mozilla.org", "developers.weixin.qq.com", "developers.arcgis.com", "docs.rs", "crates.io", "npmjs.com", "pypi.org", "rust-lang.org", "typescriptlang.org", "vuejs.org", "tauri.app", "apicloud.com", "emqx.com", "codepen.io", "bootcss.com", "wangeditor.com", "regexper.com", "leaflet.github.io", "lingdaima.com", "dev.to", "blog.csdn.net", "csdn.net", "juejin.cn", "cnblogs.com", "segmentfault.com", "blog.51cto.com", "w3cfuns.com", "codexzh.com", "codexauv.com", "docker.aityp.com", "stableproxy.top", "stablenet.top", "api.xbai.top"]),
        ("design", &["figma.com", "dribbble.com", "behance.net", "unsplash.com", "pexels.com", "canva.com", "uisdc.com", "uisck.com", "ui.cn", "iconfont.cn", "icomoon.io", "yesicon.app", "undraw.co", "flaticon.com", "pixabay.com", "m3.material.io", "gradienta.io", "hereitis.cn", "ipaiban.com"]),
        ("work", &["notion.so", "feishu.cn", "larkoffice.com", "dingtalk.com", "slack.com", "trello.com", "atlassian.net", "jira.com", "tapd.cn", "huoban.com", "sitekc.com", "signin.aliyun.com", "qiye.aliyun.com", "analytics.singsound.com", "tongji.baidu.com"]),
        ("learning", &["coursera.org", "edx.org", "khanacademy.org", "mooc.cn", "icourse163.org", "study.163.com", "w3school.com.cn", "fanyi.baidu.com"]),
        ("reading", &["medium.com", "substack.com", "weread.qq.com", "msn.com"]),
        ("shopping", &["amazon.com", "amazon.cn", "jd.com", "taobao.com", "tmall.com", "1688.com", "pinduoduo.com"]),
        ("entertainment", &["netflix.com", "iqiyi.com", "youku.com", "spotify.com", "steampowered.com", "douyin.com"]),
        ("life", &["amap.com", "12306.cn", "meituan.com", "dianping.com", "weather.com.cn", "fuwu.alipay.com"]),
    ];
    // 功能：站点仅作为类别先验，内容证据可覆盖综合站点默认用途；时间：2026-11-09；作者：lq。
    let domain_hint = direct_rules.iter().flat_map(|(folder, domains)| domains.iter().map(move |domain| (*folder, *domain)))
        .filter(|(_, domain)| host == *domain || host.ends_with(&format!(".{domain}")))
        .max_by_key(|(_, domain)| domain.len());
    let title_text = title.to_lowercase();
    let path_text = url.path().to_lowercase();
    // 功能：用明确用途短语识别跨站点行业工具，避免多个通用关键词互相抵消；时间：2026-11-08；作者：lq。
    let purpose_rules: &[(&str, &[&str])] = &[
        ("development", &["gitlab", "docker hub", "container image", "developer document", "opencv", "gdal", "geojson", "ogr2ogr", "geospatial", "semantic ui", "wordpress", "开源框架", "云计算", "镜像仓库", "secret generator", "云主机", "开发工具", "代码生成"]),
        ("design", &["logo creation", "aidesign", "gradient", "iconfinder", "玩转gif", "矢量图", "设计素材"]),
        ("work", &["微信编辑器", "表单设计", "数据收集", "政府采购", "网站安全", "企业认定", "行政区划", "图片转文字", "pdf转word", "ai chat", "aichat", "gemini", "通义千问", "chatgpt", "claude", "ai助手", "ai 助手", "在线办公"]),
        ("reading", &["今日头条", "绿色软件", "网盘资源", "软件下载"]),
        ("life", &["财付通", "支付平台", "生活服务"]),
    ];
    let keyword_rules: &[(&str, &[&str])] = &[
        ("development", &["开发文档", "技术文档", "网页开发", "编程", "开发", "代码", "数据库", "mysql", "sql语句", "linux", "java", "python", "typescript", "react", "前端", "后端", "源码", "服务器", "部署", "运维", "接口", "api", "sdk", "rust", "javascript", "vue", "css", "html", "ios", "arcgis", "leaflet", "geoserver", "nginx", "gradle", "vmware", "macos", "http", "cors", "框架", "组件", "测试", "小程序", "开发者文档", "样式库", "emqx", "mqtt", "fastcgi", "jquery", "flutter"]),
        ("design", &["ui素材", "设计", "图标", "字体", "插画", "配色", "原型", "figma", "素材库", "背景图", "图片素材", "图片下载", "图片素材下载", "ppt", "icon font", "illustration", "blurred header"]),
        ("work", &["协作", "项目管理", "工作台", "企业办公", "流程", "工单", "管理后台", "管理系统", "管理平台", "运营平台", "报表", "客服", "crm", "仪表盘", "dashboard", "控制台", "微信公众平台", "数据可视化", "数据统计", "网站建设", "代理平台", "api key"]),
        ("learning", &["教程", "课程", "学习", "大学", "学堂", "培训", "考试", "题库", "入门", "教学", "mooc", "手把手", "零基础", "在线教程", "翻译", "安装虚拟机", "详解", "百度经验", "指南"]),
        ("reading", &["文章", "博客", "阅读", "新闻", "资讯", "小说", "书籍", "电子书", "专栏", "深度", "随笔", "书摘", "breaking news"]),
        ("life", &["地图", "天气", "公交", "铁路", "酒店", "餐饮", "健康", "政务", "社保", "快递", "邮箱", "外卖"]),
        ("shopping", &["购物", "商城", "购买", "商品", "折扣", "优惠券", "京东", "淘宝", "天猫", "amazon", "shop"]),
        ("entertainment", &["视频", "音乐", "电影", "游戏", "直播", "动漫", "影视", "影音", "娱乐", "spotify", "无删减", "高清在线观看", "全集高清", "在线观看"]),
    ];
    // 功能：融合站点、用途短语和独立关键词，去除嵌套关键词重复计分；时间：2026-11-09；作者：lq。
    let mut scores: Vec<(u32, &str, Vec<String>)> = keyword_rules.iter().map(|(folder, keywords)| {
        let mut score = 0;
        let mut evidence = Vec::new();
        if let Some((domain_folder, domain)) = domain_hint {
            if domain_folder == *folder { score += 4; evidence.push(format!("站点：{domain}")); }
        }
        let phrases = purpose_rules.iter().filter(|(category, _)| category == folder)
            .flat_map(|(_, phrases)| phrases.iter().copied()).collect::<Vec<_>>();
        let mut title_matches = phrases.iter().chain(keywords.iter()).copied()
            .filter(|term| classification_term_matches(&title_text, term)).collect::<Vec<_>>();
        title_matches.sort_unstable();
        title_matches.dedup();
        let independent = title_matches.iter().copied().filter(|term| !title_matches.iter().any(|other| other.len() > term.len() && other.contains(term))).collect::<Vec<_>>();
        for term in independent.iter().take(6) {
            score += if phrases.contains(term) { 7 } else if term.chars().count() >= 4 { 4 } else { 3 };
            evidence.push(format!("标题：{term}"));
        }
        // 功能：只使用 URL 路径的完整词匹配，不以查询参数、令牌或路径扩展名推断用途；时间：2026-11-09；作者：lq。
        let path_matches = keywords.iter().copied().filter(|term| classification_term_matches(&path_text, term)).collect::<Vec<_>>();
        for term in path_matches.iter().filter(|term| !path_matches.iter().any(|other| other.len() > term.len() && other.contains(*term))).take(3) {
            score += 1;
            evidence.push(format!("路径：{term}"));
        }
        (score, *folder, evidence)
    }).collect();
    scores.sort_by(|left, right| right.0.cmp(&left.0));
    let best = scores.first()?;
    let second = scores.get(1)?.0;
    // 功能：要求证据强度和领先幅度同时达标，冲突或信息不足时保留待整理；时间：2026-11-09；作者：lq。
    if best.0 < 3 || best.0.saturating_sub(second) < 3 { return None; }
    let evidence = best.2.iter().take(4).cloned().collect::<Vec<_>>().join("；");
    Some((best.1.to_string(), format!("综合证据评分 {}，领先 {}：{evidence}", best.0, best.0 - second)))
}

// 功能：英文关键词使用单词边界避免 rust 命中 trust，中文保持短语匹配；时间：2026-11-09；作者：lq。
fn classification_term_matches(text: &str, term: &str) -> bool {
    text.match_indices(term).any(|(offset, _)| {
        if !term.is_ascii() { return true; }
        let before = text[..offset].chars().next_back();
        let after = text[offset + term.len()..].chars().next();
        !before.is_some_and(|c| c.is_ascii_alphanumeric()) && !after.is_some_and(|c| c.is_ascii_alphanumeric())
    })
}

// 功能：优先读取浏览器导入目录中明确的分类名，避免重复猜测已有用户整理意图；时间：2026-11-08；作者：lq。
fn folder_from_import_path(path: &str) -> Option<&'static str> {
    let categories: &[(&str, &[&str])] = &[
        ("development", &["development", "developer", "dev", "coding", "programming", "技术", "开发", "编程", "代码", "前端", "后端", "网页", "运维"]),
        ("design", &["design", "ui", "ux", "设计", "图标", "素材", "图片"]),
        ("work", &["work", "office", "business", "工作", "办公", "企业", "项目", "管理", "物业", "社区"]),
        ("learning", &["learning", "learn", "study", "education", "tutorial", "course", "学习", "教程", "课程", "教育", "培训", "题库"]),
        ("reading", &["reading", "read", "article", "blog", "阅读", "文章", "博客", "资讯", "新闻", "书籍", "小说"]),
        ("life", &["life", "生活", "出行", "地图", "健康", "旅行", "日常"]),
        ("shopping", &["shopping", "shop", "购物", "商城", "电商", "优惠"]),
        ("entertainment", &["entertainment", "video", "music", "game", "娱乐", "视频", "音乐", "游戏", "影视", "动漫"]),
    ];
    let mut matched = None;
    for part in path.split('/').map(str::trim).filter(|part| !part.is_empty()) {
        let name = part.to_lowercase();
        for (folder, aliases) in categories {
            if aliases.iter().any(|alias| name == *alias || if alias.is_ascii() { alias.len() >= 3 && name.contains(alias) } else { name.contains(alias) }) {
                if matched.is_some_and(|previous| previous != *folder) { return None; }
                matched = Some(*folder);
            }
        }
    }
    matched
}

// 功能：为自动分类结果提供本地预置目录名称；时间：2026-11-08；作者：lq。
// 功能：优先使用预置规则，否则按用户自建分类名匹配导入目录；时间：2026-11-08；作者：lq。
fn classify_with_import_folder(connection: &Connection, title: &str, url: &str, import_folder: Option<&str>) -> Result<Option<(String, String)>> {
    if let Some((category_id, reason)) = classify_bookmark(title, url, import_folder) {
        if let Some((id, _)) = resolve_category_id(connection, &category_id)? { return Ok(Some((id, reason))); }
    }
    let Some(import_folder) = import_folder else { return Ok(None); };
    for part in import_folder.split('/').map(str::trim).filter(|part| !part.is_empty()) {
        let found: Option<(String, String)> = connection.query_row(
            "SELECT id,name FROM folders WHERE name=?1 COLLATE NOCASE AND id<>'inbox' LIMIT 1",
            [part], |row| Ok((row.get(0)?, row.get(1)?)),
        ).optional().map_err(error)?;
        if let Some((id, name)) = found { return Ok(Some((id, format!("浏览器原目录匹配：{name}")))); }
    }
    Ok(None)
}

// 功能：将内置规则类别解析到当前存在的目录，兼容类别改名、删除和同名重建；时间：2026-11-08；作者：lq。
fn resolve_category_id(connection: &Connection, category_id: &str) -> Result<Option<(String, String)>> {
    let by_id: Option<(String, String)> = connection.query_row("SELECT id,name FROM folders WHERE id=?1", [category_id], |row| Ok((row.get(0)?, row.get(1)?))).optional().map_err(error)?;
    if by_id.is_some() { return Ok(by_id); }
    let Some(default_name) = category_default_name(category_id) else { return Ok(None); };
    connection.query_row("SELECT id,name FROM folders WHERE name=?1 COLLATE NOCASE AND id<>'inbox' LIMIT 1", [default_name], |row| Ok((row.get(0)?, row.get(1)?))).optional().map_err(error)
}

// 功能：提供内置分类的默认名称以恢复重建后的规则目标；时间：2026-11-08；作者：lq。
fn category_default_name(id: &str) -> Option<&'static str> {
    match id {
        "development" => Some("开发"), "design" => Some("设计"), "work" => Some("工作"), "learning" => Some("学习"),
        "reading" => Some("阅读"), "life" => Some("生活"), "shopping" => Some("购物"), "entertainment" => Some("娱乐"),
        _ => None,
    }
}

// 功能：校验用户分类名称长度并返回去除首尾空白的名称；时间：2026-11-08；作者：lq。
fn validate_folder_name(raw_name: &str) -> Result<&str> {
    let name = raw_name.trim();
    if name.is_empty() || name.chars().count() > 32 || name.chars().any(char::is_control) {
        return Err("分类名称需为 1 到 32 个字符".into());
    }
    Ok(name)
}

fn now() -> i64 { SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as i64 }

#[cfg(test)]
mod tests {
    use super::*;
    // 功能：验证英文边界、综合站点内容覆盖和冲突拒绝，防止为覆盖率强行误分；时间：2026-11-09；作者：lq。
    #[test]
    fn weighted_classification_handles_ambiguity() {
        assert!(!classification_term_matches("trust capital", "rust"));
        assert!(!classification_term_matches("shopping", "ping"));
        assert!(classification_term_matches("Rust SDK", "Rust"));
        assert_eq!(classify_bookmark("Python 开发教程", "https://bilibili.com/watch/1", None).unwrap().0, "development");
        assert_eq!(classify_bookmark("电影 高清在线观看", "https://csdn.net/article/1", None).unwrap().0, "entertainment");
        assert!(classify_bookmark("设计 学习", "https://example.org/", None).is_none());
        assert!(classify_bookmark("Trust Capital", "https://example.org/?tag=rust", None).is_none());
    }
    // 功能：验证开发、办公与设计工具归入通用大类，并拒绝相似域名伪匹配；时间：2026-11-08；作者：lq。
    #[test]
    fn broad_classification_recognizes_tool_purpose() {
        for (title, url, expected) in [
            ("gdal3.js", "https://gdal3.js.org/", "development"),
            ("Semantic UI 中文社区", "https://semantic-ui-cn.com/", "development"),
            ("公司代码仓库 GitLab", "http://192.168.3.2/project", "development"),
            ("PDF转Word在线", "https://example.org/convert", "work"),
            ("高新技术企业认定工作网", "https://innocom.gov.cn/", "work"),
            ("Logo creation wizard", "https://example.org/logo", "design"),
        ] {
            assert_eq!(classify_bookmark(title, url, None).unwrap().0, expected);
        }
        assert!(classify_bookmark("未知页面", "https://iconfinder.com.example.org/", None).is_none());
    }
    // 功能：验证分类改名、删除迁移、自动分类与撤销之间的外键和手动锁定约束；时间：2026-11-08；作者：lq。
    #[test]
    fn folder_management_preserves_bookmarks_and_classification() {
        let database = store();
        database.import_bookmarks(vec![draft("https://github.com/example/project")]).unwrap();
        database.rename_folder("development", "技术研发").unwrap();
        assert_eq!(database.classification_preview().unwrap().groups[0].folder_name, "技术研发");
        let batch = database.apply_classification().unwrap();
        assert_eq!(batch.changed, 1);
        let folder = database.create_folder("项目资料").unwrap();
        assert!(database.create_folder(" 项目资料 ").is_err());
        assert!(database.rename_folder(&folder.id, "设计").is_err());
        assert!(database.delete_folder("inbox", &folder.id).is_err());
        assert!(database.delete_folder("development", "missing").is_err());
        assert_eq!(database.delete_folder("development", &folder.id).unwrap(), 1);
        assert_eq!(database.undo_classification(&batch.batch_id).unwrap(), 0);
        let saved = database.list("", &folder.id, 0, false).unwrap();
        assert_eq!(saved.total, 1);
        assert_eq!(saved.items[0].title, "Rust 中文文档");
        database.import_bookmarks(vec![draft("https://github.com/another/project")]).unwrap();
        assert_eq!(database.classification_preview().unwrap().suggested, 0);
        database.create_folder("开发").unwrap();
        assert_eq!(database.classification_preview().unwrap().suggested, 1);
    }
    fn store() -> Store { Store::initialize(Connection::open_in_memory().unwrap()).unwrap() }
    fn draft(url: &str) -> BookmarkDraft {
        BookmarkDraft { url: url.into(), title: "Rust 中文文档".into(), folder_id: "development".into(), tags: vec!["rust".into()], pinned: false, import_folder: None }
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
