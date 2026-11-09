// 功能：封装本地命令；网页预览不伪装成原生浏览器或持久化存储；时间：2026-11-08；作者：lq。
import { invoke, isTauri } from '@tauri-apps/api/core';
import type { BookmarkDraft, BookmarkImportResult, BookmarkPage, Bootstrap, BrowserAction, ClassificationBatchResult, ClassificationPreview, ClassificationUndoResult, HistoryPage, RecentVisit, Settings, ViewBounds } from '../domain/models';
import { defaultSettings } from '../domain/models';

export const desktop = isTauri();
export const api = {
  bootstrap: (): Promise<Bootstrap> => desktop
    ? invoke('bootstrap') : Promise.resolve({ settings: { ...defaultSettings }, folders: [] }),
  // 功能：提供书签分类的创建、重命名与指定目标删除操作；时间：2026-11-08；作者：lq。
  createFolder: (name: string): Promise<Bootstrap['folders'][number]> => invoke('create_folder', { name }),
  renameFolder: (id: string, name: string): Promise<void> => invoke('rename_folder', { id, name }),
  deleteFolder: (id: string, targetId: string): Promise<number> => invoke('delete_folder', { id, targetId }),
  bookmarks: (query = '', folderId = '', offset = 0, pinned = false): Promise<BookmarkPage> => desktop
    ? invoke('list_bookmarks', { query, folderId, offset, pinned }) : Promise.resolve({ items: [], total: 0 }),
  saveBookmark: (draft: BookmarkDraft): Promise<void> => invoke('save_bookmark', { draft }),
  // 功能：把解析后的浏览器书签批量写入本地收藏并返回导入统计；时间：2026-11-08；作者：lq。
  importBookmarks: (drafts: BookmarkDraft[]): Promise<BookmarkImportResult> => invoke('import_bookmarks', { drafts }),
  // 功能：获取待整理收藏的本地自动分类预览；时间：2026-11-08；作者：lq。
  classificationPreview: (): Promise<ClassificationPreview> => invoke('classification_preview'),
  // 功能：应用本地规则分类并创建撤销批次；时间：2026-11-08；作者：lq。
  applyClassification: (): Promise<ClassificationBatchResult> => invoke('apply_classification'),
  // 功能：撤销指定批次且不覆盖后续手动更改；时间：2026-11-08；作者：lq。
  undoClassification: (batchId: string): Promise<ClassificationUndoResult> => invoke('undo_classification', { batchId }),
  deleteBookmark: (id: string): Promise<void> => invoke('delete_bookmark', { id }),
  saveSettings: (settings: Settings): Promise<void> => invoke('save_settings', { settings }),
  recent: (): Promise<RecentVisit[]> => desktop ? invoke('list_recent') : Promise.resolve([]),
  // 功能：提供历史搜索、分页、联想和已打开页面记录的受限 IPC；时间：2026-11-08；作者：lq。
  history: (query = '', offset = 0, limit = 100): Promise<HistoryPage> => desktop
    ? invoke('list_history', { query, offset, limit }) : Promise.resolve({ items: [], total: 0 }),
  recordVisit: (url: string, title: string): Promise<void> => desktop ? invoke('record_visit', { url, title }) : Promise.resolve(),
  deleteHistory: (url: string): Promise<void> => invoke('delete_history', { url }),
  clearHistory: (): Promise<void> => invoke('clear_history'),
  hideRecent: (url: string): Promise<void> => invoke('hide_recent', { url }),
  view: (action: BrowserAction, tabId: string, url?: string, bounds?: ViewBounds): Promise<void> =>
    desktop ? invoke('browser_view', { action, tabId, url, bounds }) : Promise.reject(new Error('请通过 pnpm tauri:dev 启动桌面应用以浏览网页。')),
};
