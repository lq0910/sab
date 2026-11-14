// 功能：构造只含导航信息的会话快照，并将后台网页恢复为按需加载状态；时间：2026-11-14；作者：lq。
import type { SessionSnapshot, SessionTab, Tab } from './models';
import { isInternal } from './navigation';

function navigation(tab: Tab): SessionTab {
  return { id: tab.id, url: tab.url, title: Array.from(tab.title).slice(0, 512).join(''), zoom: tab.zoom ?? 1 };
}
export function captureSession(tabs: Tab[], activeId: string, closed: Tab[]): SessionSnapshot {
  const kept = tabs.slice(0, 200);
  const active = tabs.find(tab => tab.id === activeId);
  if (active && !kept.includes(active)) kept[kept.length - 1] = active;
  return { version: 1, tabs: kept.map(navigation), activeId, closed: closed.slice(-20).map(navigation) };
}
export function restoreSession(snapshot: SessionSnapshot): { tabs: Tab[]; activeId: string; closed: Tab[] } {
  return {
    tabs: snapshot.tabs.map(tab => ({ ...tab, loading: false, suspended: !isInternal(tab.url) })),
    activeId: snapshot.activeId,
    closed: snapshot.closed.map(tab => ({ ...tab, loading: false })),
  };
}
// 功能：串行提交会话，保证较慢的旧请求不能覆盖较新的快照；时间：2026-11-14；作者：lq。
export function sessionWriter(save: (snapshot: SessionSnapshot) => Promise<void>) {
  let queue = Promise.resolve();
  return (snapshot: SessionSnapshot) => {
    const result = queue.then(() => save(snapshot));
    queue = result.catch(() => {});
    return result;
  };
}
