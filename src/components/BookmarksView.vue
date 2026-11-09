<script setup lang="ts">
// 功能：按 _5 页面稿实现收藏网格、列表、分类和分页；时间：2026-11-08；作者：lq。
import { computed, ref, watch } from 'vue';
import type { Bookmark, ClassificationBatchResult, ClassificationPreview, Folder } from '../domain/models';
import { api, desktop } from '../services/desktop';
import { displayHost } from '../domain/navigation';
import AppIcon from './AppIcon.vue';
import FolderManager from './FolderManager.vue';

const props = defineProps<{ folders: Folder[]; revision: number; homepage?: boolean }>();
const emit = defineEmits<{ navigate: [url: string]; create: []; edit: [bookmark: Bookmark]; settings: []; organized: []; foldersChanged: []; error: [message: string] }>();
const query = ref('');
const category = ref('');
const layout = ref<'grid' | 'list'>('grid');
const items = ref<Bookmark[]>([]);
const pinned = ref<Bookmark[]>([]);
const total = ref(0);
const loading = ref(false);
// 功能：记录规则分类预览、执行进度及最近批次撤销入口；时间：2026-11-08；作者：lq。
const classificationPreview = ref<ClassificationPreview>();
const classificationBatch = ref<ClassificationBatchResult>();
const organizing = ref(false);
const organizationMessage = ref('');
const folderManagerOpen = ref(false);
let request = 0;
const totalSaved = computed(() => props.folders.reduce((sum, folder) => sum + folder.count, 0));
const visibleFolders = computed(() => props.folders.filter(folder => folder.count > 0));
const pendingCount = computed(() => props.folders.find(folder => folder.id === 'inbox')?.count ?? 0);
// 功能：分类删除或清空后退出失效筛选，并清除旧的分类建议；时间：2026-11-08；作者：lq。
watch(() => props.folders, folders => {
  if (category.value && !folders.some(folder => folder.id === category.value && folder.count > 0)) category.value = '';
  classificationPreview.value = undefined;
});

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
// 功能：展开按分类分组的本地规则预览，确认前不改变收藏；时间：2026-11-08；作者：lq。
async function showClassificationPreview() {
  organizing.value = true;
  try { classificationPreview.value = await api.classificationPreview(); }
  catch (error) { emit('error', String(error)); }
  finally { organizing.value = false; }
}
// 功能：应用分类预览并保留最近批次的可撤销标识；时间：2026-11-08；作者：lq。
async function applyClassification() {
  organizing.value = true;
  try {
    classificationBatch.value = await api.applyClassification();
    classificationPreview.value = undefined;
    organizationMessage.value = `已整理 ${classificationBatch.value.changed} 条，${classificationBatch.value.remaining} 条仍待整理。`;
    emit('organized');
  } catch (error) { emit('error', String(error)); }
  finally { organizing.value = false; }
}
// 功能：撤销最近一次规则整理并让后续手动改动保持原样；时间：2026-11-08；作者：lq。
async function undoClassification() {
  const batch = classificationBatch.value;
  if (!batch) return;
  organizing.value = true;
  try {
    const result = await api.undoClassification(batch.batchId);
    organizationMessage.value = `已撤销并还原 ${result.restored} 条收藏。`;
    classificationBatch.value = undefined;
    emit('organized');
  } catch (error) { emit('error', String(error)); }
  finally { organizing.value = false; }
}
</script>

<template>
  <section class="bookmarks-view">
    <header class="page-heading">
      <div class="heading-with-icon"><span class="heading-icon"><AppIcon name="bookmarks" :size="24" /></span><div><h1>{{ homepage ? '收藏主页' : '收藏中心' }} <small>bookmarks.sab</small></h1><p><span class="status-dot"></span>{{ totalSaved }} 个本地收藏</p></div></div>
      <div class="inline-actions"><button v-if="desktop && pendingCount" class="button secondary" :disabled="organizing" @click="showClassificationPreview"><AppIcon name="sparkles" :size="16" />智能整理 {{ pendingCount }}</button><button v-if="desktop" class="button secondary" @click="folderManagerOpen = true"><AppIcon name="folder" :size="16" />管理分类</button><button class="button secondary" @click="emit('settings')"><AppIcon name="download" :size="16" />书签导入</button><button class="button primary" :disabled="!desktop" @click="emit('create')"><AppIcon name="plus" :size="16" />新建收藏</button></div>
    </header>
    <FolderManager v-if="folderManagerOpen" :folders="folders" @close="folderManagerOpen = false" @changed="emit('foldersChanged')" @error="emit('error', $event)" />
    <section v-if="classificationPreview" class="classification-preview" aria-live="polite">
      <header><div><h2>本地规则整理预览</h2><p>只根据书签标题和网址判断，不会打开网页或上传数据。确认后可撤销。</p></div><button class="icon-button" aria-label="关闭分类预览" @click="classificationPreview = undefined"><AppIcon name="close" :size="16" /></button></header>
      <div v-if="classificationPreview.suggested" class="classification-groups"><article v-for="group in classificationPreview.groups" :key="group.folderId"><strong>{{ group.folderName }} <span>{{ group.count }}</span></strong><small v-for="sample in group.samples" :key="sample">{{ sample }}</small></article></div>
      <p v-else class="classification-empty">目前没有足够明确的规则建议，收藏会继续留在“待整理”。</p>
      <!-- 功能：明确展示冲突或信息不足的待整理样例，避免把证据评分误当成准确率；时间：2026-11-09；作者：lq。 -->
      <p v-if="classificationPreview.remainingSamples?.length" class="classification-empty">以下收藏信息不足或类别冲突，建议手动检查：{{ classificationPreview.remainingSamples.join('、') }}。分类评分表示证据强度，不代表准确率。</p>
      <footer><span>建议整理 {{ classificationPreview.suggested }} 条 · 保留待整理 {{ classificationPreview.remaining }} 条</span><div><button class="button secondary" @click="classificationPreview = undefined">取消</button><button class="button primary" :disabled="organizing || !classificationPreview.suggested" @click="applyClassification">{{ organizing ? '整理中…' : '确认整理' }}</button></div></footer>
    </section>
    <div v-if="organizationMessage" class="classification-result" role="status"><span>{{ organizationMessage }}</span><button v-if="classificationBatch?.changed" class="text-button" :disabled="organizing" @click="undoClassification">撤销整理</button></div>
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
        <div class="bookmark-meta"><span v-for="tag in bookmark.tags" :key="tag" class="tag">#{{ tag }}</span><span class="folder-label" :title="bookmark.ruleReason || undefined">{{ folders.find(folder => folder.id === bookmark.folderId)?.name }}</span><AppIcon v-if="bookmark.pinned" name="pin" :size="13" /></div>
      </article>
    </div>
    <footer v-if="total" class="collection-footer"><span>{{ total }} 条收藏 · 按最近修改排序</span><span>已显示 {{ items.length }} 项</span><button v-if="items.length < total" class="button secondary" :disabled="loading" @click="load(true)">{{ loading ? '加载中…' : '加载更多' }}</button></footer>
  </section>
</template>
