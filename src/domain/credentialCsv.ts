// 功能：解析 Chrome 密码 CSV 并仅保留可导入网站账号；时间：2026-11-16；作者：lq。
export interface CredentialDraft { origin: string; username: string; password: string }
export interface CredentialCsvResult { credentials: CredentialDraft[]; skipped: number }

// 功能：读取带引号、换行和逗号的 Chrome CSV 字段；时间：2026-11-16；作者：lq。
function parseRows(content: string): string[][] {
  const rows: string[][] = [];
  let row: string[] = [];
  let field = '';
  let quoted = false;
  let closedQuote = false;
  for (let i = 0; i < content.length; i++) {
    const char = content[i]!;
    if (quoted) {
      if (char === '"' && content[i + 1] === '"') { field += '"'; i++; }
      else if (char === '"') { quoted = false; closedQuote = true; }
      else field += char;
    } else if (char === '"' && field === '') quoted = true;
    else if (char === ',') { row.push(field); field = ''; closedQuote = false; }
    else if (char === '\n' || char === '\r') {
      if (char === '\r' && content[i + 1] === '\n') i++;
      row.push(field); field = '';
      if (row.some(value => value !== '')) rows.push(row);
      row = [];
      closedQuote = false;
    } else {
      if (closedQuote || char === '"') throw new Error('CSV 字段格式不正确，请重新从 Chrome 导出。');
      field += char;
    }
    if (rows.length > 10000) throw new Error('CSV 条目过多，请拆分文件后导入。');
  }
  if (quoted) throw new Error('CSV 文件包含未闭合的引号。');
  if (field || row.length) { row.push(field); if (row.some(value => value !== '')) rows.push(row); }
  return rows;
}

// 功能：按 Chrome 列名提取 HTTP(S) 账号，去重并跳过空密码项目；时间：2026-11-16；作者：lq。
export function parseChromePasswordsCsv(content: string): CredentialCsvResult {
  if (content.length > 20 * 1024 * 1024) throw new Error('CSV 文件超过 20 MB。');
  const rows = parseRows(content.replace(/^\uFEFF/, ''));
  if (!rows.length) throw new Error('CSV 文件为空。');
  const headers = rows[0]!.map(value => value.trim().toLocaleLowerCase());
  const urlIndex = headers.indexOf('url');
  const usernameIndex = headers.indexOf('username');
  const passwordIndex = headers.indexOf('password');
  if (['url', 'username', 'password'].some(name => headers.filter(value => value === name).length !== 1)) {
    throw new Error('请使用 Chrome 导出的密码 CSV（需包含 url、username、password 列）。');
  }
  const credentials: CredentialDraft[] = [];
  const seen = new Set<string>();
  let skipped = 0;
  for (const row of rows.slice(1)) {
    try {
      const url = new URL(row[urlIndex] ?? '');
      const username = row[usernameIndex] ?? '';
      const password = row[passwordIndex] ?? '';
      if (row.length !== headers.length || !['http:', 'https:'].includes(url.protocol) || !url.hostname || url.username || url.password || url.hostname === 'tauri.localhost' || !password || username.length > 1024 || password.length > 4096) { skipped++; continue; }
      const origin = url.origin;
      const key = `${origin}\n${username}`;
      if (seen.has(key)) { skipped++; continue; }
      seen.add(key);
      credentials.push({ origin, username, password });
    } catch { skipped++; }
  }
  return { credentials, skipped };
}
