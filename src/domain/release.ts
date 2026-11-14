// 功能：从工程清单读取首个预览版版本和构建号，避免设置页与安装包版本漂移；时间：2026-11-14；作者：lq。
import manifest from '../../package.json';
export const release = { version: manifest.version, build: manifest.buildNumber, channel: '预览版' } as const;
