// 功能：验证超时只提示当前请求，完成、关闭和卸载后不留下过期计时器；时间：2026-11-14；作者：lq。
import { afterEach, expect, it, vi } from 'vitest';
import { loadWatchdog } from '../src/domain/loadWatchdog';
afterEach(() => vi.useRealTimers());
it('重新导航重新计时，已完成或关闭标签不会报超时', () => {
  vi.useFakeTimers();
  const timeout = vi.fn();
  const watchdog = loadWatchdog(timeout);
  watchdog.start('active'); watchdog.start('finished'); watchdog.start('closed');
  vi.advanceTimersByTime(20_000);
  watchdog.start('active'); watchdog.cancel('finished'); watchdog.cancel('closed');
  vi.advanceTimersByTime(29_999);
  expect(timeout).not.toHaveBeenCalled();
  vi.advanceTimersByTime(1);
  expect(timeout).toHaveBeenCalledExactlyOnceWith('active');
  // 功能：继续等待可以再次计时，卸载可释放所有剩余计时器；时间：2026-11-14；作者：lq。
  watchdog.start('active'); watchdog.start('other'); watchdog.clear();
  vi.runAllTimers();
  expect(timeout).toHaveBeenCalledTimes(1);
});
