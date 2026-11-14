// 功能：验证恢复不携带临时网页状态、限制快照大小和串行保存的一致性；时间：2026-11-14；作者：lq。
import { describe, expect, it, vi } from 'vitest';
import type { SessionSnapshot, Tab } from '../src/domain/models';
import { captureSession, restoreSession, sessionWriter } from '../src/domain/session';

const tab = (id: string, url = 'https://example.com/'): Tab => ({ id, url, title: '页面', loading: true, error: '旧错误', loadIssue: 'timeout', zoom: 1.2 });
describe('会话恢复', () => {
  it('只保存导航信息，后台网页等待选择后加载', () => {
    const snapshot = captureSession([tab('web'), tab('home', 'sab://home')], 'web', [tab('closed')]);
    expect(snapshot.tabs[0]).toEqual({ id: 'web', url: 'https://example.com/', title: '页面', zoom: 1.2 });
    const restored = restoreSession(snapshot);
    expect(restored.activeId).toBe('web');
    expect(restored.tabs[0]).toMatchObject({ loading: false, suspended: true, zoom: 1.2 });
    expect(restored.tabs[1]).toMatchObject({ loading: false, suspended: false });
    expect(restored.tabs[0]).not.toHaveProperty('error');
    expect(restored.closed[0]).not.toHaveProperty('loadIssue');
  });
  it('限制开放标签和关闭历史时保留当前页并按 Unicode 字符截断标题', () => {
    const tabs = Array.from({ length: 205 }, (_, index) => tab(String(index)));
    tabs[204]!.title = '😀'.repeat(600);
    const snapshot = captureSession(tabs, '204', tabs.slice(0, 25));
    expect(snapshot.tabs).toHaveLength(200);
    expect(snapshot.tabs.at(-1)!.id).toBe('204');
    expect(Array.from(snapshot.tabs.at(-1)!.title)).toHaveLength(512);
    expect(snapshot.closed).toHaveLength(20);
    expect(snapshot.closed[0]!.id).toBe('5');
    expect(tabs[204]!.title).toBe('😀'.repeat(600));
  });
  it('慢的旧保存必须完成才开始新保存，失败后仍能保存后续会话', async () => {
    let finish!: () => void;
    const save = vi.fn<(snapshot: SessionSnapshot) => Promise<void>>()
      .mockImplementationOnce(() => new Promise(resolve => { finish = resolve; }))
      .mockRejectedValueOnce(new Error('磁盘忙'))
      .mockResolvedValueOnce(undefined);
    const write = sessionWriter(save);
    const first = captureSession([tab('one')], 'one', []);
    const second = captureSession([tab('two')], 'two', []);
    const pending = write(first);
    const failed = write(second);
    const rejection = expect(failed).rejects.toThrow('磁盘忙');
    const last = write(first);
    await Promise.resolve();
    expect(save).toHaveBeenCalledTimes(1);
    finish();
    await pending; await rejection; await last;
    expect(save.mock.calls.map(([value]) => value.activeId)).toEqual(['one', 'two', 'one']);
  });
});
