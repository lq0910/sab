<script setup lang="ts">
// 功能：展示、搜索、分页和管理本地浏览历史；时间：2026-11-08；作者：lq。
import { ref, watch } from 'vue';
import type { HistoryPage, RecentVisit } from '../domain/models';
import { api } from '../services/desktop';
import AppIcon from './AppIcon.vue';

const props = defineProps<{ revision: number }>();
const emit = defineEmits<{ navigate: [url: string]; delete: [url: string]; clear: []; error: [message: string] }>();
const query = ref('');
const items = ref<RecentVisit[]>([]);
const total = ref(0);
const loading = ref(false);
let request = 0;

// 功能：按关键词加载历史首屏或后续分页，并忽略过期响应；时间：2026-11-08；作者：lq。
async function load(append = false) {
  const sequence = ++request;
  loading.value = true;
  try {
    const page: HistoryPage = await api.history(query.value, append ? items.value.length : 0, 100);
    if (sequence !== request) return;
    items.value = append ? [...items.value, ...page.items] : page.items;
    total.value = page.total;
  } catch (error) { emit('error', String(error)); }
  finally {
    if (sequence === request) loading.value = false;
  }
}

watch([query, () => props.revision], () => { void load(); }, { immediate: true });
</script>

<template>
  <section class="history-view">
    <header class="page-heading">
      <div class="heading-with-icon"><span class="heading-icon"><AppIcon name="clock" :size="24" /></span><div><h1>浏览历史 <small>history.sab</small></h1><p><span class="status-dot"></span>{{ total }} 条本地记录</p></div></div>
      <button class="button destructive" :disabled="!total || loading" @click="emit('clear')"><AppIcon name="trash" :size="15" />清除浏览数据</button>
    </header>
    <label class="history-search"><AppIcon name="search" /><input v-model="query" aria-label="搜索浏览历史" placeholder="搜索标题或网址" /><button v-if="query" class="icon-button" aria-label="清除搜索" @click="query = ''"><AppIcon name="close" :size="14" /></button></label>
    <div v-if="!items.length && !loading" class="empty-state"><AppIcon name="clock" :size="36" /><h2>{{ query ? '没有找到匹配记录' : '还没有浏览记录' }}</h2><p>{{ query ? '试试其他标题或网址。' : '通过地址栏打开网页后，记录会显示在这里。' }}</p></div>
    <div class="history-list" :aria-busy="loading">
      <article v-for="visit in items" :key="visit.url" class="history-row">
        <button class="history-open" @click="emit('navigate', visit.url)"><span class="site-icon"><AppIcon name="globe" /></span><span class="history-copy"><strong>{{ visit.title }}</strong><small>{{ visit.url }}</small></span><time>{{ new Date(visit.visitedAt).toLocaleString() }}</time></button>
        <button class="icon-button history-delete" :aria-label="`删除 ${visit.title} 的历史记录`" @click="emit('delete', visit.url)"><AppIcon name="trash" :size="15" /></button>
      </article>
    </div>
    <footer v-if="items.length < total" class="collection-footer"><span>已显示 {{ items.length }} / {{ total }} 条</span><button class="button secondary" :disabled="loading" @click="load(true)">{{ loading ? '加载中…' : '加载更多' }}</button></footer>
  </section>
</template>
