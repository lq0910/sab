// 功能：合并连续输入任务，并允许关闭页面或切换筛选时取消；时间：2026-11-14；作者：lq。
export function debounceTask(callback: () => void, delay = 180) {
  let timer: ReturnType<typeof setTimeout> | undefined;
  const cancel = () => { clearTimeout(timer); timer = undefined; };
  const schedule = () => {
    cancel();
    timer = setTimeout(() => { timer = undefined; callback(); }, delay);
  };
  return { schedule, cancel };
}
