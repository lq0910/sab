<script setup lang="ts">
// 功能：连接浏览器框架、本地页面、收藏浮层和受限原生视图；时间：2026-11-08；作者：lq。
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { LogicalPosition } from '@tauri-apps/api/dpi';
import { Menu, MenuItem } from '@tauri-apps/api/menu';
import { getCurrentWebview } from '@tauri-apps/api/webview';
import { getCurrentWindow } from '@tauri-apps/api/window';
import appLogo from '../src-tauri/icons/source.svg';
import AppIcon from './components/AppIcon.vue';
import BookmarksView from './components/BookmarksView.vue';
import HomeView from './components/HomeView.vue';
import HistoryView from './components/HistoryView.vue';
import OverlayPanel from './components/OverlayPanel.vue';
import SettingsView from './components/SettingsView.vue';
// 功能：下载列表复用现有浮层风格；时间：2026-11-14；作者：lq。
import DownloadsView from './components/DownloadsView.vue';
import { defaultSettings, type DownloadItem, type Bookmark, type BookmarkDraft, type ColorMode, type Folder, type PageEvent, type RecentVisit, type SearchEngine, type Settings, type Tab, type ThemeId, type ViewBounds } from './domain/models';
import { displayHost, isInternal, isSearchQuery, resolveAddress } from './domain/navigation';
import { api, desktop } from './services/desktop';
// 功能：复用输入合并与页面状态更新规则；时间：2026-11-14；作者：lq。
import { debounceTask } from './domain/tasks';
import { applyPageEvent } from './domain/pageLifecycle';
// 功能：复用会话序列化、加载计时和快捷键映射；时间：2026-11-14；作者：lq。
import { captureSession, restoreSession, sessionWriter } from './domain/session';
import { loadWatchdog } from './domain/loadWatchdog';
import { keyboardAction } from './domain/shortcuts';

// 功能：在模板条件中显式读取桌面运行状态；时间：2026-11-14；作者：lq。
function isDesktop() { return desktop; }
const newTab = (url = 'sab://home'): Tab => ({ id: crypto.randomUUID(), url, title: '轻页 SAB Workspace', loading: false });
const tabs = ref<Tab[]>([newTab()]);
const activeId = ref(tabs.value[0]!.id);
const active = computed(() => tabs.value.find(tab => tab.id === activeId.value)!);
const address = ref(active.value.url);
const addressElement = ref<HTMLInputElement>();
const addressSuggestions = ref<RecentVisit[]>([]);
const suggestionsOpen = ref(false);
const suggestionIndex = ref(-1);
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
// 功能：错误跟随所属标签保存，切换后仍可重试；时间：2026-11-14；作者：lq。
const pageError = computed(() => active.value.error ?? '');
const initialized = ref(false);
const panel = ref<'' | 'favorites' | 'edit' | 'menu' | 'sleep' | 'downloads' | 'quit'>('');
const contextTabId = ref('');
const tabPickerOpen = ref(false);
const tabPickerQuery = ref('');
const tabsOverflow = ref(false);
const favoriteQuery = ref('');
const favoriteCategory = ref('');
const favorites = ref<Bookmark[]>([]);
const favoriteIndex = ref(0);
const editingId = ref('');
const bookmarkDraft = ref<BookmarkDraft>({ url: '', title: '', folderId: 'inbox', tags: [], pinned: false });
const tagInput = ref('');
const saving = ref(false);
// 功能：统计真实网页视图数量，用于主动释放后台标签；时间：2026-11-14；作者：lq。
const created = ref(new Set<string>());
const sleeping = ref(false);
// 功能：下载面板与后台任务状态同步，仅有活动下载时轮询实际落盘大小；时间：2026-11-14；作者：lq。
const downloads = ref<DownloadItem[]>([]);
const activeDownloads = computed(() => downloads.value.filter(item => item.status === 'downloading'));
let downloadPoll: ReturnType<typeof setInterval> | undefined;
let downloadRequest = 0;
const downloadingTabs = computed(() => new Set(activeDownloads.value.map(item => item.tabId)));
const backgroundViews = computed(() => tabs.value.filter(tab => tab.id !== activeId.value && created.value.has(tab.id) && !downloadingTabs.value.has(tab.id)));
const recentRefresh = debounceTask(() => { void safely(async () => { recent.value = await api.recent(); historyRevision.value++; }); }, 250);
const historyRevision = ref(0);
const historyWrites = new Map<string, ReturnType<typeof debounceTask>>();
let resizeFrame = 0;
const closed = ref<Tab[]>([]);
// 功能：会话写入按顺序执行，初始化期间不覆盖旧会话；时间：2026-11-14；作者：lq。
const writeSession = sessionWriter(api.saveSession);
const sessionTask = debounceTask(() => { void safely(flushSession); }, 300);
let workspaceReady = false;
// 功能：串行消费系统链接，防止初始化和系统事件同时打开重复标签；时间：2026-11-14；作者：lq。
let externalQueue = Promise.resolve();
function openExternalLinks() {
  if (!workspaceReady) return Promise.resolve();
  const pending = externalQueue.then(async () => {
    for (const url of await api.takeExternalLinks()) await navigate(url, true);
  });
  externalQueue = pending.catch(() => {});
  return pending;
}
const disposers: UnlistenFn[] = [];
let nativeTabMenu: Menu | undefined;
let nativeReloadItem: MenuItem | undefined;
let observer: ResizeObserver | undefined;
let nativeQueue = Promise.resolve();
let searchSequence = 0;
const internal = computed(() => isInternal(active.value.url));
// 功能：识别当前 HTTPS 页面并在地址栏展示加密连接标志；时间：2026-11-08；作者：lq。
const secureConnection = computed(() => !internal.value && active.value.url.toLowerCase().startsWith('https://'));
// 功能：根据地址栏输入判断显示搜索或网页连接图标；时间：2026-11-08；作者：lq。
const addressSearch = computed(() => isSearchQuery(address.value));
const filteredTabs = computed(() => {
  const query = tabPickerQuery.value.trim().toLocaleLowerCase();
  if (!query) return tabs.value;
  return tabs.value.filter(tab => `${tab.title} ${tab.url} ${isInternal(tab.url) ? '轻页 SAB' : displayHost(tab.url)}`.toLocaleLowerCase().includes(query));
});
const contentPage = computed(() => active.value.url === 'sab://home'
  ? (settings.value.homeMode === 'minimal' ? 'home' : 'bookmarks')
  : active.value.url === 'sab://bookmarks' ? 'bookmarks' : active.value.url === 'sab://settings' ? 'settings' : active.value.url === 'sab://history' ? 'history' : 'web');

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
  await focusPage();
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
    if (internal.value || panel.value || tabPickerOpen.value || (suggestionsOpen.value && addressSuggestions.value.length) || tab.error) { await api.view('hide', tab.id); return; }
    const exists = created.value.has(tab.id);
    // 功能：休眠标签按需重新创建，不提前恢复其他后台网页；时间：2026-11-14；作者：lq。
    if (!exists) { tab.suspended = false; tab.loading = true; }
    try {
      await api.view(exists ? 'activate' : 'open', tab.id, tab.url, getBounds());
      created.value.add(tab.id);
      if (!exists && tab.zoom && tab.zoom !== 1) await api.view('zoom', tab.id, undefined, undefined, tab.zoom);
    } catch (error) { tab.loading = false; tab.error = String(error); await api.view('hide', tab.id); }
  });
}
async function activate(id: string) {
  const previousIndex = tabs.value.findIndex(tab => tab.id === activeId.value);
  const nextIndex = tabs.value.findIndex(tab => tab.id === id);
  if (id === activeId.value) { await syncView(); await focusPage(); return; }
  tabMotion.value = nextIndex > previousIndex ? 'from-left' : 'from-right';
  activeId.value = id;
  message.value = '';
  await syncView();
  await focusPage();
}
async function navigate(input: string, newWindow = false) {
  const url = resolveAddress(input, settings.value.searchEngine);
  // 功能：下载期间在新标签打开内置页面，保留下载来源网页；时间：2026-11-14；作者：lq。
  if (!newWindow && isInternal(url) && downloadingTabs.value.has(activeId.value)) newWindow = true;
  panel.value = '';
  closeAddressSuggestions();
  tabPickerOpen.value = false;
  message.value = '';
  if (newWindow) { const tab = newTab(url); tabs.value.push(tab); tabMotion.value = 'from-left'; activeId.value = tab.id; }
  const tab = active.value;
  if (isInternal(url) && created.value.has(tab.id)) { await enqueue(() => api.view('close', tab.id)); created.value.delete(tab.id); }
  cancelHistoryWrite(tab.id);
  tab.error = '';
  tab.loadIssue = undefined;
  tab.suspended = false;
  tab.url = url;
  // 功能：为内置页面设置对应标签标题，包含浏览历史入口；时间：2026-11-08；作者：lq。
  tab.title = isInternal(url) ? (url === 'sab://settings' ? '设置 - 轻页 SAB' : url === 'sab://bookmarks' ? '收藏中心' : url === 'sab://history' ? '浏览历史' : '轻页 SAB Workspace') : displayHost(url);
  tab.loading = !isInternal(url) && desktop;
  // 功能：同网址重试也重新计时，不依赖 loading 或 URL 发生变化；时间：2026-11-14；作者：lq。
  if (tab.loading) watchdog.start(tab.id); else watchdog.cancel(tab.id);
  address.value = url;
  // 功能：当前标签改为内置页时销毁原网页，停止后台脚本与媒体；时间：2026-11-14；作者：lq。
  if (isInternal(url)) {
    await syncView(); return;
  }
  await nextTick();
  try {
    await enqueue(async () => {
      await api.view('open', tab.id, url, getBounds());
      created.value.add(tab.id);
      if (tab.zoom && tab.zoom !== 1) await api.view('zoom', tab.id, undefined, undefined, tab.zoom);
      if (tab.id === activeId.value && !panel.value) await api.view('focuspage', tab.id);
    });
  } catch (error) { tab.loading = false; tab.error = String(error); if (tab.id === activeId.value) await syncView(); }
}
async function closeTab(id: string) {
  // 功能：关闭下载来源时直接显示下载面板，使保留原因可见；时间：2026-11-14；作者：lq。
  if (downloadingTabs.value.has(id)) { await showPanel('downloads'); message.value = '该标签正在下载文件，完成后即可关闭'; return; }
  const index = tabs.value.findIndex(tab => tab.id === id);
  if (index < 0) return;
  const tab = tabs.value[index]!;
  cancelHistoryWrite(id);
  if (created.value.has(id)) { await enqueue(() => api.view('close', id)); created.value.delete(id); }
  closed.value.push({ ...tab, loading: false, error: '', suspended: false });
  if (closed.value.length > 20) closed.value.shift();
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
  applyTheme(settings.value.theme, settings.value.colorMode);
  folders.value = state.folders;
  recent.value = await api.recent();
  revision.value++;
  historyRevision.value++;
}
async function saveSettings(value: Settings) {
  await api.saveSettings(value);
  await refreshData();
  message.value = '设置已保存';
}
// 功能：即时应用主题色与明暗模式，并在自动模式下跟随系统配色；时间：2026-11-08；作者：lq。
function applyTheme(theme: ThemeId, colorMode: ColorMode) {
  document.documentElement.dataset.theme = theme;
  document.documentElement.dataset.colorMode = colorMode;
}
// 功能：预览强调色及明暗模式，最终由保存设置持久化；时间：2026-11-08；作者：lq。
function previewTheme(theme: ThemeId) { settings.value.theme = theme; applyTheme(theme, settings.value.colorMode); }
// 功能：即时预览并更新当前明暗偏好；时间：2026-11-08；作者：lq。
function previewColorMode(colorMode: ColorMode) { settings.value.colorMode = colorMode; applyTheme(settings.value.theme, colorMode); }
async function changeEngine(engine: SearchEngine) { await saveSettings({ ...settings.value, searchEngine: engine }); }
// 功能：清空历史后刷新所有依赖历史记录的界面状态；时间：2026-11-08；作者：lq。
async function clearHistory() { historyWrites.forEach(task => task.cancel()); historyWrites.clear(); recentRefresh.cancel(); await api.clearHistory(); await refreshData(); addressSuggestions.value = []; message.value = '浏览历史已删除'; }
async function hideRecent(url: string) { await api.hideRecent(url); recent.value = await api.recent(); }

// 功能：按地址栏内容查询最近匹配的网址，支持空输入时展示最近访问；时间：2026-11-08；作者：lq。
let suggestionRequest = 0;
// 功能：地址输入停顿后查询，减少跨进程调用与数据库扫描；时间：2026-11-14；作者：lq。
const suggestionTask = debounceTask(() => { void loadAddressSuggestions(); });
// 功能：收起地址联想并作废尚未返回的查询；时间：2026-11-08；作者：lq。
function closeAddressSuggestions() {
  suggestionsOpen.value = false;
  suggestionTask.cancel();
  suggestionRequest++;
}
// 功能：清空地址栏时收起联想，输入非空内容时继续搜索历史；时间：2026-11-08；作者：lq。
function handleAddressInput() {
  suggestionIndex.value = -1;
  if (!address.value.trim()) {
    addressSuggestions.value = [];
    closeAddressSuggestions();
    return;
  }
  suggestionsOpen.value = true;
  suggestionRequest++;
  addressSuggestions.value = [];
  suggestionTask.schedule();
}
// 功能：点击浏览器其他区域时同时关闭地址联想和标签搜索列表；时间：2026-11-08；作者：lq。
function closeTransientPanels() {
  closeAddressSuggestions();
  void safely(closeTabPicker);
}
async function loadAddressSuggestions() {
  if (!suggestionsOpen.value) return;
  const sequence = ++suggestionRequest;
  try {
    const result = await api.history(address.value.trim().slice(0, 512), 0, 8);
    if (sequence === suggestionRequest && suggestionsOpen.value) addressSuggestions.value = result.items;
  } catch (error) { message.value = String(error); }
}
// 功能：选择地址栏历史联想并打开对应网址；时间：2026-11-08；作者：lq。
function chooseAddressSuggestion(visit: RecentVisit) { void safely(() => navigate(visit.url)); }
// 功能：支持方向键选择历史联想、回车打开及 Escape 收起；时间：2026-11-08；作者：lq。
function addressSuggestionKey(event: KeyboardEvent) {
  if (event.isComposing) return;
  if (event.key === 'ArrowDown' && addressSuggestions.value.length) {
    event.preventDefault(); suggestionIndex.value = (suggestionIndex.value + 1) % addressSuggestions.value.length;
  } else if (event.key === 'ArrowUp' && addressSuggestions.value.length) {
    event.preventDefault(); suggestionIndex.value = (suggestionIndex.value - 1 + addressSuggestions.value.length) % addressSuggestions.value.length;
  } else if (event.key === 'Enter' && suggestionIndex.value >= 0 && addressSuggestions.value[suggestionIndex.value]) {
    event.preventDefault(); chooseAddressSuggestion(addressSuggestions.value[suggestionIndex.value]!);
  } else if (event.key === 'Escape') closeAddressSuggestions();
}
// 功能：删除单条记录后同步主页、地址栏和历史页面的数据；时间：2026-11-08；作者：lq。
async function deleteHistoryEntry(url: string) {
  await api.deleteHistory(url);
  await refreshData();
  if (suggestionsOpen.value) await loadAddressSuggestions();
}

// 功能：先隐藏原生网页再显示 HTML 浮层，关闭后恢复原标签；时间：2026-11-08；作者：lq。
async function showPanel(value: typeof panel.value) {
  if (desktop) await enqueue(() => api.view('hide', active.value.id));
  message.value = '';
  panel.value = value;
}
async function closePanel() { panel.value = ''; await syncView(); await focusPage(); }
async function editBookmark(bookmark?: Bookmark) {
  editingId.value = bookmark?.id ?? '';
  Object.assign(bookmarkDraft.value, bookmark ?? { url: internal.value ? '' : active.value.url, title: internal.value ? '' : active.value.title, folderId: 'inbox', tags: [], pinned: false });
  tagInput.value = bookmarkDraft.value.tags.join(', ');
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
    await api.saveBookmark({ ...bookmarkDraft.value, tags: tagInput.value.split(/[,，]/).map(t => t.trim()).filter(Boolean) });
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
// 功能：导入完成后刷新收藏分类计数并向用户显示新增和跳过数量；时间：2026-11-08；作者：lq。
function importedBookmarks(result: { imported: number; skipped: number; folderMetadataUpdated: number }) {
  void safely(async () => { await refreshData(); message.value = `书签导入完成：新增 ${result.imported} 条，跳过 ${result.skipped} 条${result.folderMetadataUpdated ? `，补回 ${result.folderMetadataUpdated} 条原目录线索` : ''}`; });
}
// 功能：规则整理完成或撤销后刷新收藏分类数量；时间：2026-11-08；作者：lq。
function organizationCompleted() { void safely(refreshData); }
function updateEngine(value: SearchEngine) { void safely(() => changeEngine(value)); }

// 功能：聚焦外壳地址栏或当前网页，不向远程页面开放任何 IPC；时间：2026-11-14；作者：lq。
async function focusPage() {
  if (desktop && !internal.value && !panel.value && !active.value.error && created.value.has(activeId.value)) await api.view('focuspage', activeId.value);
}
async function focusAddress() {
  panel.value = '';
  tabPickerOpen.value = false;
  closeAddressSuggestions();
  if (desktop) await api.view('focusshell', activeId.value);
  await syncView();
  await nextTick();
  addressElement.value?.focus();
  addressElement.value?.select();
}
async function reopenClosedTab() {
  const tab = closed.value.pop();
  if (!tab) { message.value = '没有可恢复的标签'; return; }
  await navigate(tab.url, true);
  active.value.zoom = tab.zoom ?? 1;
  if (desktop && !internal.value && created.value.has(activeId.value)) await api.view('zoom', activeId.value, undefined, undefined, active.value.zoom);
}
async function dispatchShortcut(action: string) {
  if (!initialized.value) return;
  switch (action) {
    case 'focus-address': await focusAddress(); break;
    case 'new-tab': await navigate('sab://home', true); await focusAddress(); break;
    case 'close-tab': if (panel.value) await closePanel(); else await closeTab(activeId.value); break;
    case 'restore-tab': await reopenClosedTab(); break;
    case 'next-tab': case 'previous-tab': {
      await closePanel(); tabPickerOpen.value = false; closeAddressSuggestions();
      const index = tabs.value.findIndex(tab => tab.id === activeId.value);
      await activate(tabs.value[(index + (action === 'next-tab' ? 1 : -1) + tabs.value.length) % tabs.value.length]!.id);
      break;
    }
    case 'tab-picker': await openTabPicker(); break;
    case 'save-bookmark': await saveCurrent(); break;
    case 'bookmarks': await showPanel('favorites'); break;
    case 'history': await navigate('sab://history'); break;
    case 'settings': await navigate('sab://settings'); break;
    case 'downloads': await showPanel('downloads'); break;
    case 'reload': if (!internal.value) await reloadTab(activeId.value); break;
    case 'zoom-in': if (!internal.value) await changeZoom(0.1); break;
    case 'zoom-out': if (!internal.value) await changeZoom(-0.1); break;
    case 'zoom-reset': if (!internal.value) await changeZoom(0); break;
    case 'quit': await controlWindow('close'); break;
  }
}
// 功能：桌面快捷键只走原生菜单；网页预览沿用相同动作，输入法组合时不触发；时间：2026-11-14；作者：lq。
function keyboard(event: KeyboardEvent) {
  if (event.key === 'Escape' && tabPickerOpen.value) { void safely(closeTabPicker); return; }
  const action = keyboardAction(event);
  if (!action) return;
  if (!desktop) { event.preventDefault(); void safely(() => dispatchShortcut(action)); }
}
// 功能：退出时等待最终会话写入，再关闭原生窗口；时间：2026-11-14；作者：lq。
async function flushSession() {
  sessionTask.cancel();
  if (!desktop || !workspaceReady || !settings.value.restoreSession) return;
  await writeSession(captureSession(tabs.value, activeId.value, closed.value));
}
async function quitBrowser() { await flushSession(); await getCurrentWindow().destroy(); }
watch(() => captureSession(tabs.value, activeId.value, closed.value), () => {
  if (workspaceReady && settings.value.restoreSession) sessionTask.schedule();
}, { deep: true });
// 功能：仅对仍在加载的标签提示超时，页面最终加载成功后可自行恢复；时间：2026-11-14；作者：lq。
const watchdog = loadWatchdog(id => {
  const tab = tabs.value.find(tab => tab.id === id);
  if (!tab?.loading || tab.suspended || isInternal(tab.url)) return;
  tab.loadIssue = 'timeout';
  tab.error = '等待已超过 30 秒。可以继续等待，或检查网络后重新尝试。';
  if (id === activeId.value) void safely(syncView);
});
watch(() => tabs.value.map(tab => ({ id: tab.id, url: tab.url, loading: tab.loading })), (current, previous) => {
  for (const tab of previous ?? []) if (!current.some(item => item.id === tab.id)) watchdog.cancel(tab.id);
  for (const tab of current) {
    const old = previous?.find(item => item.id === tab.id);
    if (!tab.loading) watchdog.cancel(tab.id);
    else if (!old?.loading || old.url !== tab.url) { watchdog.start(tab.id); if (!navigator.onLine) reportOffline(); }
  }
}, { deep: true });
function reportOffline() {
  for (const tab of tabs.value) {
    if (!tab.loading || isInternal(tab.url)) continue;
    tab.loadIssue = 'offline'; tab.error = '设备当前显示为离线。请连接网络后重新尝试，也可以继续等待。';
  }
  if (active.value.loadIssue) void safely(syncView);
}
function reportOnline() { message.value = '设备已连接网络，可重新尝试打开网页'; }
async function continueWaiting() {
  active.value.error = ''; active.value.loadIssue = undefined;
  if (active.value.loading) watchdog.start(activeId.value);
  await syncView(); await focusPage();
}
// 功能：切换同网址标签时也恢复真实地址，避免沿用另一个标签的未提交输入；时间：2026-11-14；作者：lq。
watch(() => [activeId.value, active.value.url], () => { address.value = active.value.url; });
// 功能：仅在收藏浮层可见时检索，并取消被快速输入替代的查询；时间：2026-11-14；作者：lq。
const favoriteTask = debounceTask(() => { void safely(async () => {
  const sequence = searchSequence;
  const page = await api.bookmarks(favoriteQuery.value, favoriteCategory.value);
  if (sequence === searchSequence && panel.value === 'favorites') { favorites.value = page.items; favoriteIndex.value = 0; }
}); });
watch([favoriteQuery, favoriteCategory, revision, panel], () => {
  searchSequence++;
  favoriteTask.cancel();
  if (panel.value === 'favorites') favoriteTask.schedule();
});
// 功能：地址建议浮层显示时避让原生网页，关闭后恢复；时间：2026-11-14；作者：lq。
watch(() => suggestionsOpen.value && addressSuggestions.value.length > 0, () => { void safely(syncView); });

// 功能：按标签合并加载完成与随后到达的标题事件，只刷新历史相关数据；时间：2026-11-14；作者：lq。
function cancelHistoryWrite(id: string) { historyWrites.get(id)?.cancel(); historyWrites.delete(id); }
function scheduleHistoryWrite(tab: Tab) {
  cancelHistoryWrite(tab.id);
  const url = tab.url;
  const task = debounceTask(() => {
    historyWrites.delete(tab.id);
    if (!tabs.value.includes(tab) || tab.url !== url || tab.loading || tab.suspended) return;
    void safely(async () => { await api.recordVisit(url, tab.title); recentRefresh.schedule(); });
  }, 300);
  historyWrites.set(tab.id, task);
  task.schedule();
}
// 功能：停止当前加载或重新尝试打开休眠、失败的标签；时间：2026-11-14；作者：lq。
async function reloadTab(id: string) {
  const tab = tabs.value.find(item => item.id === id);
  if (!tab || isInternal(tab.url)) return;
  if (id !== activeId.value) await activate(id);
  if (!created.value.has(id) || tab.error) { await navigate(tab.url); return; }
  tab.loading = true;
  // 功能：加载中的同页刷新会替换请求，超时提示从本次刷新重新计时；时间：2026-11-14；作者：lq。
  watchdog.start(id);
  try { await enqueue(() => api.view('reload', id)); }
  catch (error) { tab.loading = false; tab.error = String(error); }
}
async function stopOrReload() {
  const tab = active.value;
  if (!tab.loading) { await reloadTab(tab.id); return; }
  await enqueue(() => api.view('stop', tab.id));
  tab.loading = false;
  cancelHistoryWrite(tab.id);
}
// 功能：通过原生 WebView 缩放当前网页，按标签保留比例；时间：2026-11-14；作者：lq。
async function changeZoom(delta: number) {
  const tab = active.value;
  if (internal.value || !created.value.has(tab.id)) return;
  const zoom = delta === 0 ? 1 : Math.min(2, Math.max(0.5, Math.round(((tab.zoom ?? 1) + delta) * 10) / 10));
  await enqueue(() => api.view('zoom', tab.id, undefined, undefined, zoom));
  tab.zoom = zoom;
}
// 功能：经用户确认后释放后台网页，保留网址标题并在再次选择时重新加载；时间：2026-11-14；作者：lq。
async function suspendBackgroundTabs() {
  if (sleeping.value) return;
  sleeping.value = true;
  let count = 0;
  try {
    for (const tab of [...backgroundViews.value]) {
      await enqueue(() => api.view('close', tab.id));
      created.value.delete(tab.id);
      cancelHistoryWrite(tab.id);
      tab.suspended = true;
      tab.loading = false;
      count++;
    }
    await closePanel();
    message.value = `已休眠 ${count} 个后台网页，选择标签时重新加载`;
  } finally { sleeping.value = false; }
}
// 功能：完成/失败立即更新面板，后台只在下载中每秒采样一次；时间：2026-11-14；作者：lq。
async function refreshDownloads() {
  const sequence = ++downloadRequest;
  const items = await api.downloads();
  if (sequence !== downloadRequest) return;
  const previous = new Map(downloads.value.map(item => [item.id, item.status]));
  downloads.value = items;
  if (items.some(item => previous.get(item.id) === 'downloading' && item.status !== 'downloading')) {
    message.value = items.some(item => previous.get(item.id) === 'downloading' && item.status === 'failed') ? '下载失败，请在下载面板中重试' : '下载完成，可在下载面板打开文件';
  }
  // 功能：页面响应转为文件下载后结束标签加载标记，避免停止按钮影响下载来源；时间：2026-11-14；作者：lq。
  const started = items.filter(item => !previous.has(item.id) && item.status === 'downloading');
  for (const item of started) { const tab = tabs.value.find(tab => tab.id === item.tabId); if (tab) { tab.loading = false; tab.error = ''; tab.loadIssue = undefined; cancelHistoryWrite(tab.id); } }
  if (started.length) { await showPanel('downloads'); }
  clearInterval(downloadPoll);
  if (activeDownloads.value.length) downloadPoll = setInterval(() => { void safely(refreshDownloads); }, 1000);
}
async function clearDownloadRecords() { await api.clearDownloads(); await refreshDownloads(); }
// 功能：重新发起 HTTP 下载；临时 Blob 链接返回来源页重新生成；时间：2026-11-14；作者：lq。
async function retryDownload(item: DownloadItem) {
  const url = /^https?:/i.test(item.url) ? item.url : item.sourceUrl;
  if (!/^https?:/i.test(url)) { message.value = '该下载链接已失效，请回到原网站重新下载'; return; }
  await navigate(url, true);
}
// 功能：窗口连续缩放时每帧最多调整一次网页尺寸，不重新激活所有标签；时间：2026-11-14；作者：lq。
function scheduleResize() {
  if (resizeFrame) return;
  resizeFrame = requestAnimationFrame(() => {
    resizeFrame = 0;
    void measureTabsOverflow();
    if (desktop && initialized.value && created.value.has(activeId.value) && !internal.value && !panel.value && !tabPickerOpen.value) {
      const id = activeId.value;
      void safely(() => enqueue(async () => { if (id === activeId.value && created.value.has(id)) await api.view('resize', id, undefined, getBounds()); }));
    }
  });
}

onMounted(() => { void safely(async () => {
  window.addEventListener('keydown', keyboard);
  window.addEventListener('offline', reportOffline);
  window.addEventListener('online', reportOnline);
  try {
  if (desktop) {
    // 功能：注册系统原生标签菜单及刷新、复制、关闭等操作；时间：2026-11-08；作者：lq。
    nativeReloadItem = await MenuItem.new({ id: 'reload', text: '重新加载', action: () => runTabMenu(reloadTab) });
    nativeTabMenu = await Menu.new({ items: [
      nativeReloadItem,
      { id: 'duplicate', text: '复制标签页', action: () => runTabMenu(duplicateTab) },
      { id: 'copy-address', text: '复制网址', action: () => runTabMenu(copyTabAddress) },
      { item: 'Separator' },
      { id: 'close-others', text: '关闭其他标签页', action: () => runTabMenu(closeOtherTabs) },
      { id: 'close', text: '关闭标签页', action: () => runTabMenu(closeTab) },
    ] });
    disposers.push(await listen<string>('sab:shortcut', event => { void safely(() => dispatchShortcut(event.payload)); }));
    // 功能：系统关闭、快捷键退出和自绘关闭按钮共用会话保存与下载确认；时间：2026-11-14；作者：lq。
    disposers.push(await getCurrentWindow().onCloseRequested(event => {
      event.preventDefault();
      void safely(async () => { if (activeDownloads.value.length) await showPanel('quit'); else await quitBrowser(); });
    }));
    const webview = getCurrentWebview();
    disposers.push(await webview.listen<PageEvent>('sab:page', event => {
      const tab = tabs.value.find(item => item.id === event.payload.tabId);
      if (!tab || isInternal(tab.url)) return;
      const hadError = !!tab.error;
      if (applyPageEvent(tab, event.payload)) scheduleHistoryWrite(tab);
      if (hadError && !tab.error && tab.id === activeId.value) void safely(syncView);
    }));
    disposers.push(await listen('sab:downloads', () => { void safely(refreshDownloads); }));
    disposers.push(await listen<string>('sab:download-notice', event => { void safely(async () => { await showPanel('downloads'); message.value = event.payload; }); }));
    disposers.push(await listen<PageEvent>('sab:new-tab', event => { void safely(() => navigate(event.payload.url, true)); }));
    disposers.push(await listen('sab:external-links', () => { void safely(openExternalLinks); }));
  }
  await refreshData();
  await refreshDownloads();
  observer = new ResizeObserver(scheduleResize);
  if (viewport.value) observer.observe(viewport.value);
  if (tabsElement.value) observer.observe(tabsElement.value);
  await measureTabsOverflow();
  // 功能：指定启动页优先；否则恢复上次标签，后台网页暂不创建视图；时间：2026-11-14；作者：lq。
  if (settings.value.startupUrl) await navigate(settings.value.startupUrl);
  else if (desktop && settings.value.restoreSession) {
    try {
      const snapshot = await api.loadSession();
      if (snapshot) { const restored = restoreSession(snapshot); tabs.value = restored.tabs; activeId.value = restored.activeId; closed.value = restored.closed; address.value = active.value.url; }
    } catch (error) { message.value = String(error); }
  }
  workspaceReady = true;
  } finally {
    // 功能：初始化完成立即交互，取消人为启动等待并同步指定启动页；时间：2026-11-14；作者：lq。
    initialized.value = true;
    await syncView();
    await openExternalLinks();
    if (workspaceReady && settings.value.restoreSession) sessionTask.schedule();
  }
}); });
// 功能：卸载时清理排队输入、历史写入与动画帧；时间：2026-11-14；作者：lq。
onUnmounted(() => { watchdog.clear(); sessionTask.cancel(); window.removeEventListener('offline', reportOffline); window.removeEventListener('online', reportOnline); clearInterval(downloadPoll); downloadRequest++; suggestionTask.cancel(); favoriteTask.cancel(); recentRefresh.cancel(); historyWrites.forEach(task => task.cancel()); cancelAnimationFrame(resizeFrame); observer?.disconnect(); disposers.forEach(stop => stop()); window.removeEventListener('keydown', keyboard); void nativeTabMenu?.close(); void nativeReloadItem?.close(); });
// 功能：只在标签宽度相关内容变化时测量，避免加载状态触发深度观察；时间：2026-11-14；作者：lq。
watch(() => tabs.value.map(tab => `${tab.id}:${tab.title}`), () => { void measureTabsOverflow(); });
</script>

<template>
  <div class="browser-shell" @click="closeTransientPanels">
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
        <button class="icon-button" aria-label="后退" :disabled="isInternal(active.url) || !isDesktop()" @click="safely(() => api.view('back', activeId))"><AppIcon name="back" /></button>
        <button class="icon-button" aria-label="前进" :disabled="isInternal(active.url) || !isDesktop()" @click="safely(() => api.view('forward', activeId))"><AppIcon name="forward" /></button>
        <button class="icon-button" :aria-label="active.loading ? '停止加载' : '刷新'" :title="active.loading ? '停止加载' : '刷新'" :disabled="isInternal(active.url) || !isDesktop()" @click="safely(stopOrReload)"><AppIcon :name="active.loading ? 'close' : 'refresh'" /></button>
        <div class="address-wrap" @click.stop>
          <form class="address-bar" @submit.prevent="safely(() => navigate(address))"><span class="address-security-icon" :class="{ secured: secureConnection && !addressSearch }" :title="addressSearch ? '搜索内容' : secureConnection ? 'HTTPS 加密连接' : (internal ? '轻页 SAB 内置页面' : '普通连接')" :aria-label="addressSearch ? '搜索内容' : secureConnection ? 'HTTPS 加密连接' : (internal ? '轻页 SAB 内置页面' : '普通连接')"><AppIcon :name="addressSearch ? 'search' : internal ? 'settings' : secureConnection ? 'lock' : 'globe'" :size="15" /></span><input ref="addressElement" v-model="address" aria-label="地址或搜索" spellcheck="false" autocomplete="off" role="combobox" aria-autocomplete="list" :aria-expanded="suggestionsOpen" :aria-controls="suggestionsOpen && addressSuggestions.length ? 'address-suggestions' : undefined" :aria-activedescendant="suggestionIndex >= 0 ? `address-option-${suggestionIndex}` : undefined" @focus="suggestionsOpen = true; suggestionIndex = -1; loadAddressSuggestions()" @input="handleAddressInput" @keydown="addressSuggestionKey" @blur="closeAddressSuggestions" /><kbd>⌘L</kbd></form>
          <section v-if="suggestionsOpen && addressSuggestions.length" id="address-suggestions" class="address-suggestions" role="listbox" aria-label="浏览历史建议">
            <button v-for="(visit, index) in addressSuggestions" :key="visit.url" :id="`address-option-${index}`" :class="['address-suggestion', { selected: suggestionIndex === index }]" role="option" :aria-selected="suggestionIndex === index" @mousedown.prevent="chooseAddressSuggestion(visit)"><AppIcon name="clock" :size="15" /><span><strong>{{ visit.title }}</strong><small>{{ visit.url }}</small></span><kbd>{{ new Date(visit.visitedAt).toLocaleDateString() }}</kbd></button>
          </section>
        </div>
        <button class="icon-button" aria-label="收藏当前页面" title="收藏当前页面 (⌘/Ctrl+D)" :disabled="isInternal(active.url) || !isDesktop()" @click="safely(saveCurrent)"><AppIcon name="star" /></button>
        <button class="icon-button" aria-label="打开收藏中心浮层" title="收藏中心 (⌘/Ctrl+Shift+O)" @click="safely(() => showPanel('favorites'))"><AppIcon name="bookmarks" /></button>
        <!-- 功能：下载始终可达，活动任务显示数量且使用原生浮层避让；时间：2026-11-14；作者：lq。 -->
        <button class="icon-button download-trigger" :aria-label="activeDownloads.length ? `下载中 ${activeDownloads.length} 个文件` : '下载管理'" title="下载管理" @click="safely(() => showPanel('downloads'))"><AppIcon name="download" /><span v-if="activeDownloads.length" class="download-count">{{ activeDownloads.length }}</span></button>
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
          <button class="tab-picker-select" :aria-label="`切换到 ${tab.title}`" @click="safely(() => chooseTab(tab.id))"><span class="tab-picker-icon"><AppIcon :name="isInternal(tab.url) ? 'grid' : 'globe'" :size="17" /></span><span class="tab-picker-copy"><strong>{{ tab.loading ? '加载中…' : tab.title }}</strong><small>{{ tab.suspended ? '已休眠 · 选择后重新加载' : isInternal(tab.url) ? '轻页 SAB' : displayHost(tab.url) }}</small></span></button>
          <button class="tab-picker-close" :aria-label="`关闭 ${tab.title}`" @click="safely(() => closeTabFromPicker(tab.id))"><AppIcon name="close" :size="16" /></button>
        </div>
      </div>
    </section>
    <main ref="viewport" class="content-viewport">
      <div v-if="!isDesktop()" class="preview-notice">界面预览 · 浏览网页与保存数据请运行桌面应用</div>
      <HomeView v-if="contentPage === 'home'" :settings="settings" :recent="recent" @navigate="homeNavigate" @engine="updateEngine" @hide-recent="hideRecent" @clear-history="clearHistory" />
      <HistoryView v-else-if="contentPage === 'history'" :revision="historyRevision" @navigate="openBookmark" @delete="deleteHistoryEntry" @clear="clearHistory" @error="reportComponentError" />
      <BookmarksView v-else-if="contentPage === 'bookmarks'" :folders="folders" :revision="revision" :homepage="active.url === 'sab://home'" @navigate="openBookmark" @create="editBookmark()" @edit="editExistingBookmark" @settings="homeNavigate('sab://settings')" @organized="organizationCompleted" @folders-changed="organizationCompleted" @error="reportComponentError" />
      <SettingsView v-else-if="contentPage === 'settings'" :settings="settings" @save="saveSettingsFromView" @theme="previewTheme" @color-mode="previewColorMode" @clear-history="clearHistory" @imported="importedBookmarks" />
      <div v-else-if="pageError || !isDesktop()" class="empty-state"><AppIcon name="globe" :size="36" /><h1>{{ !isDesktop() ? '请在桌面应用中浏览网页' : active.loadIssue === 'timeout' ? '网页加载时间较长' : active.loadIssue === 'offline' ? '设备当前离线' : '网页暂未打开' }}</h1><p>{{ pageError || '运行 pnpm tauri:dev 启动轻页 SAB。' }}</p><button class="button secondary" @click="safely(() => navigate('sab://home'))">返回主页</button><button v-if="isDesktop()" class="button primary" @click="safely(() => navigate(active.url))">重新尝试</button><button v-if="active.loadIssue && active.loading" class="button secondary" @click="safely(continueWaiting)">继续等待</button></div>
    </main>
    <div v-if="message" class="toast" role="status"><AppIcon name="info" :size="16" /><span>{{ message }}</span><button class="icon-button" aria-label="关闭提示" @click="message = ''"><AppIcon name="close" :size="14" /></button></div>

    <OverlayPanel :open="panel !== ''" :label="panel === 'favorites' ? '收藏中心' : panel === 'edit' ? '编辑收藏' : panel === 'sleep' ? '休眠后台标签' : panel === 'downloads' ? '下载管理' : panel === 'quit' ? '退出浏览器' : '浏览器菜单'" :compact="panel === 'menu'" @close="safely(closePanel)">
      <p v-if="message" class="panel-error" role="alert">{{ message }}</p>
      <template v-if="panel === 'favorites'">
        <div class="favorites-search"><AppIcon name="search" /><input v-model="favoriteQuery" aria-label="搜索收藏浮层" placeholder="搜索标题、域名、网址或标签" autofocus @keydown="event => safely(() => favoriteKey(event))" /><kbd>Esc</kbd><button class="icon-button" aria-label="关闭收藏浮层" @click="safely(closePanel)"><AppIcon name="close" :size="16" /></button></div>
        <nav class="category-tabs" aria-label="浮层分类"><button :class="{ active: !favoriteCategory }" @click="favoriteCategory = ''">全部</button><button v-for="folder in folders.filter(item => item.count > 0)" :key="folder.id" :class="{ active: favoriteCategory === folder.id }" @click="favoriteCategory = folder.id">{{ folder.name }}</button></nav>
        <div class="favorite-results"><p v-if="!favorites.length" class="empty-inline">{{ favoriteQuery ? '没有找到相关收藏' : '还没有收藏，先保存常用的网站。' }}</p><button v-for="(bookmark, index) in favorites" :id="`favorite-${index}`" :key="bookmark.id" :class="['favorite-result', { highlighted: index === favoriteIndex }]" @focus="favoriteIndex = index" @click="safely(() => navigate(bookmark.url))" @keydown="event => safely(() => favoriteKey(event))"><span class="site-icon"><AppIcon name="globe" /></span><span><strong>{{ bookmark.title }}</strong><small>{{ displayHost(bookmark.url) }}</small></span><span class="folder-label">{{ folders.find(folder => folder.id === bookmark.folderId)?.name }}</span></button></div>
        <footer class="panel-footer"><button class="button secondary" :disabled="isInternal(active.url) || !isDesktop()" @click="safely(saveCurrent)">收藏当前页面</button><button class="button primary" @click="safely(() => navigate('sab://bookmarks'))">打开收藏中心</button></footer>
      </template>
      <form v-else-if="panel === 'edit'" class="bookmark-form" @submit.prevent="safely(submitBookmark)"><header><h2>{{ editingId ? '编辑收藏' : '新建收藏' }}</h2><button class="icon-button" type="button" aria-label="关闭编辑" @click="safely(closePanel)"><AppIcon name="close" /></button></header><label class="field-label">网址<input v-model="bookmarkDraft.url" class="text-input" type="url" placeholder="https://example.com" required :readonly="!!editingId" autofocus /></label><label class="field-label">书签名<input v-model="bookmarkDraft.title" class="text-input" required maxlength="512" /></label><label class="field-label">存储目录<select v-model="bookmarkDraft.folderId" class="text-input"><option v-for="folder in folders" :key="folder.id" :value="folder.id">{{ folder.name }}</option></select></label><label class="field-label">标签（逗号分隔）<input v-model="tagInput" class="text-input" placeholder="工作, 参考资料" /></label><label class="setting-row"><span>固定到常用置顶</span><input v-model="bookmarkDraft.pinned" class="switch" type="checkbox" /></label><footer class="panel-footer"><button v-if="editingId" class="button destructive" type="button" :disabled="saving" @click="safely(deleteBookmark)">删除收藏</button><button class="button primary" type="submit" :disabled="saving">{{ saving ? '保存中…' : '保存收藏' }}</button></footer></form>
      <!-- 功能：文件打开、定位、重试和仅清除记录形成完整下载闭环；时间：2026-11-14；作者：lq。 -->
      <DownloadsView v-else-if="panel === 'downloads'" :items="downloads" @close="safely(closePanel)" @clear="safely(clearDownloadRecords)" @open="id => safely(() => api.openDownload(id))" @reveal="id => safely(() => api.openDownload(id, true))" @retry="item => safely(() => retryDownload(item))" />
      <!-- 功能：退出前确认未完成任务，用户可以继续下载或明确中断退出；时间：2026-11-14；作者：lq。 -->
      <section v-else-if="panel === 'quit'" class="sleep-confirm"><h2>还有 {{ activeDownloads.length }} 个下载未完成</h2><p>退出浏览器会中断下载。已完成文件会保留；下次启动可查看中断记录并重新下载。</p><footer class="panel-footer"><button class="button primary" @click="panel = 'downloads'">继续下载</button><button class="button destructive" @click="safely(quitBrowser)">退出浏览器</button></footer></section>
      <!-- 功能：主动释放内存前说明会丢失网页临时状态；时间：2026-11-14；作者：lq。 -->
      <section v-else-if="panel === 'sleep'" class="sleep-confirm"><h2>休眠后台标签</h2><p>将释放 {{ backgroundViews.length }} 个后台网页，保留标签标题与网址。再次选择时重新加载。</p><p>后台页面中未提交的表单、滚动位置和网页内的前进后退记录会丢失，音视频会停止。正在下载的标签会保留。</p><footer class="panel-footer"><button class="button secondary" :disabled="sleeping" @click="safely(closePanel)">取消</button><button class="button primary" :disabled="sleeping || !backgroundViews.length" @click="safely(suspendBackgroundTabs)">{{ sleeping ? '正在释放…' : '确认休眠' }}</button></footer></section>
      <nav v-else-if="panel === 'menu'" class="browser-menu" aria-label="浏览器操作"><div v-if="isDesktop()" class="resource-summary">网页视图 {{ created.size }} 个 · 已休眠 {{ tabs.filter(tab => tab.suspended).length }} 个</div><div v-if="!isInternal(active.url) && isDesktop()" class="page-zoom" aria-label="网页缩放"><span>缩放</span><button aria-label="缩小网页" :disabled="(active.zoom ?? 1) <= 0.5" @click="safely(() => changeZoom(-0.1))">−</button><button aria-label="重置网页缩放" @click="safely(() => changeZoom(0))">{{ Math.round((active.zoom ?? 1) * 100) }}%</button><button aria-label="放大网页" :disabled="(active.zoom ?? 1) >= 2" @click="safely(() => changeZoom(0.1))">+</button></div><button v-if="isDesktop()" :disabled="!backgroundViews.length" @click="panel = 'sleep'"><AppIcon name="clock" />休眠后台标签</button><hr /><button @click="safely(() => navigate('sab://home', true))"><AppIcon name="plus" />新建标签<kbd>⌘T</kbd></button><button @click="safely(() => navigate('sab://home'))"><AppIcon name="home" />主页</button><button @click="safely(() => navigate('sab://bookmarks'))"><AppIcon name="bookmarks" />收藏中心</button><button @click="safely(() => navigate('sab://history'))"><AppIcon name="clock" />浏览历史<kbd>⌘Y</kbd></button><hr /><button @click="safely(() => navigate('sab://settings'))"><AppIcon name="settings" />设置<kbd>⌘,</kbd></button></nav>
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
