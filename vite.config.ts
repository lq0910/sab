// 功能：配置 Vue 构建、固定开发端口与核心逻辑测试；时间：2026-11-08；作者：lq。
import { defineConfig } from 'vitest/config';
import vue from '@vitejs/plugin-vue';

export default defineConfig({
  plugins: [vue()],
  clearScreen: false,
  // 功能：避开占用的 Vite 端口，并与 Tauri 开发地址保持一致；时间：2026-11-08；作者：lq。
  server: { host: '127.0.0.1', port: 1421, strictPort: true },
  build: { target: ['es2022', 'safari15'] },
  test: { include: ['tests/**/*.test.ts'], environment: 'node' },
});
