// 功能：验证输入合并及关闭搜索后不再触发过期查询；时间：2026-11-14；作者：lq。
import { afterEach, expect, it, vi } from 'vitest';
import { debounceTask } from '../src/domain/tasks';
afterEach(() => vi.useRealTimers());
it('连续输入只执行最后一次，关闭后取消等待中的请求', () => {
  vi.useFakeTimers();
  const query = vi.fn();
  const task = debounceTask(query);
  task.schedule();
  vi.advanceTimersByTime(100);
  task.schedule();
  vi.advanceTimersByTime(100);
  task.schedule();
  vi.advanceTimersByTime(180);
  expect(query).toHaveBeenCalledTimes(1);
  task.schedule();
  task.cancel();
  vi.runAllTimers();
  expect(query).toHaveBeenCalledTimes(1);
});
