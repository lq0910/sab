<script setup lang="ts">
// 功能：提供本地 SVG 图标，替代页面稿中的远程图标字体；时间：2026-11-08；作者：lq。
withDefaults(defineProps<{ name: string; size?: number }>(), { size: 18 });
const paths: Record<string, string> = {
  back: 'm14 6-6 6 6 6M8 12h12', forward: 'm10 6 6 6-6 6M4 12h12',
  refresh: 'M20 7v5h-5M20 12a8 8 0 1 0-2 5',
  search: 'M21 21l-5-5M18 10a8 8 0 1 1-16 0 8 8 0 0 1 16 0',
  plus: 'M12 5v14M5 12h14', close: 'm6 6 12 12M6 18 18 6',
  star: 'm12 3 2.8 5.7 6.2.9-4.5 4.4 1.1 6.2-5.6-3-5.6 3 1.1-6.2L3 9.6l6.2-.9Z',
  bookmarks: 'M6 4h12v17l-6-4-6 4ZM9 1h12v16',
  grid: 'M3 3h7v7H3ZM14 3h7v7h-7ZM3 14h7v7H3ZM14 14h7v7h-7Z',
  list: 'M9 5h12M9 12h12M9 19h12M3 5h1M3 12h1M3 19h1',
  settings: 'm10 3-1 3-3 1-3 3 2 2-1 4 4 1 2 4 4-1 1-3 4-1 2-4-3-2V7l-4-1-1-3ZM15 12a3 3 0 1 1-6 0 3 3 0 0 1 6 0',
  palette: 'M12 3a9 9 0 1 0 0 18h1a2 2 0 0 0 1.7-3.1 1.5 1.5 0 0 1 1.3-2.3H19a2 2 0 0 0 2-2A10.7 10.7 0 0 0 12 3ZM7.5 12h.01M10 7.5h.01M15 7.5h.01M17 12h.01',
  globe: 'M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0ZM3 12h18M12 3c5 5 5 13 0 18-5-5-5-13 0-18',
  // 功能：提供地址栏 HTTPS 加密连接标志；时间：2026-11-08；作者：lq。
  lock: 'M5 10h14v11H5zM8 10V7a4 4 0 1 1 8 0v3M12 14v3',
  home: 'm3 10 9-7 9 7M5 9v12h5v-7h4v7h5V9',
  more: 'M12 4h.01M12 12h.01M12 20h.01',
  chevron: 'm8 10 4 4 4-4', check: 'm5 12 4 4L19 6',
  folder: 'M3 7V4h6l3 3h9v13H3Z', clock: 'M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0ZM12 7v5l3 2',
  pin: 'm9 3 6 0-1 6 4 4H6l4-4ZM12 13v8',
  // 功能：为本地规则自动整理按钮提供星芒标记；时间：2026-11-08；作者：lq。
  sparkles: 'm12 3 1.8 5.2L19 10l-5.2 1.8L12 17l-1.8-5.2L5 10l5.2-1.8L12 3ZM19 16l1 2.5 2.5 1-2.5 1L19 23l-1-2.5-2.5-1 2.5-1L19 16ZM5 2l.8 2.2L8 5l-2.2.8L5 8l-.8-2.2L2 5l2.2-.8L5 2Z',
  trash: 'M3 6h18M9 6V3h6v3M5 6l1 15h12l1-15M10 10v7M14 10v7',
  edit: 'm15 4 5 5M3 21l5-1L21 7l-5-5L3 15Z',
  shield: 'M12 2 3 6v6c0 5 9 10 9 10s9-5 9-10V6ZM8 12l3 3 5-6',
  download: 'M12 3v12m-5-5 5 5 5-5M4 16v5h16v-5',
  info: 'M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0ZM12 11v6M12 7h.01',
};
</script>

<template>
  <svg :width="size" :height="size" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <path :d="paths[name] || paths.globe" />
  </svg>
</template>
