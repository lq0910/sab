// 功能：封装本地命令；网页预览不伪装成原生浏览器或持久化存储；时间：2026-11-08；作者：lq。
import { invoke, isTauri } from '@tauri-apps/api/core';
import type { SessionSnapshot, DownloadItem, BookmarkDraft, BookmarkImportResult, BookmarkPage, Bootstrap, BrowserAction, ClassificationBatchResult, ClassificationPreview, ClassificationUndoResult, HistoryPage, RecentVisit, Settings, ViewBounds } from '../domain/models';
import { defaultSettings } from '../domain/models';

export const desktop = isTauri();
export const api = {
  // 功能：真实读取默认项并由用户主动发起系统设置，外部链接在外壳就绪后获取；时间：2026-11-14；作者：lq。
  defaultBrowserStatus: (): Promise<DefaultBrowserStatus> => invoke('default_browser_status'),
  requestDefaultBrowser: (): Promise<DefaultBrowserStatus> => invoke('request_default_browser'),
  takeExternalLinks: (): Promise<string[]> => desktop ? invoke('take_external_links') : Promise.resolve([]),
  // 功能：按下载记录 ID 管理文件，远程网页不能指定任意本地路径；时间：2026-11-14；作者：lq。
  downloads: (): Promise<DownloadItem[]> => desktop ? invoke('list_downloads') : Promise.resolve([]),
  clearDownloads: (): Promise<void> => invoke('clear_downloads'),
  openDownload: (id: string, reveal = false): Promise<void> => invoke('open_download', { id, reveal }),
  // 功能：本地会话读取与持久化，不在网页预览中模拟持久化；时间：2026-11-14；作者：lq。
  loadSession: (): Promise<SessionSnapshot | null> => desktop ? invoke('load_session') : Promise.resolve(null),
  saveSession: (snapshot: SessionSnapshot): Promise<void> => desktop ? invoke('save_session', { snapshot }) : Promise.resolve(),
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
  // 功能：传递经过宿主验证的缩放比例，保持远程网页无 IPC 权限；时间：2026-11-14；作者：lq。
  view: (action: BrowserAction, tabId: string, url?: string, bounds?: ViewBounds, zoom?: number): Promise<void> =>
    desktop ? invoke('browser_view', { action, tabId, url, bounds, zoom }) : Promise.reject(new Error('请通过 pnpm tauri:dev 启动桌面应用以浏览网页。')),
};

// 功能：分别反馈HTTP和HTTPS的实际系统关联，避免误报全部设为默认；时间：2026-11-14；作者：lq。
export interface DefaultBrowserStatus { supported: boolean; http: boolean; https: boolean; message: string }
