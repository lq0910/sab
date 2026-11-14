-- 功能：保存下载状态、用户选定位置和中断恢复信息；时间：2026-11-14；作者：lq。
CREATE TABLE downloads (
  id TEXT PRIMARY KEY, url TEXT NOT NULL, tab_id TEXT NOT NULL,
  source_url TEXT NOT NULL, name TEXT NOT NULL, path TEXT NOT NULL,
  temp_path TEXT NOT NULL, status TEXT NOT NULL,
  received_bytes INTEGER NOT NULL DEFAULT 0, started_at INTEGER NOT NULL,
  finished_at INTEGER, error TEXT NOT NULL DEFAULT '', overwrite_allowed INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX downloads_recent ON downloads(started_at DESC);
CREATE INDEX downloads_active ON downloads(tab_id, url) WHERE status='downloading';
PRAGMA user_version = 4;
