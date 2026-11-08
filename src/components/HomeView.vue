<script setup lang="ts">
// 功能：按 _4 页面稿实现搜索主页；最近记录默认由设置关闭；时间：2026-11-08；作者：lq。
import { ref } from 'vue';
import appLogo from '../../src-tauri/icons/source.svg';
import type { RecentVisit, SearchEngine, Settings } from '../domain/models';
import { displayHost, engines } from '../domain/navigation';
import AppIcon from './AppIcon.vue';
const props = defineProps<{ settings: Settings; recent: RecentVisit[] }>();
const emit = defineEmits<{ navigate: [value: string]; engine: [value: SearchEngine]; hideRecent: [url: string]; clearHistory: [] }>();
const query = ref('');
const expanded = ref(false);
// 功能：最近访问为空或被关闭时隐藏整块区域，并切换主页搜索布局；时间：2026-11-08；作者：lq。
const showRecentSection = () => props.settings.showRecent && props.recent.length > 0;
</script>

<template>
  <div class="home-view" :class="{ 'home-view--no-recent': !showRecentSection() }">
    <section class="home-search" aria-label="搜索主页">
      <!-- 功能：在无最近记录的主页搜索框上方展示 SAB 品牌标志；时间：2026-11-08；作者：lq。 -->
      <img v-if="!showRecentSection()" class="home-brand-logo" :src="appLogo" alt="轻页 SAB" />
      <form class="search-capsule" @submit.prevent="emit('navigate', query)">
        <div class="engine-control">
          <AppIcon name="search" />
          <select aria-label="搜索引擎" :value="props.settings.searchEngine" @change="emit('engine', ($event.target as HTMLSelectElement).value as SearchEngine)">
            <option v-for="(engine, id) in engines" :key="id" :value="id">{{ engine.name }}</option>
          </select>
        </div>
        <input v-model="query" aria-label="搜索或输入网址" placeholder="键入搜索词，或输入网址直达" autocomplete="off" autofocus />
        <button class="search-submit" type="submit" aria-label="开始搜索"><AppIcon name="forward" :size="20" /></button>
      </form>
      <div class="search-hints"><span><kbd>Enter</kbd> 直接检索</span><span><kbd>⌘ / Ctrl L</kbd> 聚焦地址栏</span></div>
    </section>
    <section v-if="showRecentSection()" class="recent-section" aria-label="最近访问">
      <div class="section-heading">
        <h2><AppIcon name="clock" />最近访问 <span class="count-badge">{{ recent.length }} 项</span></h2>
        <div class="inline-actions">
          <button v-if="recent.length > 6" class="text-button" @click="expanded = !expanded">{{ expanded ? '收起' : '展开更多' }}<AppIcon name="chevron" :size="14" /></button>
          <button class="text-button" :disabled="!recent.length" @click="emit('clearHistory')"><AppIcon name="trash" :size="14" />删除浏览历史</button>
        </div>
      </div>
      <div class="recent-grid">
        <article v-for="visit in recent.slice(0, expanded ? 20 : 6)" :key="visit.url" class="recent-card">
          <button class="site-link" @click="emit('navigate', visit.url)"><span class="site-icon"><AppIcon name="globe" /></span><span><strong>{{ visit.title }}</strong><small>{{ displayHost(visit.url) }}</small></span></button>
          <button class="icon-button recent-hide" aria-label="在主页隐藏此项" @click="emit('hideRecent', visit.url)"><AppIcon name="close" :size="14" /></button>
        </article>
      </div>
    </section>
    <footer class="home-footer"><span class="status-dot"></span>轻页 SAB · Small And Beautiful</footer>
  </div>
</template>
