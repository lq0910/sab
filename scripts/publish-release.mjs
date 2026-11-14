// 功能：聚合某平台所有成功架构产物并自动发布到已有Tag，两个平台并发创建Release可安全重试；时间：2026-11-14；作者：lq。
import { spawnSync } from 'node:child_process';
import { readdirSync, readFileSync, writeFileSync, statSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { join } from 'node:path';
// 功能：手动分支构建使用工程版本，所有架构成功后才允许创建Tag；时间：2026-11-14；作者：lq。
const manifest = JSON.parse(readFileSync(new URL('../package.json', import.meta.url)));
const tag = process.env.RELEASE_TAG || (process.env.GITHUB_REF_TYPE === 'tag' ? process.env.GITHUB_REF_NAME : `v${manifest.version}`);
const repository = process.env.GITHUB_REPOSITORY;
const directory = process.argv[2] || 'release-assets';
if (!/^v\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/.test(tag || '') || !/^[\w.-]+\/[\w.-]+$/.test(repository || '')) throw new Error('无效的Tag或仓库');
const run = args => spawnSync('gh', args, { encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] });
const files = readdirSync(directory).filter(file => !file.startsWith('.') && statSync(join(directory, file)).isFile());
const builds = files.filter(file => file.endsWith('_build.json')).map(file => JSON.parse(readFileSync(join(directory, file))));
if (builds.length !== 3) throw new Error('必须完成当前平台的三个架构构建后才能发布');
// 功能：禁止重复或混合架构冒充完整平台矩阵，发布前再次核对文件大小和哈希；时间：2026-11-14；作者：lq。
const platform = builds[0].platform;
const expected = { macos: ['arm64', 'universal', 'x64'], windows: ['arm64', 'x64', 'x86'] }[platform];
if (!expected || builds.some(build => build.platform !== platform) || builds.map(build => build.architecture).sort().join(',') !== expected.sort().join(',')) throw new Error('平台或架构矩阵不完整');
for (const build of builds) {
  if (build.tag !== tag || build.commit !== process.env.GITHUB_SHA) throw new Error('构建Tag或commit与发布任务不符');
  for (const artifact of build.artifacts) {
    if (!/^[\w.-]+$/.test(artifact.file)) throw new Error('非法发布资产名称');
    const content = readFileSync(join(directory, artifact.file));
    if (content.length !== artifact.bytes || createHash('sha256').update(content).digest('hex') !== artifact.sha256) throw new Error('发布资产校验失败');
  }
}
// 功能：已有Tag必须指向本次提交，避免手动重跑覆盖其他提交的发行资产；时间：2026-11-14；作者：lq。
const existingTag = run(['api', `repos/${repository}/commits/${encodeURIComponent(tag)}`, '--jq', '.sha']);
if (existingTag.status === 0 && existingTag.stdout.trim() !== process.env.GITHUB_SHA) throw new Error('同名Tag指向其他提交，拒绝发布');
if (existingTag.status !== 0 && !existingTag.stderr.includes('404')) throw new Error(existingTag.stderr || '无法核对Tag');
if (existingTag.status !== 0 && process.env.GITHUB_EVENT_NAME !== 'workflow_dispatch') throw new Error('Tag不存在，只有手动构建允许自动创建');
const notes = join(directory, '.release-notes.md');
writeFileSync(notes, `轻页 SAB ${tag}\n\nmacOS 12+：Apple Silicon / Intel / Universal。Windows 10/11：x64 / x86 / ARM64（ARM64要求对应系统）。不同平台的资产会在各自构建完成后追加。\n\n包含浏览、收藏、下载管理、会话恢复和默认浏览器设置。每个架构均附SHA-256与构建元数据。默认浏览器需用户在系统中确认。\n\n下载前查看对应_build.json：developerIdSigned / notarized / windowsSigned 字段如实标注签名状态。未配置证书的构建未经过发行者签名，系统可能显示验证提示。\n`);
if (run(['release', 'view', tag, '--repo', repository]).status !== 0) {
  const args = ['release', 'create', tag, '--repo', repository, ...(existingTag.status === 0 ? ['--verify-tag'] : ['--target', process.env.GITHUB_SHA]), '--title', `轻页 SAB ${tag}`, '--notes-file', notes];
  if (tag.includes('-')) args.push('--prerelease');
  const result = run(args);
  if (result.status !== 0 && run(['release', 'view', tag, '--repo', repository]).status !== 0) throw new Error(result.stderr || '创建Release失败');
}
// 功能：创建Release后再次核对Tag，阻止两个不同提交并发创建同名版本后混入错误产物；时间：2026-11-14；作者：lq。
const confirmedTag = run(['api', `repos/${repository}/commits/${encodeURIComponent(tag)}`, '--jq', '.sha']);
if (confirmedTag.status !== 0 || confirmedTag.stdout.trim() !== process.env.GITHUB_SHA) throw new Error('发布前Tag提交复核失败');
const upload = run(['release', 'upload', tag, '--repo', repository, ...files.map(file => join(directory, file)), '--clobber']);
if (upload.status !== 0) throw new Error(upload.stderr || '发布资产失败');
console.log(`已发布 ${builds[0].platform} 到 ${repository} ${tag}`);
