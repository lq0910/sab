<script setup lang="ts">
// @ts-nocheck
// 功能：连接浏览器框架、本地页面、收藏浮层和受限原生视图；时间：2026-11-08；作者：lq。
import { computed, nextTick, onMounted, onUnmounted, reactive, ref, watch } from 'vue';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { LogicalPosition } from '@tauri-apps/api/dpi';
import { Menu, MenuItem } from '@tauri-apps/api/menu';
import { getCurrentWebview } from '@tauri-apps/api/webview';
import { getCurrentWindow } from '@tauri-apps/api/window';
import appLogo from '../src-tauri/icons/source.svg';
import AppIcon from './components/AppIcon.vue';
import BookmarksView from './components/BookmarksView.vue';
import HomeView from './components/HomeView.vue';
import OverlayPanel from './components/OverlayPanel.vue';
import SettingsView from './components/SettingsView.vue';
import { defaultSettings, type Bookmark, type BookmarkDraft, type Folder, type PageEvent, type RecentVisit, type SearchEngine, type Settings, type Tab, type ThemeId, type ViewBounds } from './domain/models';
import { displayHost, isInternal, resolveAddress } from './domain/navigation';
import { api, desktop } from './services/desktop';

const newTab = (url = 'sab://home'): Tab => ({ id: crypto.randomUUID(), url, title: '轻页 SAB Workspace', loading: false });
const tabs = ref<Tab[]>([newTab()]);
const activeId = ref(tabs.value[0]!.id);
const active = computed(() => tabs.value.find(tab => tab.id === activeId.value)!);
const address = ref(active.value.url);
const addressElement = ref<HTMLInputElement>();
const viewport = ref<HTMLElement>();
const tabsElement = ref<HTMLElement>();
const tabPickerButton = ref<HTMLElement>();
const tabPickerSearch = ref<HTMLInputElement>();
const tabPickerPosition = ref({ left: 12, top: 42 });
const tabMotion = ref<'from-left' | 'from-right' | ''>('');
const settings = ref<Settings>({ ...defaultSettings });
const folders = ref<Folder[]>([]);
const recent = ref<RecentVisit[]>([]);
const revision = ref(0);
const message = ref('');
const pageError = ref('');
const initialized = ref(false);
const panel = ref<'' | 'favorites' | 'edit' | 'menu'>('');
const contextTabId = ref('');
const tabPickerOpen = ref(false);
const tabPickerQuery = ref('');
const tabsOverflow = ref(false);
const favoriteQuery = ref('');
const favoriteCategory = ref('');
const favorites = ref<Bookmark[]>([]);
const favoriteIndex = ref(0);
const editingId = ref('');
const bookmarkDraft = reactive<BookmarkDraft>({ url: '', title: '', folderId: 'inbox', tags: [], pinned: false });
const tagInput = ref('');
const saving = ref(false);
const created = new Set<string>();
const closed: Tab[] = [];
const disposers: UnlistenFn[] = [];
let nativeTabMenu: Menu | undefined;
let nativeReloadItem: MenuItem | undefined;
let observer: ResizeObserver | undefined;
let nativeQueue = Promise.resolve();
let searchSequence = 0;
const internal = computed(() => isInternal(active.value.url));
const filteredTabs = computed(() => {
  const query = tabPickerQuery.value.trim().toLocaleLowerCase();
  if (!query) return tabs.value;
  return tabs.value.filter(tab => `${tab.title} ${tab.url} ${isInternal(tab.url) ? '轻页 SAB' : displayHost(tab.url)}`.toLocaleLowerCase().includes(query));
});
const contentPage = computed(() => active.value.url === 'sab://home'
  ? (settings.value.homeMode === 'minimal' ? 'home' : 'bookmarks')
  : active.value.url === 'sab://bookmarks' ? 'bookmarks' : active.value.url === 'sab://settings' ? 'settings' : 'web');

// 功能：为自绘标题栏提供最小化、最大化和关闭窗口操作；时间：2026-11-08；作者：lq。
async function controlWindow(action: 'minimize' | 'maximize' | 'close') {
  if (!desktop) return;
  const window = getCurrentWindow();
  if (action === 'minimize') await window.minimize();
  else if (action === 'maximize') await window.toggleMaximize();
  else await window.close();
}

// 功能：通过 Tauri 原生窗口接口拖动无边框窗口，跳过标签和按钮等交互控件；时间：2026-11-08；作者：lq。
function dragWindow(event: MouseEvent) {
  if (!desktop || event.button !== 0 || !(event.target instanceof Element)) return;
  if (event.target.closest('button, input, a, [role="tab"]')) return;
  void safely(() => getCurrentWindow().startDragging());
}

// 功能：集中显示异步错误，避免事件处理器产生未处理拒绝；时间：2026-11-08；作者：lq。
async function safely(operation: () => Promise<unknown>) {
  try { await operation(); } catch (error) { message.value = String(error); }
}
function enqueue(operation: () => Promise<void>): Promise<void> {
  const pending = nativeQueue.then(operation);
  nativeQueue = pending.catch(() => {});
  return pending;
}
function getBounds(): ViewBounds {
  const rect = viewport.value!.getBoundingClientRect();
  return { x: rect.x, y: rect.y, width: Math.max(1, rect.width), height: Math.max(1, rect.height) };
}

// 功能：检测标签栏是否溢出，并在下一次布局后同步测量结果；时间：2026-11-08；作者：lq。
async function measureTabsOverflow() {
  await nextTick();
  const element = tabsElement.value;
  if (element) tabsOverflow.value = element.scrollWidth > element.clientWidth + 1;
}

// 功能：打开可搜索的标签列表并把焦点放入搜索框；时间：2026-11-08；作者：lq。
async function openTabPicker() {
  const rect = tabPickerButton.value?.getBoundingClientRect();
  if (rect) tabPickerPosition.value = {
    left: Math.max(8, Math.min(rect.left, window.innerWidth - 328)),
    top: rect.bottom + 6,
  };
  tabPickerOpen.value = true;
  tabPickerQuery.value = '';
  if (desktop) await enqueue(() => api.view('hide', active.value.id));
  await nextTick();
  tabPickerSearch.value?.focus();
}

// 功能：关闭标签列表后恢复当前网页视图；时间：2026-11-08；作者：lq。
async function closeTabPicker() {
  if (!tabPickerOpen.value) return;
  tabPickerOpen.value = false;
  await syncView();
}

// 功能：在标签列表中激活目标页并滚动回可见区域；时间：2026-11-08；作者：lq。
async function chooseTab(id: string) {
  tabPickerOpen.value = false;
  await activate(id);
  await nextTick();
  tabsElement.value?.querySelector<HTMLElement>(`[data-tab-id="${id}"]`)?.scrollIntoView({ block: 'nearest', inline: 'nearest' });
}

// 功能：从搜索列表关闭标签并收起列表，避免原生网页遮挡浮层；时间：2026-11-08；作者：lq。
async function closeTabFromPicker(id: string) {
  tabPickerOpen.value = false;
  await closeTab(id);
}

// 功能：串行处理标签切换和浮层避让，保持最终视图与活动标签一致；时间：2026-11-08；作者：lq。
async function syncView() {
  if (!desktop || !initialized.value) return;
  await nextTick();
  await enqueue(async () => {
    const tab = active.value;
    if (internal.value || panel.value) { await api.view('hide', tab.id); return; }
    const exists = created.has(tab.id);
    await api.view(exists ? 'activate' : 'open', tab.id, tab.url, getBounds());
    created.add(tab.id);
  });
}
async function activate(id: string) {
  const previousIndex = tabs.value.findIndex(tab => tab.id === activeId.value);
  const nextIndex = tabs.value.findIndex(tab => tab.id === id);
  if (id === activeId.value) return;
  tabMotion.value = nextIndex > previousIndex ? 'from-left' : 'from-right';
  activeId.value = id;
  message.value = '';
  pageError.value = '';
  await syncView();
}
async function navigate(input: string, newWindow = false) {
  const url = resolveAddress(input, settings.value.searchEngine);
  panel.value = '';
  message.value = '';
  pageError.value = '';
  if (newWindow) { const tab = newTab(url); tabs.value.push(tab); tabMotion.value = 'from-left'; activeId.value = tab.id; }
  const tab = active.value;
  tab.url = url;
  tab.title = isInternal(url) ? (url === 'sab://settings' ? '设置 - 轻页 SAB' : url === 'sab://bookmarks' ? '收藏中心' : '轻页 SAB Workspace') : displayHost(url);
  tab.loading = !isInternal(url) && desktop;
  address.value = url;
  if (isInternal(url)) { await syncView(); return; }
  await nextTick();
  try {
    await enqueue(async () => {
      await api.view('open', tab.id, url, getBounds());
      created.add(tab.id);
    });
  } catch (error) { tab.loading = false; pageError.value = String(error); }
}
async function closeTab(id: string) {
  const index = tabs.value.findIndex(tab => tab.id === id);
  if (index < 0) return;
  const tab = tabs.value[index]!;
  if (created.has(id)) { await enqueue(() => api.view('close', id)); created.delete(id); }
  closed.push({ ...tab, loading: false });
  if (closed.length > 20) closed.shift();
  tabs.value.splice(index, 1);
  if (!tabs.value.length) tabs.value.push(newTab());
  if (activeId.value === id) {
    const nextIndex = Math.min(index, tabs.value.length - 1);
    tabMotion.value = nextIndex >= index ? 'from-left' : 'from-right';
    activeId.value = tabs.value[nextIndex]!.id;
  }
  await measureTabsOverflow();
  await syncView();
}
// 功能：在标签栏右键处弹出系统原生菜单，避免被独立网页视图遮挡；时间：2026-11-08；作者：lq。
async function openTabContext(event: MouseEvent, tabId: string) {
  event.preventDefault();
  contextTabId.value = tabId;
  const tab = tabs.value.find(item => item.id === tabId);
  await nativeReloadItem?.setEnabled(!!tab && !isInternal(tab.url));
  await nativeTabMenu?.popup(new LogicalPosition(event.clientX, event.clientY));
}
// 功能：按系统菜单回调处理指定标签，原生菜单由操作系统负责关闭；时间：2026-11-08；作者：lq。
function runTabMenu(action: (id: string) => Promise<unknown>) {
  const id = contextTabId.value;
  if (id) void safely(() => action(id));
}
async function duplicateTab(id: string) {
  const tab = tabs.value.find(item => item.id === id);
  if (tab) await navigate(tab.url, true);
}
async function closeOtherTabs(id: string) {
  await activate(id);
  for (const tab of [...tabs.value].filter(item => item.id !== id)) await closeTab(tab.id);
}
async function copyTabAddress(id: string) {
  const tab = tabs.value.find(item => item.id === id);
  if (!tab) return;
  await navigator.clipboard.writeText(tab.url);
  message.value = '已复制网址';
}
async function refreshData() {
  const state = await api.bootstrap();
  settings.value = { ...defaultSettings, ...state.settings };
  applyTheme(settings.value.theme);
  folders.value = state.folders;
  recent.value = await api.recent();
  revision.value++;
}
async function saveSettings(value: Settings) {
  await api.saveSettings(value);
  await refreshData();
  message.value = '设置已保存';
}
// 功能：即时切换根节点主题令牌，并让保存设置持久化；时间：2026-11-08；作者：lq。
function applyTheme(theme: ThemeId) { document.documentElement.dataset.theme = theme; }
function previewTheme(theme: ThemeId) { settings.value.theme = theme; applyTheme(theme); }
async function changeEngine(engine: SearchEngine) { await saveSettings({ ...settings.value, searchEngine: engine }); }
async function clearHistory() { await api.clearHistory(); recent.value = []; message.value = '浏览历史已删除'; }
async function hideRecent(url: string) { await api.hideRecent(url); recent.value = await api.recent(); }

// 功能：先隐藏原生网页再显示 HTML 浮层，关闭后恢复原标签；时间：2026-11-08；作者：lq。
async function showPanel(value: typeof panel.value) {
  if (desktop) await enqueue(() => api.view('hide', active.value.id));
  message.value = '';
  panel.value = value;
}
async function closePanel() { panel.value = ''; await syncView(); }
async function editBookmark(bookmark?: Bookmark) {
  editingId.value = bookmark?.id ?? '';
  Object.assign(bookmarkDraft, bookmark ?? { url: internal.value ? '' : active.value.url, title: internal.value ? '' : active.value.title, folderId: 'inbox', tags: [], pinned: false });
  tagInput.value = bookmarkDraft.tags.join(', ');
  await showPanel('edit');
}
async function saveCurrent() {
  if (internal.value) { message.value = '请先打开要收藏的网页'; return; }
  const matches = await api.bookmarks(active.value.url);
  await editBookmark(matches.items.find(item => item.url === active.value.url));
}
async function submitBookmark() {
  if (saving.value) return;
  saving.value = true;
  try {
    await api.saveBookmark({ ...bookmarkDraft, tags: tagInput.value.split(/[,，]/).map(t => t.trim()).filter(Boolean) });
    await refreshData();
    await closePanel();
    message.value = '收藏已保存';
  } finally { saving.value = false; }
}
async function deleteBookmark() {
  await api.deleteBookmark(editingId.value);
  await refreshData();
  await closePanel();
  message.value = '收藏已删除';
}
async function favoriteKey(event: KeyboardEvent) {
  if (event.isComposing) return;
  if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
    event.preventDefault();
    favoriteIndex.value = Math.max(0, Math.min(favorites.value.length - 1, favoriteIndex.value + (event.key === 'ArrowDown' ? 1 : -1)));
    document.getElementById(`favorite-${favoriteIndex.value}`)?.scrollIntoView({ block: 'nearest' });
  } else if (event.key === 'Enter' && favorites.value[favoriteIndex.value]) {
    event.preventDefault();
    await navigate(favorites.value[favoriteIndex.value]!.url, event.metaKey || event.ctrlKey);
  }
}
// 功能：为模板绑定显式类型的事件处理器；时间：2026-11-08；作者：lq。
function homeNavigate(value: string) { void safely(() => navigate(value)); }
function openBookmark(url: string) { void safely(() => navigate(url)); }
function editExistingBookmark(bookmark: Bookmark) { void safely(() => editBookmark(bookmark)); }
function reportComponentError(value: string) { message.value = value; }
function saveSettingsFromView(value: Settings) { void safely(() => saveSettings(value)); }
function updateEngine(value: SearchEngine) { void safely(() => changeEngine(value)); }

// 功能：绑定本地 UI 快捷键；网页持有焦点时的原生快捷键另行验收；时间：2026-11-08；作者：lq。
function keyboard(event: KeyboardEvent) {
  if (event.key === 'Escape' && tabPickerOpen.value) { void safely(closeTabPicker); return; }
  if (event.isComposing || !(event.metaKey || event.ctrlKey)) return;
  const key = event.key.toLowerCase();
  const action = key === 'a' && event.shiftKey ? openTabPicker
    : key === 'l' ? async () => { await closePanel(); addressElement.value?.focus(); addressElement.value?.select(); }
    : key === 't' && event.shiftKey ? async () => { const tab = closed.pop(); if (tab) await navigate(tab.url, true); }
    : key === 't' ? () => navigate('sab://home', true)
    : key === 'w' && !panel.value ? () => closeTab(activeId.value)
    : key === 'd' ? saveCurrent
    : key === 'o' && event.shiftKey ? () => showPanel('favorites')
    : key === ',' ? () => navigate('sab://settings')
    : key === 'r' && !internal.value ? () => api.view('reload', activeId.value)
    : undefined;
  if (action) { event.preventDefault(); void safely(action); }
}
watch(() => active.value.url, value => { address.value = value; });
watch([favoriteQuery, favoriteCategory, revision], async () => {
  const sequence = ++searchSequence;
  try {
    const page = await api.bookmarks(favoriteQuery.value, favoriteCategory.value);
    if (sequence === searchSequence) { favorites.value = page.items; favoriteIndex.value = 0; }
  } catch (error) { message.value = String(error); }
}, { immediate: true });

onMounted(() => { void safely(async () => {
  const launchStartedAt = performance.now();
  window.addEventListener('keydown', keyboard);
  try {
  if (desktop) {
    // 功能：注册系统原生标签菜单及刷新、复制、关闭等操作；时间：2026-11-08；作者：lq。
    nativeReloadItem = await MenuItem.new({ id: 'reload', text: '重新加载', action: () => runTabMenu(id => api.view('reload', id)) });
    nativeTabMenu = await Menu.new({ items: [
      nativeReloadItem,
      { id: 'duplicate', text: '复制标签页', action: () => runTabMenu(duplicateTab) },
      { id: 'copy-address', text: '复制网址', action: () => runTabMenu(copyTabAddress) },
      { item: 'Separator' },
      { id: 'close-others', text: '关闭其他标签页', action: () => runTabMenu(closeOtherTabs) },
      { id: 'close', text: '关闭标签页', action: () => runTabMenu(closeTab) },
    ] });
    const webview = getCurrentWebview();
    disposers.push(await webview.listen<PageEvent>('sab:page', event => {
      const tab = tabs.value.find(item => item.id === event.payload.tabId);
      if (!tab || isInternal(tab.url)) return;
      tab.url = event.payload.url;
      if (event.payload.title) tab.title = event.payload.title;
      if (event.payload.loading !== undefined) tab.loading = event.payload.loading;
    }));
    disposers.push(await listen<PageEvent>('sab:new-tab', event => { void safely(() => navigate(event.payload.url, true)); }));
  }
  await refreshData();
  observer = new ResizeObserver(() => { void safely(syncView); void measureTabsOverflow(); });
  if (viewport.value) observer.observe(viewport.value);
  if (tabsElement.value) observer.observe(tabsElement.value);
  await measureTabsOverflow();
  if (settings.value.startupUrl) await navigate(settings.value.startupUrl);
  } finally {
    // 功能：为冷启动品牌动画保留短暂展示时间，并在初始化失败时也移除启动层；时间：2026-11-08；作者：lq。
    await new Promise(resolve => window.setTimeout(resolve, Math.max(0, 620 - (performance.now() - launchStartedAt))));
    initialized.value = true;
  }
}); });
onUnmounted(() => { observer?.disconnect(); disposers.forEach(stop => stop()); window.removeEventListener('keydown', keyboard); void nativeTabMenu?.close(); void nativeReloadItem?.close(); });
watch(tabs, () => { void measureTabsOverflow(); }, { deep: true });
</script>

<template>
  <div class="browser-shell" @click="safely(closeTabPicker)">
    <header class="browser-chrome">
      <div class="tab-strip" @mousedown="dragWindow">
        <!-- 功能：将窗口控制按钮与标签页放入同一栏；时间：2026-11-08；作者：lq。 -->
        <div class="window-controls">
          <button class="window-control close" aria-label="关闭窗口" title="关闭窗口" @click.stop="safely(() => controlWindow('close'))"></button>
          <button class="window-control minimize" aria-label="最小化窗口" title="最小化窗口" @click.stop="safely(() => controlWindow('minimize'))"></button>
          <button class="window-control maximize" aria-label="最大化窗口" title="最大化窗口" @click.stop="safely(() => controlWindow('maximize'))"></button>
        </div>
        <!-- 功能：仅在标签栏溢出时显示可搜索的标签管理入口；时间：2026-11-08；作者：lq。 -->
        <button v-if="tabsOverflow" ref="tabPickerButton" class="tab-picker-trigger" aria-label="搜索标签页" title="搜索标签页 (⌘/Ctrl+Shift+A)" :aria-expanded="tabPickerOpen" @click.stop="safely(openTabPicker)"><AppIcon name="chevron" :size="17" /></button>
        <div ref="tabsElement" class="tabs" role="tablist" aria-label="浏览器标签">
          <div v-for="tab in tabs" :key="tab.id" :data-tab-id="tab.id" :class="['browser-tab', { active: activeId === tab.id, 'flow-from-left': activeId === tab.id && tabMotion === 'from-left', 'flow-from-right': activeId === tab.id && tabMotion === 'from-right' }]" @contextmenu="safely(() => openTabContext($event, tab.id))">
            <button role="tab" :aria-selected="activeId === tab.id" :tabindex="activeId === tab.id ? 0 : -1" @click="safely(() => activate(tab.id))" @keydown.right.prevent="safely(() => activate(tabs[(tabs.findIndex(item => item.id === tab.id) + 1) % tabs.length]!.id))" @keydown.left.prevent="safely(() => activate(tabs[(tabs.findIndex(item => item.id === tab.id) - 1 + tabs.length) % tabs.length]!.id))"><AppIcon :name="isInternal(tab.url) ? 'grid' : 'globe'" :size="15" /><span>{{ tab.loading ? '加载中…' : tab.title }}</span></button>
            <button class="tab-close" :aria-label="`关闭 ${tab.title}`" @click="safely(() => closeTab(tab.id))"><AppIcon name="close" :size="13" /></button>
          </div>
        </div>
        <button class="icon-button new-tab" aria-label="新建标签" title="新建标签 (⌘/Ctrl+T)" @click="safely(() => navigate('sab://home', true))"><AppIcon name="plus" /></button>
      </div>
      <div class="navigation-bar">
        <button class="icon-button" aria-label="后退" :disabled="internal || !desktop" @click="safely(() => api.view('back', activeId))"><AppIcon name="back" /></button>
        <button class="icon-button" aria-label="前进" :disabled="internal || !desktop" @click="safely(() => api.view('forward', activeId))"><AppIcon name="forward" /></button>
        <button class="icon-button" aria-label="刷新" :disabled="internal || !desktop" @click="safely(() => api.view('reload', activeId))"><AppIcon name="refresh" /></button>
        <form class="address-bar" @submit.prevent="safely(() => navigate(address))"><AppIcon :name="internal ? 'settings' : 'globe'" :size="15" /><input ref="addressElement" v-model="address" aria-label="地址或搜索" spellcheck="false" autocomplete="off" @focus="($event.target as HTMLInputElement).select()" /><kbd>⌘L</kbd></form>
        <button class="icon-button" aria-label="收藏当前页面" title="收藏当前页面 (⌘/Ctrl+D)" :disabled="internal || !desktop" @click="safely(saveCurrent)"><AppIcon name="star" /></button>
        <button class="icon-button" aria-label="打开收藏中心浮层" title="收藏中心 (⌘/Ctrl+Shift+O)" @click="safely(() => showPanel('favorites'))"><AppIcon name="bookmarks" /></button>
        <button class="icon-button" aria-label="浏览器菜单" @click="safely(() => showPanel('menu'))"><AppIcon name="more" /></button>
      </div>
    </header>
    <!-- 功能：提供 Chrome 风格的已打开标签搜索、切换和关闭列表；时间：2026-11-08；作者：lq。 -->
    <section v-if="tabPickerOpen" class="tab-picker" aria-label="打开的标签页" :style="{ left: `${tabPickerPosition.left}px`, top: `${tabPickerPosition.top}px` }" @click.stop>
      <label class="tab-picker-search"><AppIcon name="search" :size="19" /><input ref="tabPickerSearch" v-model="tabPickerQuery" aria-label="搜索标签页" placeholder="搜索标签页" /><kbd>⌘⇧A</kbd></label>
      <h2>打开的标签页 <span>{{ filteredTabs.length }}</span></h2>
      <div class="tab-picker-list">
        <p v-if="!filteredTabs.length" class="tab-picker-empty">没有匹配的标签页</p>
        <div v-for="tab in filteredTabs" :key="tab.id" :class="['tab-picker-item', { selected: activeId === tab.id }]">
          <button class="tab-picker-select" :aria-label="`切换到 ${tab.title}`" @click="safely(() => chooseTab(tab.id))"><span class="tab-picker-icon"><AppIcon :name="isInternal(tab.url) ? 'grid' : 'globe'" :size="17" /></span><span class="tab-picker-copy"><strong>{{ tab.loading ? '加载中…' : tab.title }}</strong><small>{{ isInternal(tab.url) ? '轻页 SAB' : displayHost(tab.url) }}</small></span></button>
          <button class="tab-picker-close" :aria-label="`关闭 ${tab.title}`" @click="safely(() => closeTabFromPicker(tab.id))"><AppIcon name="close" :size="16" /></button>
        </div>
      </div>
    </section>
    <main ref="viewport" class="content-viewport">
      <div v-if="!desktop" class="preview-notice">界面预览 · 浏览网页与保存数据请运行桌面应用</div>
      <HomeView v-if="contentPage === 'home'" :settings="settings" :recent="recent" @navigate="homeNavigate" @engine="updateEngine" @hide-recent="hideRecent" @clear-history="clearHistory" />
      <BookmarksView v-else-if="contentPage === 'bookmarks'" :folders="folders" :revision="revision" :homepage="active.url === 'sab://home'" @navigate="openBookmark" @create="editBookmark()" @edit="editExistingBookmark" @settings="homeNavigate('sab://settings')" @error="reportComponentError" />
      <SettingsView v-else-if="contentPage === 'settings'" :settings="settings" @save="saveSettingsFromView" @theme="previewTheme" @clear-history="clearHistory" />
      <div v-else-if="pageError || !desktop" class="empty-state"><AppIcon name="globe" :size="36" /><h1>{{ desktop ? '网页暂未打开' : '请在桌面应用中浏览网页' }}</h1><p>{{ pageError || '运行 pnpm tauri:dev 启动轻页 SAB。' }}</p><button class="button secondary" @click="safely(() => navigate('sab://home'))">返回主页</button></div>
    </main>
    <div v-if="message" class="toast" role="status"><AppIcon name="info" :size="16" /><span>{{ message }}</span><button class="icon-button" aria-label="关闭提示" @click="message = ''"><AppIcon name="close" :size="14" /></button></div>

    <OverlayPanel :open="panel !== ''" :label="panel === 'favorites' ? '收藏中心' : panel === 'edit' ? '编辑收藏' : '浏览器菜单'" :compact="panel === 'menu'" @close="safely(closePanel)">
      <p v-if="message" class="panel-error" role="alert">{{ message }}</p>
      <template v-if="panel === 'favorites'">
        <div class="favorites-search"><AppIcon name="search" /><input v-model="favoriteQuery" aria-label="搜索收藏浮层" placeholder="搜索标题、域名、网址或标签" autofocus @keydown="event => safely(() => favoriteKey(event))" /><kbd>Esc</kbd><button class="icon-button" aria-label="关闭收藏浮层" @click="safely(closePanel)"><AppIcon name="close" :size="16" /></button></div>
        <nav class="category-tabs" aria-label="浮层分类"><button :class="{ active: !favoriteCategory }" @click="favoriteCategory = ''">全部</button><button v-for="folder in folders.filter(item => item.count > 0)" :key="folder.id" :class="{ active: favoriteCategory === folder.id }" @click="favoriteCategory = folder.id">{{ folder.name }}</button></nav>
        <div class="favorite-results"><p v-if="!favorites.length" class="empty-inline">{{ favoriteQuery ? '没有找到相关收藏' : '还没有收藏，先保存常用的网站。' }}</p><button v-for="(bookmark, index) in favorites" :id="`favorite-${index}`" :key="bookmark.id" :class="['favorite-result', { highlighted: index === favoriteIndex }]" @focus="favoriteIndex = index" @click="safely(() => navigate(bookmark.url))" @keydown="event => safely(() => favoriteKey(event))"><span class="site-icon"><AppIcon name="globe" /></span><span><strong>{{ bookmark.title }}</strong><small>{{ displayHost(bookmark.url) }}</small></span><span class="folder-label">{{ folders.find(folder => folder.id === bookmark.folderId)?.name }}</span></button></div>
        <footer class="panel-footer"><button class="button secondary" :disabled="internal || !desktop" @click="safely(saveCurrent)">收藏当前页面</button><button class="button primary" @click="safely(() => navigate('sab://bookmarks'))">打开收藏中心</button></footer>
      </template>
      <form v-else-if="panel === 'edit'" class="bookmark-form" @submit.prevent="safely(submitBookmark)"><header><h2>{{ editingId ? '编辑收藏' : '新建收藏' }}</h2><button class="icon-button" type="button" aria-label="关闭编辑" @click="safely(closePanel)"><AppIcon name="close" /></button></header><label class="field-label">网址<input v-model="bookmarkDraft.url" class="text-input" type="url" placeholder="https://example.com" required :readonly="!!editingId" autofocus /></label><label class="field-label">书签名<input v-model="bookmarkDraft.title" class="text-input" required maxlength="512" /></label><label class="field-label">存储目录<select v-model="bookmarkDraft.folderId" class="text-input"><option v-for="folder in folders" :key="folder.id" :value="folder.id">{{ folder.name }}</option></select></label><label class="field-label">标签（逗号分隔）<input v-model="tagInput" class="text-input" placeholder="工作, 参考资料" /></label><label class="setting-row"><span>固定到常用置顶</span><input v-model="bookmarkDraft.pinned" class="switch" type="checkbox" /></label><footer class="panel-footer"><button v-if="editingId" class="button destructive" type="button" :disabled="saving" @click="safely(deleteBookmark)">删除收藏</button><button class="button primary" type="submit" :disabled="saving">{{ saving ? '保存中…' : '保存收藏' }}</button></footer></form>
      <nav v-else-if="panel === 'menu'" class="browser-menu" aria-label="浏览器操作"><button @click="safely(() => navigate('sab://home', true))"><AppIcon name="plus" />新建标签<kbd>⌘T</kbd></button><button @click="safely(() => navigate('sab://home'))"><AppIcon name="home" />主页</button><button @click="safely(() => navigate('sab://bookmarks'))"><AppIcon name="bookmarks" />收藏中心</button><hr /><button @click="safely(() => navigate('sab://settings'))"><AppIcon name="settings" />设置<kbd>⌘,</kbd></button></nav>
    </OverlayPanel>
    <!-- 功能：首次冷启动期间展示 SAB 品牌旋转标记，待本地状态和初始页面就绪后淡出；时间：2026-11-08；作者：lq。 -->
    <Transition name="launch-screen">
      <div v-if="!initialized" class="launch-screen" role="status" aria-live="polite">
        <img class="launch-logo" :src="appLogo" alt="" />
        <strong>轻页 SAB</strong>
        <span>正在准备浏览器…</span>
      </div>
    </Transition>
  </div>
</template>
