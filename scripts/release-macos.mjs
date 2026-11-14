// 功能：核对版本、执行检查并归档首个macOS预览版安装包、校验和和源码指纹；时间：2026-11-14；作者：lq。
// 仅提供未签名预览包流程：pnpm release:mac --unsigned；正式分发须另行配置Developer ID并公证。
import { createHash } from 'node:crypto';
import { spawnSync } from 'node:child_process';
import { existsSync, mkdirSync, readFileSync, copyFileSync, writeFileSync, statSync, mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const args = process.argv.slice(2);
const refreshMetadata = args.includes('--refresh-metadata');
if (process.platform !== 'darwin' || !(args.includes('--unsigned') || refreshMetadata) || args.some(arg => !['--unsigned', '--refresh-metadata'].includes(arg)) || (refreshMetadata && args.length !== 1)) {
  throw new Error('请在macOS运行 pnpm release:mac --unsigned。此流程只生成未公证预览包，不适用于正式签名发布。');
}
// 功能：避免工程、原生宿主与安装包显示不同版本或构建号；时间：2026-11-14；作者：lq。
const manifest = JSON.parse(readFileSync(join(root, 'package.json'), 'utf8'));
const config = JSON.parse(readFileSync(join(root, 'src-tauri/tauri.conf.json'), 'utf8'));
const cargo = readFileSync(join(root, 'src-tauri/Cargo.toml'), 'utf8');
if (config.version !== manifest.version || !cargo.includes(`version = "${manifest.version}"`) || config.bundle.macOS.bundleVersion !== manifest.buildNumber) throw new Error('版本或构建号不一致，停止归档');
const output = resolve(process.env.SAB_RELEASE_DIR || join(root, 'output/releases', `${manifest.version}-macos-${process.arch}-preview`));
const hash = path => createHash('sha256').update(readFileSync(path)).digest('hex');
// 功能：后续任意版本使用对应说明；未创建专属记录时使用通用发布文档；时间：2026-11-14；作者：lq。
const versionNotes = join(root, `docs/首个预览版发布记录-${manifest.version}.md`);
const releaseNotes = existsSync(versionNotes) ? versionNotes : join(root, 'docs/GitHub构建发布与默认浏览器-20261114.md');
if (refreshMetadata) {
  const metadataPath = join(output, 'release.json');
  if (!existsSync(metadataPath)) throw new Error(`发布目录或release.json不存在：${output}`);
  const metadata = JSON.parse(readFileSync(metadataPath, 'utf8'));
  for (const artifact of metadata.artifacts) {
    const path = join(output, artifact.file);
    if (!existsSync(path) || hash(path) !== artifact.sha256 || statSync(path).size !== artifact.bytes) throw new Error(`产物与release.json不一致：${artifact.file}`);
  }
  const tracked = spawnSync('git', ['ls-files', '-z', '--cached', '--others', '--exclude-standard'], { cwd: root, encoding: 'utf8' });
  if (tracked.status !== 0) throw new Error('无法构造源码指纹');
  const sources = [...new Set(tracked.stdout.split('\0').filter(Boolean))].sort().filter(file => existsSync(join(root, file)));
  writeFileSync(join(output, 'SOURCE_SHA256SUMS.txt'), sources.map(file => `${hash(join(root, file))}  ${file}`).join('\n') + '\n');
  copyFileSync(releaseNotes, join(output, '发布说明.md'));
  copyFileSync(join(root, 'LICENSE'), join(output, 'LICENSE'));
  metadata.signature = 'adhoc';
  metadata.developerIdSigned = false;
  metadata.notarized = false;
  metadata.sourceFingerprint = 'SOURCE_SHA256SUMS.txt';
  writeFileSync(metadataPath, JSON.stringify(metadata, null, 2) + '\n');
  console.log(`已刷新来源指纹与ad-hoc签名状态：${output}`);
  process.exit(0);
}
if (existsSync(output)) throw new Error(`已有版本目录，禁止覆盖：${output}。请先归档旧产物或更新版本/构建号。`);
const run = (command, values, options = {}) => {
  const result = spawnSync(command, values, { cwd: root, stdio: 'inherit', ...options });
  if (result.error || result.status !== 0) throw new Error(`${command}执行失败：${result.error?.message ?? result.status}`);
  return result;
};
run('pnpm', ['verify']);
run('pnpm', ['tauri', 'build', '--bundles', 'app,dmg', '--no-sign', '--ci', '--', '--locked']);
const target = resolve(root, process.env.CARGO_TARGET_DIR || 'src-tauri/target');
const name = config.productName;
const architecture = process.arch === 'arm64' ? 'aarch64' : process.arch === 'x64' ? 'x64' : process.arch;
const app = join(target, 'release/bundle/macos', `${name}.app`);
const dmg = join(target, 'release/bundle/dmg', `${name}_${manifest.version}_${architecture}.dmg`);
if (!existsSync(app) || !existsSync(dmg)) throw new Error('缺少app或DMG产物，停止归档');
// 功能：图标必须写入.app清单和资源目录，避免再次发布Finder显示占位图标的包；时间：2026-11-14；作者：lq。
const iconRead = spawnSync('plutil', ['-extract', 'CFBundleIconFile', 'raw', '-o', '-', join(app, 'Contents/Info.plist')], { encoding: 'utf8' });
const icon = iconRead.stdout?.trim();
if (iconRead.status !== 0 || !icon || !existsSync(join(app, 'Contents/Resources', icon.endsWith('.icns') ? icon : `${icon}.icns`))) throw new Error('应用包缺少图标，停止发布');
run('hdiutil', ['verify', dmg]);
// 功能：先在独立临时目录完成归档检查，失败时不生成具有成功含义的发布目录；时间：2026-11-14；作者：lq。
const staging = mkdtempSync(join(tmpdir(), 'sab-release-'));
try {
  const zipName = `SAB_${manifest.version}_macos_${process.arch}_preview.zip`;
  const dmgName = `SAB_${manifest.version}_macos_${process.arch}_preview.dmg`;
  run('ditto', ['-c', '-k', '--sequesterRsrc', '--keepParent', app, join(staging, zipName)]);
  run('unzip', ['-tq', join(staging, zipName)]);
  copyFileSync(dmg, join(staging, dmgName));
  copyFileSync(releaseNotes, join(staging, '发布说明.md'));
  copyFileSync(join(root, 'LICENSE'), join(staging, 'LICENSE'));
  const artifacts = [dmgName, zipName].map(file => ({ file, bytes: statSync(join(staging, file)).size, sha256: hash(join(staging, file)) }));
  writeFileSync(join(staging, 'SHA256SUMS.txt'), artifacts.map(item => `${item.sha256}  ${item.file}`).join('\n') + '\n');
  const tracked = spawnSync('git', ['ls-files', '-z', '--cached', '--others', '--exclude-standard'], { cwd: root, encoding: 'utf8' });
  if (tracked.status !== 0) throw new Error('无法构造源码指纹');
  const sources = [...new Set(tracked.stdout.split('\0').filter(Boolean))].sort().filter(file => existsSync(join(root, file)));
  writeFileSync(join(staging, 'SOURCE_SHA256SUMS.txt'), sources.map(file => `${hash(join(root, file))}  ${file}`).join('\n') + '\n');
  const revision = spawnSync('git', ['rev-parse', 'HEAD'], { cwd: root, encoding: 'utf8' }).stdout.trim();
  writeFileSync(join(staging, 'release.json'), JSON.stringify({ version: manifest.version, build: manifest.buildNumber, platform: 'darwin', architecture: process.arch, createdAt: new Date().toISOString(), channel: 'preview', signature: 'adhoc', developerIdSigned: false, notarized: false, gitBase: revision, sourceFingerprint: 'SOURCE_SHA256SUMS.txt', artifacts }, null, 2) + '\n');
  mkdirSync(dirname(output), { recursive: true });
  run('ditto', [staging, output]);
  console.log(`预览版已归档：${output}。仅有ad-hoc本机签名、未公证，公开分发前需要Developer ID签名、公证及设备验收。`);
} finally { rmSync(staging, { recursive: true, force: true }); }
