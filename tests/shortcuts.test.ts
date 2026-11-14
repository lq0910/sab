// 功能：验证网页预览的快捷键与系统菜单动作一致并避开输入法和 Alt 组合；时间：2026-11-14；作者：lq。
import { expect, it } from 'vitest';
import { keyboardAction } from '../src/domain/shortcuts';
const key = (value: string, modifiers = {}) => ({ key: value, ctrlKey: false, metaKey: true, shiftKey: false, altKey: false, isComposing: false, ...modifiers });
it('常用快捷键与标签切换使用同一动作名称', () => {
  expect(keyboardAction(key('L'))).toBe('focus-address');
  expect(keyboardAction(key('t', { ctrlKey: true, metaKey: false }))).toBe('new-tab');
  expect(keyboardAction(key('T', { shiftKey: true }))).toBe('restore-tab');
  expect(keyboardAction(key('Tab', { metaKey: false, ctrlKey: true }))).toBe('next-tab');
  expect(keyboardAction(key('Tab', { metaKey: false, ctrlKey: true, shiftKey: true }))).toBe('previous-tab');
  expect(keyboardAction(key('J', { shiftKey: true }))).toBe('downloads');
  expect(keyboardAction(key('='))).toBe('zoom-in');
});
it('普通输入、组合输入和 Alt 组合键不触发浏览器命令', () => {
  expect(keyboardAction(key('t', { metaKey: false }))).toBeUndefined();
  expect(keyboardAction(key('t', { isComposing: true }))).toBeUndefined();
  expect(keyboardAction(key('t', { altKey: true }))).toBeUndefined();
  expect(keyboardAction(key('x'))).toBeUndefined();
});
