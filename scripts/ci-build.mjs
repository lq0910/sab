// 功能：按Tag校验版本、构建各架构安装包、核验应用图标并生成无冲突发布资产及SHA；时间：2026-11-14；作者：lq。
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { readFileSync, writeFileSync, existsSync, mkdirSync, copyFileSync, readdirSync, statSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('../', import.meta.url));
const target = process.argv[2];
const targets = {
  'aarch64-apple-darwin': ['macos', 'arm64'],
  'x86_64-apple-darwin': ['macos', 'x64'],
  'universal-apple-darwin': ['macos', 'universal'],
  'x86_64-pc-windows-msvc': ['windows', 'x64'],
  'aarch64-pc-windows-msvc': ['windows', 'arm64'],
  'i686-pc-windows-msvc': ['windows', 'x86'],
};
if (!targets[target]) throw new Error('不支持的构建架构');
const [platform, architecture] = targets[target];
const manifest = JSON.parse(readFileSync(join(root, 'package.json')));
const config = JSON.parse(readFileSync(join(root, 'src-tauri/tauri.conf.json')));
const cargoVersion = readFileSync(join(root, 'src-tauri/Cargo.toml'), 'utf8').match(/^version = "([^"]+)"/m)?.[1];
const tag = process.env.RELEASE_TAG || (process.env.GITHUB_REF_TYPE === 'tag' ? process.env.GITHUB_REF_NAME : `v${manifest.version}`);
if (!/^v\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/.test(tag || '') || tag !== `v${manifest.version}` || config.version !== manifest.version || cargoVersion !== manifest.version || config.bundle.macOS.bundleVersion !== manifest.buildNumber) throw new Error('Tag、package.json、Cargo.toml、Tauri版本或构建号不一致');
if ((platform === 'macos' && process.platform !== 'darwin') || (platform === 'windows' && process.platform !== 'win32')) throw new Error('请使用对应平台的原生Runner');
const run = (command, args, capture = false) => {
  // 功能：Windows的pnpm.cmd由系统shell启动；所有命令参数均来自固定命令和上方白名单；时间：2026-11-14；作者：lq。
  const result = spawnSync(command, args, { cwd: root, stdio: capture ? 'pipe' : 'inherit', encoding: 'utf8', shell: process.platform === 'win32' && command === 'pnpm' });
  if (result.error || result.status !== 0) throw new Error(`${command}失败：${result.error?.message || result.stderr || result.status}`);
  return result.stdout?.trim();
};
const options = ['tauri', 'build', '--target', target, '--bundles', platform === 'macos' ? 'app,dmg' : 'nsis', '--ci'];
const signedMac = platform === 'macos' && !!process.env.APPLE_SIGNING_IDENTITY;
if (platform === 'macos' && !signedMac) options.push('--no-sign');
run('pnpm', [...options, '--', '--locked']);
const bundle = resolve(root, process.env.CARGO_TARGET_DIR || 'src-tauri/target', target, 'release/bundle');
const output = join(root, 'output/ci', `${platform}-${architecture}`);
mkdirSync(output, { recursive: true });
const prefix = `SAB_${manifest.version}_${platform}_${architecture}`;
const artifacts = [];
function copyAsset(path, name) { copyFileSync(path, join(output, name)); artifacts.push(name); }
if (platform === 'macos') {
  const app = join(bundle, 'macos', `${config.productName}.app`);
  const plist = join(app, 'Contents/Info.plist');
  const icon = run('plutil', ['-extract', 'CFBundleIconFile', 'raw', '-o', '-', plist], true);
  const iconPath = join(app, 'Contents/Resources', icon.endsWith('.icns') ? icon : `${icon}.icns`);
  if (!existsSync(iconPath) || statSync(iconPath).size < 1000) throw new Error('应用图标未正确写入包，拒绝发布');
  const executable = join(app, 'Contents/MacOS/sab');
  const archs = run('lipo', ['-archs', executable], true);
  for (const required of architecture === 'universal' ? ['arm64', 'x86_64'] : [architecture === 'arm64' ? 'arm64' : 'x86_64']) if (!archs.split(' ').includes(required)) throw new Error('应用架构不完整');
  const dmg = readdirSync(join(bundle, 'dmg')).filter(name => name.endsWith('.dmg'));
  if (dmg.length !== 1) throw new Error('DMG产物数量不符');
  run('hdiutil', ['verify', join(bundle, 'dmg', dmg[0])]);
  copyAsset(join(bundle, 'dmg', dmg[0]), `${prefix}.dmg`);
  const zip = `${prefix}.app.zip`;
  run('ditto', ['-c', '-k', '--sequesterRsrc', '--keepParent', app, join(output, zip)]);
  run('unzip', ['-tq', join(output, zip)]);
  if (signedMac) { run('codesign', ['--verify', '--deep', '--strict', app]); run('xcrun', ['stapler', 'validate', app]); }
  artifacts.push(zip);
} else {
  const installers = readdirSync(join(bundle, 'nsis')).filter(name => name.endsWith('.exe'));
  if (installers.length !== 1) throw new Error('NSIS产物数量不符');
  copyAsset(join(bundle, 'nsis', installers[0]), `${prefix}_setup.exe`);
}
const hashes = artifacts.map(file => ({ file, bytes: statSync(join(output, file)).size, sha256: createHash('sha256').update(readFileSync(join(output, file))).digest('hex') }));
writeFileSync(join(output, `${prefix}_SHA256SUMS.txt`), hashes.map(item => `${item.sha256}  ${item.file}`).join('\n') + '\n');
writeFileSync(join(output, `${prefix}_build.json`), JSON.stringify({ version: manifest.version, build: manifest.buildNumber, tag, commit: process.env.GITHUB_SHA || null, platform, architecture, developerIdSigned: signedMac, notarized: signedMac, windowsSigned: false, artifacts: hashes }, null, 2) + '\n');
console.log(`发布资产就绪：${output}`);
