-- 功能：保存自动分类批次及原目录，用于无损撤销分类变更；时间：2026-11-08；作者：lq。
CREATE TABLE classification_batches (
  id TEXT PRIMARY KEY,
  created_at INTEGER NOT NULL,
  undone_at INTEGER
);
CREATE TABLE classification_batch_items (
  batch_id TEXT NOT NULL REFERENCES classification_batches(id),
  bookmark_id TEXT NOT NULL REFERENCES bookmarks(id),
  from_folder TEXT NOT NULL REFERENCES folders(id),
  to_folder TEXT NOT NULL REFERENCES folders(id),
  reason TEXT NOT NULL,
  PRIMARY KEY(batch_id, bookmark_id)
);
CREATE INDEX classification_batch_recent ON classification_batches(created_at DESC);
PRAGMA user_version = 2;
