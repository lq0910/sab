// 功能：区分地址和搜索词，保留协议、查询参数和锚点；时间：2026-11-08；作者：lq。
import type { SearchEngine } from './models';

export const engines: Record<SearchEngine, { name: string; prefix: string }> = {
  bing: { name: '必应 Bing', prefix: 'https://www.bing.com/search?q=' },
  baidu: { name: '百度 Baidu', prefix: 'https://www.baidu.com/s?wd=' },
  google: { name: 'Google', prefix: 'https://www.google.com/search?q=' },
  duckduckgo: { name: 'DuckDuckGo', prefix: 'https://duckduckgo.com/?q=' },
};

// 功能：区分需要交给搜索引擎的词语与可直接打开的网址；时间：2026-11-08；作者：lq。
export function isSearchQuery(input: string): boolean {
  const value = input.trim();
  if (!value || ['sab://home', 'sab://bookmarks', 'sab://settings', 'sab://history'].includes(value)) return false;
  if (/^[a-z][a-z\d+.-]*:/i.test(value) && !/^[\w.-]+:\d+(?:\/|$)/.test(value)) return false;
  return !(!/\s/.test(value) && /^(localhost|\[[\da-f:]+\]|[\w\p{L}-]+(?:\.[\w\p{L}-]+)+)(:\d+)?([/?#]|$)/iu.test(value));
}

export function resolveAddress(input: string, engine: SearchEngine = 'bing'): string {
  const value = input.trim();
  if (!value) return 'sab://home';
  // 功能：将轻页内置历史页纳入可直接输入的地址；时间：2026-11-08；作者：lq。
  if (['sab://home', 'sab://bookmarks', 'sab://settings', 'sab://history'].includes(value)) return value;
  if (/^[a-z][a-z\d+.-]*:/i.test(value) && !/^[\w.-]+:\d+(?:\/|$)/.test(value)) {
    const url = new URL(value);
    if (!['https:', 'http:'].includes(url.protocol) || url.username || url.password) {
      throw new Error('请输入 HTTP 或 HTTPS 地址。');
    }
    return url.href;
  }
  if (!/\s/.test(value) && /^(localhost|\[[\da-f:]+\]|[\w\p{L}-]+(?:\.[\w\p{L}-]+)+)(:\d+)?([/?#]|$)/iu.test(value)) {
    return new URL(`https://${value}`).href;
  }
  return engines[engine].prefix + encodeURIComponent(value);
}

export function displayHost(url: string): string {
  try { return new URL(url).host; } catch { return url; }
}

export function isInternal(url: string): boolean {
  return url.startsWith('sab://');
}
