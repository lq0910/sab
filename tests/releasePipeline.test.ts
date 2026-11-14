// 功能：验证发布前的Tag/矩阵/哈希保护，并用本地假gh验收双平台并发创建Release的恢复；时间：2026-11-14；作者：lq。
import { afterEach, describe, expect, it } from 'vitest';
import { mkdtempSync, writeFileSync, readFileSync, rmSync, existsSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { tmpdir } from 'node:os';
import { createHash } from 'node:crypto';
import { spawnSync } from 'node:child_process';
const directories: string[] = [];
afterEach(() => directories.splice(0).forEach(path => rmSync(path, { recursive: true, force: true })));
function fixture(architectures = ['arm64', 'x64', 'x86']) {
  const directory = mkdtempSync(join(tmpdir(), 'sab-publish-test-')); directories.push(directory);
  for (const architecture of architectures) {
    const file = `${architecture}.exe`;
    const content = Buffer.from(`fixture-${architecture}`);
    writeFileSync(join(directory, file), content);
    writeFileSync(join(directory, `${architecture}_build.json`), JSON.stringify({ platform: 'windows', architecture, tag: 'v0.1.0', commit: 'fixture-commit', artifacts: [{ file, bytes: content.length, sha256: createHash('sha256').update(content).digest('hex') }] }));
  }
  return directory;
}
function publish(directory: string, extra = {}) {
  return spawnSync(process.execPath, ['scripts/publish-release.mjs', directory], { cwd: resolve('.'), encoding: 'utf8', env: { ...process.env, RELEASE_TAG: 'v0.1.0', GITHUB_REPOSITORY: 'fixture/sab', GITHUB_SHA: 'fixture-commit', ...extra } });
}
// 功能：模拟Tag查询、并发Release创建冲突和上传，禁止测试访问网络；时间：2026-11-14；作者：lq。
function fakeGh(directory: string) {
  const log = join(directory, 'gh-log'); const marker = join(directory, 'created');
  writeFileSync(join(directory, 'gh'), `#!${process.execPath}\nconst fs=require('node:fs');const args=process.argv.slice(2);fs.appendFileSync(process.env.SAB_GH_LOG,JSON.stringify(args)+'\\n');if(args[0]==='api'){if(process.env.SAB_API_404&&!fs.existsSync(process.env.SAB_GH_MARK)){console.error('HTTP 404');process.exit(1);}console.log(process.env.SAB_TAG_COMMIT||'fixture-commit');process.exit(0);}if(args[1]==='create'){fs.writeFileSync(process.env.SAB_GH_MARK,'created by other workflow');process.exit(1);}if(args[1]==='view')process.exit(fs.existsSync(process.env.SAB_GH_MARK)?0:1);process.exit(0);\n`, { mode: 0o755 });
  return { log, marker, env: { PATH: directory, SAB_GH_LOG: log, SAB_GH_MARK: marker } };
}
describe('发布流水线保护', () => {
  it('版本不匹配时停止，不能构建错误Tag', () => {
    const result = spawnSync(process.execPath, ['scripts/ci-build.mjs', 'aarch64-apple-darwin'], { encoding: 'utf8', env: { ...process.env, RELEASE_TAG: 'v999.0.0' } });
    expect(result.status).not.toBe(0);
    expect(result.stderr).toContain('不一致');
  });
  it('缺少架构或混合平台时停止，禁止发布半个平台', () => {
    expect(publish(fixture(['x64'])).stderr).toContain('三个架构');
    const directory = fixture();
    const path = join(directory, 'x64_build.json');
    const data = JSON.parse(readFileSync(path, 'utf8')); data.platform = 'macos'; writeFileSync(path, JSON.stringify(data));
    expect(publish(directory).stderr).toContain('平台或架构矩阵不完整');
  });
  it('文件被替换或commit不符时停止发布', () => {
    const directory = fixture(); writeFileSync(join(directory, 'x64.exe'), 'tampered');
    expect(publish(directory).stderr).toContain('校验失败');
    expect(publish(fixture(), { GITHUB_SHA: 'different' }).stderr).toContain('commit');
  });
  // 功能：替身命令只写临时日志，不访问网络或真实GitHub；Windows构建仍执行上方纯逻辑保护测试；时间：2026-11-14；作者：lq。
  it.skipIf(process.platform === 'win32')('另一平台已并发创建Release时继续上传独立资产', () => {
    const directory = fixture(); const { log, marker, env } = fakeGh(directory);
    const result = publish(directory, env);
    expect(result.status, result.stderr).toBe(0);
    expect(existsSync(marker)).toBe(true);
    const calls = readFileSync(log, 'utf8').trim().split('\n').map(line => JSON.parse(line) as string[]);
    expect(calls.map(args => args[0] === 'api' ? 'api' : args[1])).toEqual(['api', 'view', 'create', 'view', 'api', 'upload']);
    expect(calls.at(-1)!.some(value => value.endsWith('arm64.exe'))).toBe(true);
    expect(calls.at(-1)!.includes('--clobber')).toBe(true);
  });
  it.skipIf(process.platform === 'win32')('手动分支构建完成后创建指向本次提交的Tag', () => {
    const directory = fixture(); const { log, env } = fakeGh(directory);
    const result = publish(directory, { ...env, GITHUB_EVENT_NAME: 'workflow_dispatch', SAB_API_404: '1' });
    expect(result.status, result.stderr).toBe(0);
    const calls = readFileSync(log, 'utf8').trim().split('\n').map(line => JSON.parse(line) as string[]);
    const create = calls.find(args => args[1] === 'create')!;
    expect(create.slice(create.indexOf('--target'), create.indexOf('--target') + 2)).toEqual(['--target', 'fixture-commit']);
    expect(create.includes('--verify-tag')).toBe(false);
  });
  it.skipIf(process.platform === 'win32')('已有Tag属于其他提交时不能覆盖Release', () => {
    const directory = fixture(); const { log, env } = fakeGh(directory);
    const result = publish(directory, { ...env, SAB_TAG_COMMIT: 'other-commit' });
    expect(result.stderr).toContain('同名Tag指向其他提交');
    expect(readFileSync(log, 'utf8')).not.toContain('upload');
  });
});
