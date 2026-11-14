// 功能：验证真实事件乱序、加载标题变化和休眠后的迟到事件；时间：2026-11-14；作者：lq。
import { describe, expect, it } from 'vitest';
import { applyPageEvent } from '../src/domain/pageLifecycle';
import type { Tab } from '../src/domain/models';

describe('网页生命周期', () => {
  // 功能：超时后只接受真实完成事件恢复，标题变化不能掩盖加载问题；时间：2026-11-14；作者：lq。
  it('标题事件保留超时提示，加载完成后清除提示', () => {
    const tab: Tab = { id: 'one', url: 'https://example.com/', title: 'Example', loading: true, error: '等待超时', loadIssue: 'timeout' };
    applyPageEvent(tab, { tabId: 'one', url: tab.url, title: '新标题' });
    expect(tab.error).toBe('等待超时');
    expect(tab.loadIssue).toBe('timeout');
    applyPageEvent(tab, { tabId: 'one', url: tab.url, loading: false });
    expect(tab.loading).toBe(false);
    expect(tab.error).toBe('');
    expect(tab.loadIssue).toBeUndefined();
  });
  it('标题提前到达时继续加载，完成后允许记录最终标题', () => {
    const tab: Tab = { id: 'one', url: 'https://example.com/', title: 'Example', loading: true };
    expect(applyPageEvent(tab, { tabId: 'one', url: tab.url, title: '最终标题' })).toBe(false);
    expect(tab.loading).toBe(true);
    expect(applyPageEvent(tab, { tabId: 'one', url: tab.url, loading: false })).toBe(true);
    expect(tab.title).toBe('最终标题');
    expect(applyPageEvent(tab, { tabId: 'one', url: tab.url, title: '最终标题' })).toBe(false);
  });
  it('忽略其他标签、已休眠标签及内置页面的旧事件', () => {
    for (const state of [{ url: 'sab://home' }, { url: 'https://example.com/', suspended: true }]) {
      const tab: Tab = { id: 'one', title: '保留', loading: false, ...state };
      expect(applyPageEvent(tab, { tabId: 'one', url: 'https://old.example/', loading: false })).toBe(false);
      expect(tab.title).toBe('保留');
      expect(tab.url).toBe(state.url);
    }
    const tab: Tab = { id: 'one', url: 'https://example.com/', title: '保留', loading: false };
    expect(applyPageEvent(tab, { tabId: 'two', url: 'https://old.example/', title: '错误标签' })).toBe(false);
    expect(tab.title).toBe('保留');
  });
});
