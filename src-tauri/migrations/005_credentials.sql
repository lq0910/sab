-- 功能：只保存网站和用户名索引，不将密码正文写入 SQLite；时间：2026-11-16；作者：lq。
CREATE TABLE credentials (
  id TEXT PRIMARY KEY,
  origin TEXT NOT NULL,
  username TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  UNIQUE(origin, username)
);
CREATE INDEX credentials_origin ON credentials(origin, username);
PRAGMA user_version = 5;
