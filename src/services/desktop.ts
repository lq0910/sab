// 功能：封装本地命令；网页预览不伪装成原生浏览器或持久化存储；时间：2026-11-08；作者：lq。
import { invoke, isTauri } from '@tauri-apps/api/core';
import type { BookmarkDraft, BookmarkPage, Bootstrap, BrowserAction, RecentVisit, Settings, ViewBounds } from '../domain/models';
import { defaultSettings } from '../domain/models';

export const desktop = isTauri();
export const api = {
  bootstrap: (): Promise<Bootstrap> => desktop
    ? invoke('bootstrap') : Promise.resolve({ settings: { ...defaultSettings }, folders: [] }),
  bookmarks: (query = '', folderId = '', offset = 0, pinned = false): Promise<BookmarkPage> => desktop
    ? invoke('list_bookmarks', { query, folderId, offset, pinned }) : Promise.resolve({ items: [], total: 0 }),
  saveBookmark: (draft: BookmarkDraft): Promise<void> => invoke('save_bookmark', { draft }),
  deleteBookmark: (id: string): Promise<void> => invoke('delete_bookmark', { id }),
  saveSettings: (settings: Settings): Promise<void> => invoke('save_settings', { settings }),
  recent: (): Promise<RecentVisit[]> => desktop ? invoke('list_recent') : Promise.resolve([]),
  clearHistory: (): Promise<void> => invoke('clear_history'),
  hideRecent: (url: string): Promise<void> => invoke('hide_recent', { url }),
  view: (action: BrowserAction, tabId: string, url?: string, bounds?: ViewBounds): Promise<void> =>
    desktop ? invoke('browser_view', { action, tabId, url, bounds }) : Promise.reject(new Error('请通过 pnpm tauri:dev 启动桌面应用以浏览网页。')),
};
