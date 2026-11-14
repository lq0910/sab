// 功能：验证下载交互只发送记录 ID，失败可重试、活动任务不被清理；时间：2026-11-14；作者：lq。
import * as Vue from 'vue';
import { createRenderer, h, type Component } from 'vue';
import { compileScript, parse } from 'vue/compiler-sfc';
import { readFileSync } from 'node:fs';
import ts from 'typescript';
import { describe, expect, it, vi } from 'vitest';
// 功能：在 Node 测试中以客户端模式编译组件，避免 SSR 编译丢失点击事件；时间：2026-11-14；作者：lq。
const { descriptor } = parse(readFileSync(new URL('../src/components/DownloadsView.vue', import.meta.url), 'utf8'));
const compiled = compileScript(descriptor, { id: 'download-view-test', inlineTemplate: true });
const javascript = ts.transpileModule(compiled.content, { compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022 } }).outputText;
const moduleExports: { default?: Component } = {};
new Function('require', 'exports', javascript)((name: string) => {
  if (name === 'vue') return Vue;
  if (name === './AppIcon.vue') return { default: { render: () => h('span') } };
  throw new Error(`Unexpected test dependency: ${name}`);
}, moduleExports);
const DownloadsView = moduleExports.default!;
import type { DownloadItem } from '../src/domain/models';

// 功能：用 Vue 自定义渲染器测试真实组件事件，不依赖浏览器或用户桌面；时间：2026-11-14；作者：lq。
interface TestNode { tag: string; text: string; props: Record<string, unknown>; children: TestNode[]; parent?: TestNode }
function node(tag: string, text = ''): TestNode { return { tag, text, props: {}, children: [] }; }
const renderer = createRenderer<TestNode, TestNode>({
  createElement: tag => node(tag), createText: text => node('#text', text), createComment: text => node('#comment', text),
  setText: (target, text) => { target.text = text; }, setElementText: (target, text) => { target.text = text; },
  patchProp: (target, key, _previous, value) => { target.props[key] = value; },
  insert: (child, parent, anchor) => { child.parent = parent; const index = anchor ? parent.children.indexOf(anchor) : -1; if (index < 0) parent.children.push(child); else parent.children.splice(index, 0, child); },
  remove: target => { const siblings = target.parent?.children; if (siblings) siblings.splice(siblings.indexOf(target), 1); },
  parentNode: target => target.parent ?? null, nextSibling: target => { const siblings = target.parent?.children; return siblings?.[siblings.indexOf(target) + 1] ?? null; },
});
function descendants(root: TestNode): TestNode[] { return [root, ...root.children.flatMap(descendants)]; }
function mount(items: DownloadItem[]) {
  const events = { onOpen: vi.fn(), onReveal: vi.fn(), onRetry: vi.fn(), onClear: vi.fn() };
  const root = node('root');
  renderer.createApp({ render: () => h(DownloadsView, { items, ...events }) }).mount(root);
  return { events, root, elements: descendants(root) };
}
function download(status: DownloadItem['status']): DownloadItem {
  return { id: 'download-id', url: 'https://example.com/file', tabId: 'tab', sourceUrl: 'https://example.com/', name: '文档.pdf', path: '/tmp/文档.pdf', status, receivedBytes: 4096, startedAt: Date.now(), error: '' };
}
function click(elements: TestNode[], text: string) {
  const button = elements.find(element => element.tag === 'button' && element.text === text);
  expect(button, text).toBeDefined();
  (button!.props.onClick as () => void)();
}

describe('下载面板交互', () => {
  it('完成文件只按记录 ID 打开或定位', () => {
    const { events, elements } = mount([download('completed')]);
    click(elements, '打开文件'); click(elements, '在文件夹中显示');
    expect(events.onOpen).toHaveBeenCalledWith('download-id');
    expect(events.onReveal).toHaveBeenCalledWith('download-id');
  });
  it('活动下载没有打开按钮或虚假进度，并禁止仅活动任务时清理', () => {
    const { elements } = mount([download('downloading')]);
    expect(elements.some(element => element.tag === 'button' && element.text === '打开文件')).toBe(false);
    const progress = elements.find(element => element.tag === 'progress');
    expect(progress).toBeDefined();
    expect(progress!.props.value).toBeUndefined();
    expect(elements.find(element => element.tag === 'button' && element.text === '清除已结束记录')!.props.disabled).toBe(true);
  });
  it('失败重试保留来源信息，清除只发送清理记录事件', () => {
    const item = download('failed');
    const { events, elements } = mount([item]);
    click(elements, '重新下载'); click(elements, '清除已结束记录');
    expect(events.onRetry).toHaveBeenCalledWith(item);
    expect(events.onClear).toHaveBeenCalledOnce();
  });
});
