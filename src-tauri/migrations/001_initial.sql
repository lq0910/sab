-- 功能：建立本地单一数据源，预留修订、软删除和导入批次；时间：2026-11-08；作者：lq。
CREATE TABLE folders (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  parent_id TEXT REFERENCES folders(id),
  position INTEGER NOT NULL DEFAULT 0
);
INSERT INTO folders (id, name, position) VALUES
  ('development', '开发', 0), ('design', '设计', 1), ('work', '工作', 2),
  ('learning', '学习', 3), ('reading', '阅读', 4), ('life', '生活', 5),
  ('shopping', '购物', 6), ('entertainment', '娱乐', 7), ('inbox', '待整理', 8);
CREATE TABLE import_batches (
  id TEXT PRIMARY KEY, source TEXT NOT NULL, created_at INTEGER NOT NULL
);
CREATE TABLE bookmarks (
  id TEXT PRIMARY KEY,
  url TEXT NOT NULL,
  title TEXT NOT NULL,
  folder_id TEXT NOT NULL REFERENCES folders(id),
  tags TEXT NOT NULL DEFAULT '[]',
  pinned INTEGER NOT NULL DEFAULT 0,
  pinned_order INTEGER,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  revision INTEGER NOT NULL DEFAULT 1,
  deleted_at INTEGER,
  manual_lock INTEGER NOT NULL DEFAULT 1,
  rule_reason TEXT,
  import_batch_id TEXT REFERENCES import_batches(id)
);
CREATE UNIQUE INDEX bookmarks_live_url ON bookmarks(url) WHERE deleted_at IS NULL;
CREATE INDEX bookmarks_folder ON bookmarks(folder_id, updated_at DESC) WHERE deleted_at IS NULL;
CREATE TABLE settings (id INTEGER PRIMARY KEY CHECK(id = 1), value TEXT NOT NULL);
CREATE TABLE history (
  url TEXT PRIMARY KEY, title TEXT NOT NULL, visited_at INTEGER NOT NULL,
  hidden INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX history_recent ON history(visited_at DESC);
-- 功能：后续会话恢复与整理规则的结构预留，当前不对外宣称实现；时间：2026-11-08；作者：lq。
CREATE TABLE sessions (id TEXT PRIMARY KEY, value TEXT NOT NULL, updated_at INTEGER NOT NULL);
CREATE TABLE classification_rules (
  id TEXT PRIMARY KEY, pattern TEXT NOT NULL, folder_id TEXT NOT NULL REFERENCES folders(id),
  priority INTEGER NOT NULL DEFAULT 0, revision INTEGER NOT NULL DEFAULT 1
);
PRAGMA user_version = 1;
