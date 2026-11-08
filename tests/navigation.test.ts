// 功能：验证不可信地址与保守 URL 处理边界；时间：2026-11-08；作者：lq。
import { describe, expect, it } from 'vitest';
import { resolveAddress } from '../src/domain/navigation';

describe('地址与搜索', () => {
  it('默认必应且正确编码中文', () => {
    expect(resolveAddress('轻页 浏览器')).toBe('https://www.bing.com/search?q=%E8%BD%BB%E9%A1%B5%20%E6%B5%8F%E8%A7%88%E5%99%A8');
  });
  it('不合并协议，不剥离查询参数和锚点', () => {
    expect(resolveAddress('http://example.com/?utm_source=sab#part')).toBe('http://example.com/?utm_source=sab#part');
    expect(resolveAddress('example.com/path?q=1#x')).toBe('https://example.com/path?q=1#x');
  });
  it('支持本地端口、IP、中文域名和 IPv6', () => {
    expect(resolveAddress('localhost:8080/path')).toBe('https://localhost:8080/path');
    expect(resolveAddress('127.0.0.1:9000')).toBe('https://127.0.0.1:9000/');
    expect(resolveAddress('[::1]:8080/path')).toBe('https://[::1]:8080/path');
    expect(resolveAddress('例子.中国')).toContain('xn--');
  });
  it.each(['javascript:alert(1)', 'file:///etc/passwd', 'data:text/html,test', 'https://user:secret@example.com'])('拒绝危险或携带凭据的地址 %s', (value) => {
    expect(() => resolveAddress(value)).toThrow();
  });
  it('仅识别受支持的内部页', () => {
    expect(resolveAddress('sab://bookmarks')).toBe('sab://bookmarks');
    expect(() => resolveAddress('sab://unknown')).toThrow();
  });
});
