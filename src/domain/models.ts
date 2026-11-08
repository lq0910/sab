// 功能：定义受限 IPC 的数据契约，SQLite 是持久化的唯一来源；时间：2026-11-08；作者：lq。
export type HomeMode = 'minimal' | 'bookmarks';
export type SearchEngine = 'bing' | 'baidu' | 'google' | 'duckduckgo';
export type ThemeId = 'obsidian' | 'blue' | 'violet' | 'emerald' | 'amber';
export interface Settings {
  homeMode: HomeMode;
  searchEngine: SearchEngine;
  showRecent: boolean;
  startupUrl: string;
  theme: ThemeId;
}
export interface Folder { id: string; name: string; count: number }
export interface Bookmark {
  id: string;
  url: string;
  title: string;
  folderId: string;
  tags: string[];
  pinned: boolean;
  updatedAt: number;
}
export interface BookmarkDraft {
  url: string;
  title: string;
  folderId: string;
  tags: string[];
  pinned: boolean;
}
export interface BookmarkPage { items: Bookmark[]; total: number }
export interface Bootstrap { settings: Settings; folders: Folder[] }
export interface RecentVisit { url: string; title: string; visitedAt: number }
export interface Tab { id: string; url: string; title: string; loading: boolean }
export interface ViewBounds { x: number; y: number; width: number; height: number }
export type BrowserAction = 'open' | 'activate' | 'resize' | 'hide' | 'close' | 'reload' | 'back' | 'forward';
export interface PageEvent { tabId: string; url: string; title?: string; loading?: boolean }
export const defaultSettings: Settings = {
  // 功能：默认主页显示最近访问区，即使为空也展示品牌提示；时间：2026-11-08；作者：lq。
  homeMode: 'minimal', searchEngine: 'bing', showRecent: true, startupUrl: '', theme: 'obsidian',
};
