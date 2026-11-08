<script setup lang="ts">
// 功能：按 _5 页面稿实现收藏网格、列表、分类和分页；时间：2026-11-08；作者：lq。
import { computed, ref, watch } from 'vue';
import type { Bookmark, Folder } from '../domain/models';
import { api, desktop } from '../services/desktop';
import { displayHost } from '../domain/navigation';
import AppIcon from './AppIcon.vue';

const props = defineProps<{ folders: Folder[]; revision: number; homepage?: boolean }>();
const emit = defineEmits<{ navigate: [url: string]; create: []; edit: [bookmark: Bookmark]; settings: []; error: [message: string] }>();
const query = ref('');
const category = ref('');
const layout = ref<'grid' | 'list'>('grid');
const items = ref<Bookmark[]>([]);
const pinned = ref<Bookmark[]>([]);
const total = ref(0);
const loading = ref(false);
let request = 0;
const totalSaved = computed(() => props.folders.reduce((sum, folder) => sum + folder.count, 0));
const visibleFolders = computed(() => props.folders.filter(folder => folder.count > 0));

// 功能：丢弃过期查询响应，避免快速输入覆盖新结果；时间：2026-11-08；作者：lq。
async function load(append = false) {
  const sequence = ++request;
  loading.value = true;
  try {
    const page = await api.bookmarks(query.value, category.value, append ? items.value.length : 0);
    if (sequence !== request) return;
    items.value = append ? [...items.value, ...page.items] : page.items;
    total.value = page.total;
  } catch (error) { emit('error', String(error)); }
  finally { if (sequence === request) loading.value = false; }
}
watch([query, category, () => props.revision], () => { void load(); }, { immediate: true });
watch(() => props.revision, async () => {
  try { pinned.value = (await api.bookmarks('', '', 0, true)).items.slice(0, 6); }
  catch (error) { emit('error', String(error)); }
}, { immediate: true });
</script>

<template>
  <section class="bookmarks-view">
    <header class="page-heading">
      <div class="heading-with-icon"><span class="heading-icon"><AppIcon name="bookmarks" :size="24" /></span><div><h1>{{ homepage ? '收藏主页' : '收藏中心' }} <small>bookmarks.sab</small></h1><p><span class="status-dot"></span>{{ totalSaved }} 个本地收藏</p></div></div>
      <div class="inline-actions"><button class="button secondary" @click="emit('settings')"><AppIcon name="download" :size="16" />书签导入</button><button class="button primary" :disabled="!desktop" @click="emit('create')"><AppIcon name="plus" :size="16" />新建收藏</button></div>
    </header>
    <div class="collection-toolbar">
      <label class="filter-input"><AppIcon name="search" /><input v-model="query" placeholder="搜索收藏标题、网址与标签" aria-label="搜索收藏" /><button v-if="query" class="icon-button" aria-label="清除搜索" @click="query = ''"><AppIcon name="close" :size="14" /></button></label>
      <div class="segmented-control" aria-label="收藏展示方式"><button :class="{ selected: layout === 'grid' }" :aria-pressed="layout === 'grid'" @click="layout = 'grid'"><AppIcon name="grid" :size="16" />网格</button><button :class="{ selected: layout === 'list' }" :aria-pressed="layout === 'list'" @click="layout = 'list'"><AppIcon name="list" :size="16" />列表</button></div>
    </div>
    <section v-if="pinned.length && !query" class="pinned-section">
      <h2><AppIcon name="pin" :size="16" />常用置顶 · Pinned <span class="count-badge">{{ pinned.length }} 个</span></h2>
      <div class="pinned-grid"><button v-for="bookmark in pinned" :key="bookmark.id" class="pinned-site" @click="emit('navigate', bookmark.url)"><span class="site-icon"><AppIcon name="globe" /></span><strong>{{ bookmark.title }}</strong><small>{{ displayHost(bookmark.url) }}</small></button></div>
    </section>
    <nav class="category-tabs" aria-label="收藏分类"><button :class="{ active: !category }" :aria-pressed="!category" @click="category = ''">全部 ({{ totalSaved }})</button><button v-for="folder in visibleFolders" :key="folder.id" :class="{ active: category === folder.id, inbox: folder.id === 'inbox' }" :aria-pressed="category === folder.id" @click="category = folder.id">{{ folder.name }} ({{ folder.count }})</button></nav>
    <div v-if="!items.length && !loading" class="empty-state"><AppIcon :name="query ? 'search' : 'bookmarks'" :size="36" /><h2>{{ query ? '没有找到相关收藏' : '收藏常用的网站' }}</h2><p>{{ query ? '试试其他标题、域名或标签。' : '保存后即可在主页和收藏中心快速打开。' }}</p><button v-if="!query" class="button primary" :disabled="!desktop" @click="emit('create')">新建收藏</button></div>
    <div :class="['bookmark-collection', layout]" :aria-busy="loading">
      <article v-for="bookmark in items" :key="bookmark.id" class="bookmark-card">
        <button class="site-link" @click="emit('navigate', bookmark.url)"><span class="site-icon"><AppIcon name="globe" /></span><span><strong>{{ bookmark.title }}</strong><small>{{ displayHost(bookmark.url) }}</small></span></button>
        <button class="icon-button edit-bookmark" :aria-label="`编辑 ${bookmark.title}`" @click="emit('edit', bookmark)"><AppIcon name="more" :size="16" /></button>
        <div class="bookmark-meta"><span v-for="tag in bookmark.tags" :key="tag" class="tag">#{{ tag }}</span><span class="folder-label">{{ folders.find(folder => folder.id === bookmark.folderId)?.name }}</span><AppIcon v-if="bookmark.pinned" name="pin" :size="13" /></div>
      </article>
    </div>
    <footer v-if="total" class="collection-footer"><span>{{ total }} 条收藏 · 按最近修改排序</span><span>已显示 {{ items.length }} 项</span><button v-if="items.length < total" class="button secondary" :disabled="loading" @click="load(true)">{{ loading ? '加载中…' : '加载更多' }}</button></footer>
  </section>
</template>
