<script setup lang="ts">
// 功能：提供原生下载活动、完成文件操作和失败重试入口；时间：2026-11-14；作者：lq。
import { computed } from 'vue';
import type { DownloadItem } from '../domain/models';
import AppIcon from './AppIcon.vue';
const props = defineProps<{ items: DownloadItem[] }>();
const emit = defineEmits<{ close: []; clear: []; open: [id: string]; reveal: [id: string]; retry: [item: DownloadItem] }>();
const activeCount = computed(() => props.items.filter(item => item.status === 'downloading').length);
const labels = { downloading: '正在下载', completed: '下载完成', failed: '下载失败', interrupted: '下载已中断' };
// 功能：以真实字节大小呈现下载状态，总大小未知时不显示虚假百分比；时间：2026-11-14；作者：lq。
function formatBytes(bytes: number) {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 ** 2) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 ** 3) return `${(bytes / 1024 ** 2).toFixed(1)} MB`;
  return `${(bytes / 1024 ** 3).toFixed(2)} GB`;
}
</script>

<template>
  <section class="downloads-view">
    <header class="downloads-heading"><div><h2>下载</h2><p>{{ activeCount ? `${activeCount} 个文件正在下载` : '最近的下载文件' }}</p></div><button class="icon-button" aria-label="关闭下载面板" @click="emit('close')"><AppIcon name="close" /></button></header>
    <div v-if="!items.length" class="download-empty"><AppIcon name="download" :size="32" /><h3>下载文件会显示在这里</h3><p>在网页中点击下载，选择保存位置后即可查看状态。</p></div>
    <ol v-else class="downloads-list" aria-label="下载文件">
      <li v-for="item in items" :key="item.id" :class="['download-item', `download-item--${item.status}`]">
        <div class="download-file-icon"><AppIcon :name="item.status === 'completed' ? 'check' : item.status === 'failed' || item.status === 'interrupted' ? 'info' : 'download'" :size="21" /></div>
        <div class="download-copy"><strong :title="item.name">{{ item.name }}</strong><p class="download-status"><span>{{ labels[item.status] }}</span><span v-if="item.status === 'completed' || item.receivedBytes">{{ formatBytes(item.receivedBytes) }}{{ item.status === 'downloading' ? ' 已接收' : '' }}</span><time :title="new Date(item.startedAt).toLocaleString()">{{ new Date(item.startedAt).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }) }}</time></p>
          <progress v-if="item.status === 'downloading'" :aria-label="`${item.name} 正在下载`"></progress>
          <p v-if="item.status === 'downloading'" class="download-hint">下载完成后可打开文件 · 请保留来源标签</p>
          <p v-if="item.error" class="download-error">{{ item.error }}</p><p class="download-path" :title="item.path">{{ item.path }}</p>
          <div class="download-actions"><template v-if="item.status === 'completed'"><button class="text-button" @click="emit('open', item.id)">打开文件</button><button class="text-button" @click="emit('reveal', item.id)">在文件夹中显示</button></template><button v-if="item.status === 'failed' || item.status === 'interrupted'" class="text-button" @click="emit('retry', item)">{{ /^https?:/i.test(item.url) ? '重新下载' : '返回来源页重新下载' }}</button></div>
        </div>
      </li>
    </ol>
    <footer class="panel-footer"><small class="muted">保留最近 200 项；清除记录不会删除文件。</small><button class="text-button" :disabled="!items.some(item => item.status !== 'downloading')" @click="emit('clear')">清除已结束记录</button></footer>
  </section>
</template>
