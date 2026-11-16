// 功能：验证 Chrome CSV 中原始密码字符保真、危险来源过滤及结构错误；时间：2026-11-16；作者：lq。
import { expect, it } from 'vitest';
import { parseChromePasswordsCsv } from '../src/domain/credentialCsv';

it('保留引号、逗号、换行、前后空格，支持 BOM 和不同列顺序', () => {
  const parsed = parseChromePasswordsCsv('\uFEFFpassword,url,username,name,note\r\n"  a,""b\n尾  ",https://example.com/login," person ",Example,\r\n');
  expect(parsed.credentials).toEqual([{ origin: 'https://example.com', username: ' person ', password: '  a,"b\n尾  ' }]);
  expect(parsed.skipped).toBe(0);
});
it('同来源同用户名只保留第一条，保留空用户名和不同端口', () => {
  const parsed = parseChromePasswordsCsv('name,url,username,password\nOne,https://example.com/a,user,first\nTwo,https://example.com/b,user,second\nEmpty,https://example.com/,,secret\nPort,https://example.com:444/,user,port');
  expect(parsed.credentials.map(account => account.password)).toEqual(['first', 'secret', 'port']);
  expect(parsed.skipped).toBe(1);
});
it('跳过应用凭据、带用户信息的 URL、空密码和字段数错误的条目', () => {
  const parsed = parseChromePasswordsCsv('name,url,username,password\nOne,android://abc,user,secret\nTwo,https://user:pass@example.com/,user,secret\nThree,https://tauri.localhost/,user,secret\nEmpty,https://example.com/,user,\nBroken,https://example.com,user\n');
  expect(parsed.credentials).toEqual([]);
  expect(parsed.skipped).toBe(5);
});
it('拒绝重复必要列、未闭合引号和引号后的多余字符', () => {
  for (const csv of ['url,username,password,password\n', 'url,username,password\nhttps://example.com,user,"unfinished', 'url,username,password\nhttps://example.com,user,"secret"tail']) {
    expect(() => parseChromePasswordsCsv(csv)).toThrow();
  }
});
