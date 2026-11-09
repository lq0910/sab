-- 功能：保存浏览器导入时的原始目录层级并作为分类规则输入；时间：2026-11-08；作者：lq。
ALTER TABLE bookmarks ADD COLUMN import_folder TEXT;
PRAGMA user_version = 3;
