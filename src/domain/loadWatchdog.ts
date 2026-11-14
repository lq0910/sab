// 功能：在缺少平台网络错误回调时提供可取消的加载超时提示，不自动重放网页请求；时间：2026-11-14；作者：lq。
export function loadWatchdog(onTimeout: (id: string) => void, delay = 30_000) {
  const timers = new Map<string, ReturnType<typeof setTimeout>>();
  const cancel = (id: string) => { clearTimeout(timers.get(id)); timers.delete(id); };
  const start = (id: string) => {
    cancel(id);
    timers.set(id, setTimeout(() => { timers.delete(id); onTimeout(id); }, delay));
  };
  const clear = () => { timers.forEach(clearTimeout); timers.clear(); };
  return { start, cancel, clear };
}
