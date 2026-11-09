<script setup lang="ts">
// 功能：按 chrome 设置页稿实现启动、搜索和本地偏好，未实现能力明确标记；时间：2026-11-08；作者：lq。
import { onMounted, onUnmounted, reactive, ref, watch } from 'vue';
import type { BookmarkDraft, BookmarkImportResult, ColorMode, Settings, ThemeId } from '../domain/models';
import { engines } from '../domain/navigation';
import { api, desktop } from '../services/desktop';
import AppIcon from './AppIcon.vue';
import officialAccountQr from '../../docs/qrcode_for_gh_554a10e3d22b_258.jpg';
const props = defineProps<{ settings: Settings }>();
const emit = defineEmits<{ save: [value: Settings]; clearHistory: []; theme: [value: ThemeId]; colorMode: [value: ColorMode]; imported: [value: BookmarkImportResult] }>();
const draft = reactive({ ...props.settings });
const activeSection = ref('appearance');
// 功能：跟踪 HTML 书签文件选择、导入进度和结果提示；时间：2026-11-08；作者：lq。
const importInput = ref<HTMLInputElement>();
const importing = ref(false);
const importMessage = ref('');
let sectionObserver: IntersectionObserver | undefined;
watch(() => props.settings, value => Object.assign(draft, value), { deep: true });
const sections = [
  { id: 'appearance', title: '外观与主题', icon: 'palette' },
  { id: 'startup', title: '启动行为与主页', icon: 'home' },
  { id: 'search', title: '搜索引擎', icon: 'search' },
  { id: 'import', title: '书签迁移与导入', icon: 'download' },
  { id: 'default', title: '默认浏览器', icon: 'shield' },
  { id: 'privacy', title: '隐私与本地数据', icon: 'shield' },
  { id: 'about', title: '关于轻页 SAB', icon: 'info' },
];
// 功能：展示自动、浅色和深色三种可选的界面模式；时间：2026-11-08；作者：lq。
const colorModes: { id: ColorMode; name: string; description: string }[] = [
  { id: 'auto', name: '自动', description: '跟随系统外观' },
  { id: 'light', name: '浅色', description: '使用明亮界面' },
  { id: 'dark', name: '深色', description: '使用深色界面' },
];
// 功能：列出精心配好的深色主题色板；时间：2026-11-08；作者：lq。
const themes: { id: ThemeId; name: string; description: string; color: string; companion: string }[] = [
  { id: 'obsidian', name: '黑曜', description: '黑色主调 · 默认', color: '#171a20', companion: '#59616d' },
  { id: 'blue', name: '深海蓝', description: '清透而专注', color: '#2563eb', companion: '#8cb2ff' },
  { id: 'violet', name: '夜紫', description: '柔和微光', color: '#7046c7', companion: '#b9a0ed' },
  { id: 'emerald', name: '翡翠', description: '安静自然', color: '#08785b', companion: '#73c6a6' },
  { id: 'amber', name: '暖金', description: '温润克制', color: '#98620a', companion: '#e3bd70' },
];
function selectTheme(theme: ThemeId) {
  draft.theme = theme;
  emit('theme', theme);
}
// 功能：即时预览明暗模式并保留在当前设置草稿中；时间：2026-11-08；作者：lq。
function selectColorMode(mode: ColorMode) {
  draft.colorMode = mode;
  emit('colorMode', mode);
}
// 功能：根据滚动位置高亮侧栏并滚动到选择项；时间：2026-11-08；作者：lq。
function selectSection(id: string) {
  activeSection.value = id;
  document.getElementById(id)?.scrollIntoView({ behavior: 'smooth', block: 'start' });
}
// 功能：解析 Chrome、Edge 等浏览器导出的 Netscape 书签 HTML 并保留网页标题；时间：2026-11-08；作者：lq。
function parseBookmarkHtml(content: string): { items: BookmarkDraft[]; skipped: number } {
  const document = new DOMParser().parseFromString(content, 'text/html');
  let skipped = 0;
  // 功能：从 Netscape 书签 HTML 的嵌套目录结构还原书签原目录路径；时间：2026-11-08；作者：lq。
  function importFolderPath(anchor: HTMLAnchorElement): string {
    const folders: string[] = [];
    let list = anchor.parentElement?.closest('dl') ?? null;
    while (list) {
      const heading = Array.from(list.parentElement?.children ?? []).find(child => child.tagName.toLowerCase() === 'h3');
      const name = heading?.textContent?.trim();
      if (name && !['bookmarks bar', 'bookmarks menu', 'other bookmarks', '收藏栏', '书签栏', '其他书签'].includes(name.toLocaleLowerCase())) folders.unshift(name);
      list = list.parentElement?.parentElement?.closest('dl') ?? null;
    }
    return folders.join(' / ').slice(0, 512);
  }
  const items = Array.from(document.querySelectorAll<HTMLAnchorElement>('a[href]')).flatMap((anchor) => {
    try {
      const url = new URL(anchor.getAttribute('href') ?? '');
      if (!['http:', 'https:'].includes(url.protocol) || !url.hostname || url.username || url.password) { skipped++; return []; }
      const title = anchor.textContent?.trim() || url.hostname;
      const importFolder = importFolderPath(anchor);
      return [{ url: url.href, title: title.slice(0, 2048), folderId: 'inbox', tags: [], pinned: false, ...(importFolder ? { importFolder } : {}) }];
    } catch { skipped++; return []; }
  });
  return { items, skipped };
}
// 功能：读取用户选定的浏览器书签 HTML 并调用本地批量导入；时间：2026-11-08；作者：lq。
async function importBookmarkFile(event: Event) {
  const input = event.currentTarget as HTMLInputElement;
  const file = input.files?.[0];
  if (!file) return;
  importing.value = true;
  importMessage.value = '';
  try {
    if (file.size > 20 * 1024 * 1024) throw new Error('文件超过 20 MB，请先缩小书签文件后重试。');
    const parsed = parseBookmarkHtml(await file.text());
    const drafts = parsed.items;
    if (!drafts.length) throw new Error('文件中没有找到可导入的 HTTP 或 HTTPS 书签。');
    if (drafts.length > 5000) throw new Error('单次最多导入 5000 条书签，请拆分文件后重试。');
    const result = await api.importBookmarks(drafts);
    const completed = { imported: result.imported, skipped: result.skipped + parsed.skipped, folderMetadataUpdated: result.folderMetadataUpdated };
    importMessage.value = `导入完成：新增 ${completed.imported} 条，跳过 ${completed.skipped} 条${completed.folderMetadataUpdated ? `，补回 ${completed.folderMetadataUpdated} 条原目录线索` : ''}。`;
    emit('imported', completed);
  } catch (error) {
    importMessage.value = `导入失败：${error instanceof Error ? error.message : String(error)}`;
  } finally {
    importing.value = false;
    input.value = '';
  }
}
onMounted(() => {
  const root = document.querySelector('.content-viewport');
  sectionObserver = new IntersectionObserver((entries) => {
    const visible = entries.filter(entry => entry.isIntersecting).sort((a, b) => a.boundingClientRect.top - b.boundingClientRect.top);
    if (visible[0]?.target.id) activeSection.value = visible[0].target.id;
  }, { root, rootMargin: '-12% 0px -65% 0px', threshold: 0 });
  sections.forEach(section => {
    const element = document.getElementById(section.id);
    if (element) sectionObserver?.observe(element);
  });
});
onUnmounted(() => sectionObserver?.disconnect());
</script>

<template>
  <div class="settings-layout">
    <aside class="settings-sidebar"><h1><AppIcon name="settings" />系统设置</h1><nav aria-label="设置分类"><a v-for="section in sections" :key="section.id" :href="`#${section.id}`" :class="{ active: activeSection === section.id }" :aria-current="activeSection === section.id ? 'location' : undefined" @click.prevent="selectSection(section.id)"><AppIcon :name="section.icon" :size="16" />{{ section.title }}</a></nav><p>偏好保存在当前设备，无需登录。</p></aside>
    <div class="settings-content">
      <form @submit.prevent="emit('save', { ...draft })">
        <section id="appearance" class="settings-section"><h2><AppIcon name="palette" />外观与主题 <small>appearance.theme</small></h2><p>选择界面明暗模式和强调色，切换后立即预览。</p><h3 class="appearance-subheading">界面明暗</h3><div class="mode-options" role="radiogroup" aria-label="界面明暗模式"><button v-for="mode in colorModes" :key="mode.id" type="button" role="radio" :aria-checked="draft.colorMode === mode.id" :class="['mode-option', { selected: draft.colorMode === mode.id }]" @click="selectColorMode(mode.id)"><span><strong>{{ mode.name }}</strong><small>{{ mode.description }}</small></span><AppIcon v-if="draft.colorMode === mode.id" name="check" :size="16" /></button></div><h3 class="appearance-subheading theme-heading">强调色</h3><div class="theme-options" role="radiogroup" aria-label="浏览器主题"><button v-for="theme in themes" :key="theme.id" type="button" role="radio" :aria-checked="draft.theme === theme.id" :class="['theme-option', { selected: draft.theme === theme.id }]" @click="selectTheme(theme.id)"><span class="theme-swatch" :style="{ '--swatch': theme.color, '--swatch-companion': theme.companion }"><i></i><i></i><i></i></span><span class="theme-copy"><strong>{{ theme.name }}</strong><small>{{ theme.description }}</small></span><AppIcon v-if="draft.theme === theme.id" name="check" :size="16" /></button></div><small class="muted">外观偏好保存在本机；点击底部“保存设置”后下次启动继续使用。</small></section>
        <section id="startup" class="settings-section"><h2><AppIcon name="home" />启动行为与新标签页 <small>startup.mode</small></h2><p>主页模式与搜索引擎分别配置。</p><label class="radio-option" :class="{ checked: draft.homeMode === 'minimal' }"><input v-model="draft.homeMode" type="radio" value="minimal" name="homepage" /><span><strong>打开极简搜索主页（默认）</strong><small>保留一个搜索框，专注当前浏览。</small></span></label><label class="radio-option" :class="{ checked: draft.homeMode === 'bookmarks' }"><input v-model="draft.homeMode" type="radio" value="bookmarks" name="homepage" /><span><strong>打开收藏书签主页</strong><small>常用置顶、分类与收藏网格。</small></span></label><label class="field-label">启动时打开指定网页（可选）<input v-model="draft.startupUrl" class="text-input" type="url" placeholder="https://example.com" /></label><small class="muted">留空使用内置主页，新标签始终使用上述主页模式。</small><label class="setting-row"><span><strong>在主页显示最近访问</strong><small>默认开启，最多显示 20 项。</small></span><input v-model="draft.showRecent" class="switch" type="checkbox" /></label></section>
        <section id="search" class="settings-section"><h2><AppIcon name="search" />搜索引擎 <small>search.engine</small></h2><label class="setting-row"><span><strong>地址栏默认搜索引擎</strong><small>输入搜索词并回车后使用。</small></span><select v-model="draft.searchEngine" class="text-input"><option v-for="(engine, id) in engines" :key="id" :value="id">{{ engine.name }}</option></select></label></section>
        <div class="settings-save"><button class="button primary" type="submit" :disabled="!desktop">保存设置</button></div>
      </form>
      <section id="import" class="settings-section"><h2><AppIcon name="download" />书签迁移与导入</h2><p>在 Chrome、Edge 或其他浏览器中将书签导出为 HTML 文件，再选择文件导入。会保留原目录层级作为自动分类线索；导入内容进入“待整理”，已存在的网址不会覆盖原收藏。若之前用旧版导入过，可重新选择原文件补回目录线索，网址会自动去重。</p><input ref="importInput" class="visually-hidden" type="file" accept=".html,.htm,text/html" aria-label="选择浏览器书签 HTML 文件" @change="importBookmarkFile" /><div class="inline-actions"><button class="button secondary" :disabled="!desktop || importing" @click="importInput?.click()"><AppIcon name="download" :size="16" />{{ importing ? '正在导入…' : '选择书签 HTML 文件' }}</button></div><p v-if="importMessage" class="muted" role="status" aria-live="polite">{{ importMessage }}</p><small class="muted">单次最多 5000 条书签，文件最大 20 MB。</small></section>
      <section id="default" class="settings-section"><h2><AppIcon name="shield" />默认浏览器</h2><p>系统注册、设置引导和实际状态回读尚未接入。</p><span class="state-badge">尚未验证</span></section>
      <section id="privacy" class="settings-section"><h2><AppIcon name="shield" />隐私与本地数据</h2><label class="setting-row"><span><strong>删除浏览历史</strong><small>不会删除收藏或网站登录数据。</small></span><button class="button destructive" :disabled="!desktop" @click="emit('clearHistory')">删除历史</button></label><p>密码管理、隐私会话和网站数据清理仍待专项实现与验证。</p></section>
      <section id="about" class="settings-section"><h2><AppIcon name="info" />关于轻页 SAB <small>v0.1.0</small></h2><div class="about-brand"><span class="heading-icon">SAB</span><div><strong>轻页 · Small And Beautiful</strong><p>技术骨架 · 作者：非人类程序员 · Apache-2.0</p></div></div><p>基于 v0.3 定稿设计。当前为开发阶段，尚未完成日常浏览与发行验收。</p><!-- 功能：展示公众号二维码缩略图及 QQ 联系方式；时间：2026-11-08；作者：lq。 --><div class="about-contact"><span>QQ：960661824</span><img class="about-qr" :src="officialAccountQr" alt="公众号二维码" title="公众号二维码" /></div></section>
    </div>
  </div>
</template>
