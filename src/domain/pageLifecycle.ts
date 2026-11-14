// 功能：分离标题与加载事件，避免动态标题提前结束加载并重复写入历史；时间：2026-11-14；作者：lq。
import type { PageEvent, Tab } from './models';
import { isInternal } from './navigation';

export function applyPageEvent(tab: Tab, event: PageEvent): boolean {
  if (tab.id !== event.tabId || isInternal(tab.url) || tab.suspended) return false;
  // 功能：Blob 下载不改变地址栏和历史记录中的来源网页；时间：2026-11-14；作者：lq。
  if (!/^https?:\/\//i.test(event.url)) return false;
  const titleChanged = !!event.title && event.title !== tab.title;
  tab.url = event.url;
  if (event.title) tab.title = event.title;
  if (event.loading !== undefined) tab.loading = event.loading;
  // 功能：加载最终完成时清除超时/离线提示，不把标题变化当成恢复成功；时间：2026-11-14；作者：lq。
  if (event.loading === false) { tab.error = ''; tab.loadIssue = undefined; }
  return event.loading === false || (titleChanged && !tab.loading);
}
