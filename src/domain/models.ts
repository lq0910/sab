// 功能：定义受限 IPC 的数据契约，SQLite 是持久化的唯一来源；时间：2026-11-08；作者：lq。
export type HomeMode = 'minimal' | 'bookmarks';
export type SearchEngine = 'bing' | 'baidu' | 'google' | 'duckduckgo';
export type ThemeId = 'obsidian' | 'blue' | 'violet' | 'emerald' | 'amber';
// 功能：定义自动、浅色、深色三种界面明暗偏好；时间：2026-11-08；作者：lq。
export type ColorMode = 'auto' | 'light' | 'dark';
export interface Settings {
  homeMode: HomeMode;
  searchEngine: SearchEngine;
  showRecent: boolean;
  startupUrl: string;
  theme: ThemeId;
  // 功能：保存界面明暗模式并允许自动跟随系统；时间：2026-11-08；作者：lq。
  colorMode: ColorMode;
  // 功能：允许启动时恢复上次标签并支持关闭后清理会话；时间：2026-11-14；作者：lq。
  restoreSession: boolean;
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
  // 功能：展示自动归类依据并便于用户判断分类结果；时间：2026-11-08；作者：lq。
  ruleReason?: string | null;
}
export interface BookmarkDraft {
  url: string;
  title: string;
  folderId: string;
  tags: string[];
  pinned: boolean;
  // 功能：保留浏览器书签原目录作为本地分类线索；时间：2026-11-08；作者：lq。
  importFolder?: string;
}
// 功能：返回浏览器书签批量导入新增与跳过数量；时间：2026-11-08；作者：lq。
export interface BookmarkImportResult {
  imported: number;
  skipped: number;
  // 功能：提示重复导入时补回的浏览器原目录线索数量；时间：2026-11-08；作者：lq。
  folderMetadataUpdated: number;
}
// 功能：呈现自动分类预览的分组计数与示例；时间：2026-11-08；作者：lq。
export interface ClassificationGroup { folderId: string; folderName: string; count: number; samples: string[] }
// 功能：显示规则建议数量及仍需手动整理的数量；时间：2026-11-08；作者：lq。
// 功能：展示综合评分无法确定的样例，评分表示证据强度而非准确率；时间：2026-11-09；作者：lq。
export interface ClassificationPreview { groups: ClassificationGroup[]; suggested: number; remaining: number; remainingSamples: string[] }
// 功能：保存可撤销分类批次和本次移动统计；时间：2026-11-08；作者：lq。
export interface ClassificationBatchResult { batchId: string; changed: number; remaining: number }
// 功能：汇报自动分类撤销成功还原的条目数；时间：2026-11-08；作者：lq。
export interface ClassificationUndoResult { restored: number }
export interface BookmarkPage { items: Bookmark[]; total: number }
export interface Bootstrap { settings: Settings; folders: Folder[] }
// 功能：统一主页最近访问、地址栏联想与历史页使用的访问记录结构；时间：2026-11-08；作者：lq。
export interface RecentVisit { url: string; title: string; visitedAt: number }
// 功能：承载历史列表的分页结果，避免完整历史一次性加载；时间：2026-11-08；作者：lq。
export interface HistoryPage { items: RecentVisit[]; total: number }
// 功能：按标签保存错误、缩放和休眠状态，避免切换标签串用状态；时间：2026-11-14；作者：lq。
export interface Tab { id: string; url: string; title: string; loading: boolean; error?: string; loadIssue?: 'timeout' | 'offline'; suspended?: boolean; zoom?: number }
export interface ViewBounds { x: number; y: number; width: number; height: number }
// 功能：增加停止加载与原生网页缩放操作；时间：2026-11-14；作者：lq。
export type BrowserAction = 'open' | 'activate' | 'resize' | 'hide' | 'close' | 'reload' | 'back' | 'forward' | 'stop' | 'zoom' | 'focusshell' | 'focuspage';
export interface PageEvent { tabId: string; url: string; title?: string; loading?: boolean }
export const defaultSettings: Settings = {
  // 功能：默认主页显示最近访问区，即使为空也展示品牌提示；时间：2026-11-08；作者：lq。
  homeMode: 'minimal', searchEngine: 'bing', showRecent: true, startupUrl: '', theme: 'obsidian', colorMode: 'auto', restoreSession: true,
};

// 功能：展示持久化下载状态和原生引擎实际接收大小；时间：2026-11-14；作者：lq。
export interface DownloadItem {
  id: string; url: string; tabId: string; sourceUrl: string; name: string; path: string;
  status: 'downloading' | 'completed' | 'failed' | 'interrupted'; receivedBytes: number;
  startedAt: number; finishedAt?: number; error: string;
}

// 功能：会话只保存导航信息与缩放，不保存网页正文、表单和运行状态；时间：2026-11-14；作者：lq。
export interface SessionTab { id: string; url: string; title: string; zoom: number }
export interface SessionSnapshot { version: 1; tabs: SessionTab[]; activeId: string; closed: SessionTab[] }
