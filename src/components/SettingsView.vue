<script setup lang="ts">
// 功能：按 chrome 设置页稿实现启动、搜索和本地偏好，未实现能力明确标记；时间：2026-11-08；作者：lq。
import { computed, onMounted, onUnmounted, reactive, ref, watch } from 'vue';
import type { BookmarkDraft, BookmarkImportResult, ColorMode, Settings, ThemeId } from '../domain/models';
import { engines } from '../domain/navigation';
import { api, desktop, type CredentialAccount, type DefaultBrowserStatus } from '../services/desktop';
import AppIcon from './AppIcon.vue';
import officialAccountQr from '../../docs/qrcode_for_gh_554a10e3d22b_258.jpg';
// 功能：关于页显示与构建清单一致的版本、构建号及已交付范围；时间：2026-11-14；作者：lq。
import { release } from '../domain/release';
// 功能：校验 Chrome 导出的 CSV 后分批送入本地系统密码库；时间：2026-11-16；作者：lq。
import { parseChromePasswordsCsv, type CredentialDraft } from '../domain/credentialCsv';
const props = defineProps<{ settings: Settings }>();
const emit = defineEmits<{ save: [value: Settings]; clearHistory: []; theme: [value: ThemeId]; colorMode: [value: ColorMode]; imported: [value: BookmarkImportResult] }>();
const draft = reactive({ ...props.settings });
const activeSection = ref('appearance');
// 功能：跟踪 HTML 书签文件选择、导入进度和结果提示；时间：2026-11-08；作者：lq。
const importInput = ref<HTMLInputElement>();
const importing = ref(false);
const importMessage = ref('');
const passwordInput = ref<HTMLInputElement>();
const importingPasswords = ref(false);
const passwordImportMessage = ref('');
// 功能：选择文件后先预览数量并等待明确导入操作，账号列表只持有公开索引；时间：2026-11-16；作者：lq。
const pendingPasswords = ref<CredentialDraft[]>([]);
const passwordSkipped = ref(0);
const savedAccounts = ref<CredentialAccount[]>([]);
const accountQuery = ref('');
const accountLimit = ref(50);
const deletingAccount = ref('');
const confirmDeleteAccount = ref('');
const filteredAccounts = computed(() => savedAccounts.value.filter(account => `${account.origin} ${account.username}`.toLocaleLowerCase().includes(accountQuery.value.trim().toLocaleLowerCase())));
watch(accountQuery, () => { accountLimit.value = 50; });
let sectionObserver: IntersectionObserver | undefined;
// 功能：默认浏览器状态读取、请求反馈及返回应用后的刷新；时间：2026-11-14；作者：lq。
const browserStatus = ref<DefaultBrowserStatus>();
const browserBusy = ref(false);
const browserMessage = ref('');
async function refreshDefaultBrowser() {
  if (!desktop || browserBusy.value) return;
  browserBusy.value = true;
  try { browserStatus.value = await api.defaultBrowserStatus(); browserMessage.value = browserStatus.value.message; }
  catch (error) { browserMessage.value = String(error); }
  finally { browserBusy.value = false; }
}
async function setDefaultBrowser() {
  if (!desktop || browserBusy.value) return;
  browserBusy.value = true;
  try { browserStatus.value = await api.requestDefaultBrowser(); browserMessage.value = browserStatus.value.message; }
  catch (error) { browserMessage.value = String(error); }
  finally { browserBusy.value = false; }
}
// 功能：只在设置整体重载后刷新草稿，主题预览不覆盖其他未保存选项；时间：2026-11-14；作者：lq。
watch(() => props.settings, value => Object.assign(draft, value));
const sections = [
  { id: 'appearance', title: '外观与主题', icon: 'palette' },
  { id: 'startup', title: '启动行为与主页', icon: 'home' },
  { id: 'search', title: '搜索引擎', icon: 'search' },
  { id: 'import', title: '书签迁移与导入', icon: 'download' },
  // 功能：为 Chrome 登录密码迁移提供独立设置入口；时间：2026-11-16；作者：lq。
  { id: 'passwords', title: '密码管理', icon: 'key' },
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
// 功能：读取 Chrome CSV 仅预览数量，在本地界面明确确认后才调用密码库；时间：2026-11-16；作者：lq。
async function importPasswordFile(event: Event) {
  const input = event.currentTarget as HTMLInputElement;
  const file = input.files?.[0];
  if (!file) return;
  clearPendingPasswords();
  passwordImportMessage.value = '';
  try {
    if (file.size > 20 * 1024 * 1024) throw new Error('CSV 文件超过 20 MB。');
    const parsed = parseChromePasswordsCsv(await file.text());
    if (!parsed.credentials.length) throw new Error('CSV 中没有找到可导入的账号密码。');
    if (parsed.credentials.length > 5000) throw new Error('单次最多导入 5000 个账号。');
    pendingPasswords.value = parsed.credentials;
    passwordSkipped.value = parsed.skipped;
  } catch (error) {
    passwordImportMessage.value = `导入失败：${error instanceof Error ? error.message : String(error)}`;
  } finally {
    input.value = '';
  }
}
// 功能：取消或完成后尽早解除 JavaScript 对密码字段的引用；时间：2026-11-16；作者：lq。
function clearPendingPasswords() {
  pendingPasswords.value.forEach(account => { account.password = ''; });
  pendingPasswords.value = [];
}
// 功能：分批导入避免一个大批次长期占用数据库，失败数和已完成数独立显示；时间：2026-11-16；作者：lq。
async function confirmPasswordImport() {
  if (importingPasswords.value || !pendingPasswords.value.length) return;
  importingPasswords.value = true;
  let imported = 0, skipped = passwordSkipped.value, failed = 0;
  try {
    for (let offset = 0; offset < pendingPasswords.value.length; offset += 25) {
      const batch = pendingPasswords.value.slice(offset, offset + 25);
      const result = await api.importCredentials(batch);
      imported += result.imported; skipped += result.skipped; failed += result.failed;
      batch.forEach(account => { account.password = ''; });
      passwordImportMessage.value = `正在导入 ${Math.min(offset + 25, pendingPasswords.value.length)} / ${pendingPasswords.value.length}…`;
    }
    passwordImportMessage.value = `导入完成：新增 ${imported} 个账号，跳过 ${skipped} 个项目${failed ? `，${failed} 个失败；请解锁系统密码库并允许访问后，重新导入原 CSV` : ''}。`;
  } catch (error) {
    passwordImportMessage.value = `已新增 ${imported} 个账号。导入中断：${String(error)}。重新选择原 CSV 可补入剩余项目。`;
  } finally {
    clearPendingPasswords(); importingPasswords.value = false;
    await refreshSavedAccounts();
  }
}
// 功能：设置页按需检索不含密码的账号列表；时间：2026-11-16；作者：lq。
async function refreshSavedAccounts() {
  if (!desktop) return;
  try { savedAccounts.value = await api.listCredentials(''); }
  catch (error) { passwordImportMessage.value = String(error); }
}
// 功能：单条确认删除后清理系统密码与本地账号索引；时间：2026-11-16；作者：lq。
async function removeSavedAccount(id: string) {
  if (deletingAccount.value) return;
  deletingAccount.value = id;
  try { await api.deleteCredential(id); confirmDeleteAccount.value = ''; await refreshSavedAccounts(); }
  catch (error) { passwordImportMessage.value = String(error); }
  finally { deletingAccount.value = ''; }
}
onMounted(() => {
  void refreshDefaultBrowser();
  void refreshSavedAccounts();
  window.addEventListener('focus', refreshDefaultBrowser);
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
onUnmounted(() => { if (!importingPasswords.value) clearPendingPasswords(); sectionObserver?.disconnect(); window.removeEventListener('focus', refreshDefaultBrowser); });
</script>

<template>
  <div class="settings-layout">
    <aside class="settings-sidebar"><h1><AppIcon name="settings" />系统设置</h1><nav aria-label="设置分类"><a v-for="section in sections" :key="section.id" :href="`#${section.id}`" :class="{ active: activeSection === section.id }" :aria-current="activeSection === section.id ? 'location' : undefined" @click.prevent="selectSection(section.id)"><AppIcon :name="section.icon" :size="16" />{{ section.title }}</a></nav><p>偏好保存在当前设备，无需登录。</p></aside>
    <div class="settings-content">
      <form @submit.prevent="emit('save', { ...draft })">
        <section id="appearance" class="settings-section"><h2><AppIcon name="palette" />外观与主题 <small>appearance.theme</small></h2><p>选择界面明暗模式和强调色，切换后立即预览。</p><h3 class="appearance-subheading">界面明暗</h3><div class="mode-options" role="radiogroup" aria-label="界面明暗模式"><button v-for="mode in colorModes" :key="mode.id" type="button" role="radio" :aria-checked="draft.colorMode === mode.id" :class="['mode-option', { selected: draft.colorMode === mode.id }]" @click="selectColorMode(mode.id)"><span><strong>{{ mode.name }}</strong><small>{{ mode.description }}</small></span><AppIcon v-if="draft.colorMode === mode.id" name="check" :size="16" /></button></div><h3 class="appearance-subheading theme-heading">强调色</h3><div class="theme-options" role="radiogroup" aria-label="浏览器主题"><button v-for="theme in themes" :key="theme.id" type="button" role="radio" :aria-checked="draft.theme === theme.id" :class="['theme-option', { selected: draft.theme === theme.id }]" @click="selectTheme(theme.id)"><span class="theme-swatch" :style="{ '--swatch': theme.color, '--swatch-companion': theme.companion }"><i></i><i></i><i></i></span><span class="theme-copy"><strong>{{ theme.name }}</strong><small>{{ theme.description }}</small></span><AppIcon v-if="draft.theme === theme.id" name="check" :size="16" /></button></div><small class="muted">外观偏好保存在本机；点击底部“保存设置”后下次启动继续使用。</small></section>
        <section id="startup" class="settings-section"><h2><AppIcon name="home" />启动行为与新标签页 <small>startup.mode</small></h2><p>主页模式与搜索引擎分别配置。</p><label class="radio-option" :class="{ checked: draft.homeMode === 'minimal' }"><input v-model="draft.homeMode" type="radio" value="minimal" name="homepage" /><span><strong>打开极简搜索主页（默认）</strong><small>保留一个搜索框，专注当前浏览。</small></span></label><label class="radio-option" :class="{ checked: draft.homeMode === 'bookmarks' }"><input v-model="draft.homeMode" type="radio" value="bookmarks" name="homepage" /><span><strong>打开收藏书签主页</strong><small>常用置顶、分类与收藏网格。</small></span></label><label class="field-label">启动时打开指定网页（可选）<input v-model="draft.startupUrl" class="text-input" type="url" placeholder="https://example.com" /></label><small class="muted">指定网页优先于会话恢复；留空时按下方恢复设置启动。新标签使用上述主页模式。</small><!-- 功能：允许用户关闭会话保存，并说明恢复范围；时间：2026-11-14；作者：lq。 --><label class="setting-row"><span><strong>启动时恢复上次标签</strong><small>默认开启。恢复网址、标签和缩放；后台网页选择后才加载。关闭并保存后会清除会话记录。</small></span><input v-model="draft.restoreSession" class="switch" type="checkbox" /></label><label class="setting-row"><span><strong>在主页显示最近访问</strong><small>默认开启，最多显示 20 项。</small></span><input v-model="draft.showRecent" class="switch" type="checkbox" /></label></section>
        <section id="search" class="settings-section"><h2><AppIcon name="search" />搜索引擎 <small>search.engine</small></h2><label class="setting-row"><span><strong>地址栏默认搜索引擎</strong><small>输入搜索词并回车后使用。</small></span><select v-model="draft.searchEngine" class="text-input"><option v-for="(engine, id) in engines" :key="id" :value="id">{{ engine.name }}</option></select></label></section>
        <div class="settings-save"><button class="button primary" type="submit" :disabled="!desktop">保存设置</button></div>
      </form>
      <section id="import" class="settings-section"><h2><AppIcon name="download" />书签迁移与导入</h2><p>在 Chrome、Edge 或其他浏览器中将书签导出为 HTML 文件，再选择文件导入。会保留原目录层级作为自动分类线索；导入内容进入“待整理”，已存在的网址不会覆盖原收藏。若之前用旧版导入过，可重新选择原文件补回目录线索，网址会自动去重。</p><input ref="importInput" class="visually-hidden" type="file" accept=".html,.htm,text/html" aria-label="选择浏览器书签 HTML 文件" @change="importBookmarkFile" /><div class="inline-actions"><button class="button secondary" :disabled="!desktop || importing" @click="importInput?.click()"><AppIcon name="download" :size="16" />{{ importing ? '正在导入…' : '选择书签 HTML 文件' }}</button></div><p v-if="importMessage" class="muted" role="status" aria-live="polite">{{ importMessage }}</p><small class="muted">单次最多 5000 条书签，文件最大 20 MB。</small></section>
      <!-- 功能：提供 Chrome 登录密码 CSV 的本机安全迁移入口；时间：2026-11-16；作者：lq。 -->
      <section id="passwords" class="settings-section">
        <h2><AppIcon name="key" />密码管理</h2>
        <p>在 Chrome「密码管理工具 → 设置 → 导出密码」中保存 CSV，再选择文件导入。密码保存在 macOS 钥匙串或 Windows 凭据管理器。导出文件是明文，请导入后删除。</p>
        <input ref="passwordInput" class="visually-hidden" type="file" accept=".csv,text/csv" aria-label="选择 Chrome 密码 CSV 文件" @change="importPasswordFile" />
        <div class="inline-actions"><button class="button secondary" :disabled="!desktop || importingPasswords" @click="passwordInput?.click()"><AppIcon name="download" :size="16" />{{ importingPasswords ? '正在导入…' : '选择 Chrome 密码 CSV' }}</button></div>
        <div v-if="pendingPasswords.length && !importingPasswords" class="password-import-preview"><p>准备导入 {{ pendingPasswords.length }} 个账号，跳过 {{ passwordSkipped }} 个无效或重复项目。已有同网站同账号的密码会保留。</p><div class="inline-actions"><button class="button primary" @click="confirmPasswordImport">确认导入 {{ pendingPasswords.length }} 个账号</button><button class="button secondary" @click="clearPendingPasswords">取消</button></div></div>
        <p v-if="passwordImportMessage" class="muted" role="status" aria-live="polite">{{ passwordImportMessage }}</p>
        <small class="muted">单次最多 5000 个账号、20 MB。普通 HTTPS 登录页可通过地址栏右侧钥匙按钮选择账号填写；不自动提交，也不自动保存新密码。Passkey 不在 CSV 迁移范围内。</small>
        <div class="saved-accounts-heading"><h3>已保存账号 <span>{{ savedAccounts.length }}</span></h3><button class="button secondary" :disabled="!desktop || importingPasswords" @click="refreshSavedAccounts">刷新</button></div>
        <input v-if="savedAccounts.length" v-model="accountQuery" class="text-input" type="search" aria-label="搜索已保存账号" placeholder="搜索网站或账号" />
        <p v-if="!filteredAccounts.length" class="muted">{{ accountQuery ? '没有匹配的账号' : '还没有已保存账号' }}</p>
        <div class="saved-accounts"><div v-for="account in filteredAccounts.slice(0, accountLimit)" :key="account.id" class="saved-account"><AppIcon name="key" :size="16" /><span><strong>{{ account.username || '未保存用户名' }}</strong><small>{{ account.origin }}</small></span><div v-if="confirmDeleteAccount === account.id" class="inline-actions"><button class="button destructive" :disabled="!!deletingAccount" @click="removeSavedAccount(account.id)">确认删除</button><button class="button secondary" :disabled="!!deletingAccount" @click="confirmDeleteAccount = ''">取消</button></div><button v-else class="icon-button" :aria-label="`删除 ${account.origin} 的 ${account.username || '未命名账号'}`" :disabled="importingPasswords" @click="confirmDeleteAccount = account.id"><AppIcon name="trash" :size="16" /></button></div></div>
        <button v-if="filteredAccounts.length > accountLimit" class="button secondary" @click="accountLimit += 50">显示更多账号</button>
      </section>
      <!-- 功能：设置默认浏览器展示真实HTTP/HTTPS状态，点击才请求系统更改；时间：2026-11-14；作者：lq。 -->
      <section id="default" class="settings-section"><h2><AppIcon name="shield" />默认浏览器</h2><p>{{ browserStatus?.http && browserStatus?.https ? '轻页 SAB 已是默认浏览器' : '让系统中的网页链接使用轻页 SAB 打开。' }}</p><p v-if="browserStatus" class="muted">HTTP：{{ browserStatus.http ? '已默认' : '未默认' }} · HTTPS：{{ browserStatus.https ? '已默认' : '未默认' }}</p><div class="inline-actions"><button class="button primary" :disabled="!desktop || browserBusy || browserStatus?.supported === false || (browserStatus?.http && browserStatus?.https)" @click="setDefaultBrowser">{{ browserBusy ? '正在读取系统状态…' : '设为默认浏览器' }}</button><button class="button secondary" :disabled="!desktop || browserBusy" @click="refreshDefaultBrowser">刷新状态</button></div><p class="muted" role="status" aria-live="polite">{{ browserMessage }}</p></section>
      <section id="privacy" class="settings-section"><h2><AppIcon name="shield" />隐私与本地数据</h2><label class="setting-row"><span><strong>删除浏览历史</strong><small>不会删除收藏或网站登录数据。</small></span><button class="button destructive" :disabled="!desktop" @click="emit('clearHistory')">删除历史</button></label><p>密码库使用操作系统安全存储，不参与浏览历史清理。</p></section>
      <!-- 功能：首版清楚标注预览渠道、版本和本地优先的数据边界；时间：2026-11-14；作者：lq。 -->
      <section id="about" class="settings-section"><h2><AppIcon name="info" />关于轻页 SAB <small>v{{ release.version }} · {{ release.channel }}</small></h2><div class="about-brand"><span class="heading-icon">SAB</span><div><strong>轻页 · Small And Beautiful</strong><p>作者：lq · Apache-2.0 · 构建 {{ release.build }}</p></div></div><p>轻量日常浏览、收藏整理与文件下载。使用系统网页引擎；收藏、历史、下载记录和标签会话保存在当前设备，不提供账号同步。</p><p>预览版支持 Chrome 密码 CSV 导入和手动填表；暂不支持自动更新和隐私窗口。默认浏览器设置由系统确认。下载支持保存位置、记录、重试与文件定位；暂停、单任务取消及断点续传尚未提供。</p><!-- 功能：展示公众号二维码缩略图及 QQ 联系方式；时间：2026-11-08；作者：lq。 --><div class="about-contact"><span>QQ：960661824</span><img class="about-qr" :src="officialAccountQr" alt="公众号二维码" title="公众号二维码" /></div></section>
    </div>
  </div>
</template>
