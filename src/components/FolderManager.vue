<script setup lang="ts">
// 功能：管理收藏分类的名称、创建和删除，并为删除分类指定书签迁移位置；时间：2026-11-08；作者：lq。
import { onMounted, onUnmounted, reactive, ref, watch } from 'vue';
import type { Folder } from '../domain/models';
import { api } from '../services/desktop';
import AppIcon from './AppIcon.vue';

const props = defineProps<{ folders: Folder[] }>();
const emit = defineEmits<{ close: []; changed: []; error: [message: string] }>();
const names = reactive<Record<string, string>>({});
const newName = ref('');
const deletingId = ref('');
const deleteTargetId = ref('inbox');
const working = ref(false);
const status = ref('');
// 功能：限制分类管理键盘焦点在弹窗内，支持 Escape 关闭并恢复原按钮焦点；时间：2026-11-08；作者：lq。
const dialog = ref<HTMLElement>();
const previousFocus = document.activeElement as HTMLElement | null;
function dialogKey(event: KeyboardEvent) {
  if (event.key === 'Escape' && !working.value) { event.preventDefault(); event.stopPropagation(); emit('close'); }
  if (event.key !== 'Tab') return;
  const elements = Array.from(dialog.value?.querySelectorAll<HTMLElement>('button:not(:disabled), input:not(:disabled), select:not(:disabled)') ?? []);
  const first = elements[0];
  const last = elements[elements.length - 1];
  if (event.shiftKey && document.activeElement === first) { event.preventDefault(); last?.focus(); }
  if (!event.shiftKey && document.activeElement === last) { event.preventDefault(); first?.focus(); }
}
onMounted(() => dialog.value?.querySelector<HTMLInputElement>('input')?.focus());
onUnmounted(() => previousFocus?.focus());
watch(() => props.folders, (folders) => {
  for (const folder of folders) names[folder.id] = folder.name;
}, { immediate: true, deep: true });

// 功能：创建新分类并刷新主页和收藏分类列表；时间：2026-11-08；作者：lq。
async function createFolder() {
  if (working.value || !newName.value.trim()) return;
  working.value = true;
  try {
    await api.createFolder(newName.value);
    newName.value = '';
    status.value = '分类已创建';
    emit('changed');
  } catch (error) { emit('error', String(error)); }
  finally { working.value = false; }
}

// 功能：保存单个分类的新名称且保留原有书签关联；时间：2026-11-08；作者：lq。
async function renameFolder(folder: Folder) {
  const name = names[folder.id]?.trim() ?? '';
  if (working.value || !name || name === folder.name) return;
  working.value = true;
  try {
    await api.renameFolder(folder.id, name);
    status.value = `“${folder.name}”已改名为“${name}”`;
    emit('changed');
  } catch (error) { emit('error', String(error)); }
  finally { working.value = false; }
}

// 功能：确认目标分类后迁移收藏并删除原分类；时间：2026-11-08；作者：lq。
async function deleteFolder(folder: Folder) {
  if (working.value || !deletingId.value || !deleteTargetId.value) return;
  working.value = true;
  try {
    const moved = await api.deleteFolder(folder.id, deleteTargetId.value);
    const target = props.folders.find(item => item.id === deleteTargetId.value)?.name ?? '目标分类';
    status.value = `分类已删除，${moved} 条收藏已移入“${target}”`;
    deletingId.value = '';
    emit('changed');
  } catch (error) { emit('error', String(error)); }
  finally { working.value = false; }
}
</script>

<template>
  <div class="folder-manager-backdrop" @click.self="emit('close')">
    <section ref="dialog" class="folder-manager" role="dialog" aria-modal="true" aria-labelledby="folder-manager-title" @keydown="dialogKey">
      <header class="folder-manager-header"><div><h2 id="folder-manager-title">管理收藏分类</h2><p>删除分类前，先选择把其中的收藏移到哪里。</p></div><button class="icon-button" aria-label="关闭分类管理" @click="emit('close')"><AppIcon name="close" /></button></header>
      <form class="folder-create" @submit.prevent="createFolder"><input v-model="newName" class="text-input" maxlength="32" placeholder="新分类名称" aria-label="新分类名称" :disabled="working" /><button class="button primary" type="submit" :disabled="working || !newName.trim()"><AppIcon name="plus" :size="15" />添加分类</button></form>
      <div class="folder-manager-list">
        <article v-for="folder in folders" :key="folder.id" class="folder-manager-row">
          <div class="folder-manager-edit"><AppIcon name="folder" :size="17" /><input v-model="names[folder.id]" class="text-input" :aria-label="`分类名称：${folder.name}`" maxlength="32" :disabled="working || folder.id === 'inbox'" @keyup.enter="renameFolder(folder)" /><span class="folder-manager-count">{{ folder.count }}</span><button class="button secondary" :disabled="working || folder.id === 'inbox' || names[folder.id]?.trim() === folder.name" @click="renameFolder(folder)">保存</button><button v-if="folder.id !== 'inbox'" class="icon-button folder-delete" :disabled="working" :aria-label="`删除分类 ${folder.name}`" @click="deletingId = folder.id; deleteTargetId = 'inbox'"><AppIcon name="trash" :size="16" /></button><span v-else class="folder-system-label">系统分类</span></div>
          <div v-if="deletingId === folder.id" class="folder-delete-confirm"><label>迁移到<select v-model="deleteTargetId" class="text-input"><option v-for="target in folders.filter(item => item.id !== folder.id)" :key="target.id" :value="target.id">{{ target.name }}</option></select></label><button class="button secondary" :disabled="working" @click="deletingId = ''">取消</button><button class="button destructive" :disabled="working || !deleteTargetId" @click="deleteFolder(folder)">确认删除</button></div>
        </article>
      </div>
      <footer><span v-if="status" role="status" aria-live="polite">{{ status }}</span><span v-else>“待整理”是系统分类，不能删除或改名。</span><button class="button secondary" @click="emit('close')">完成</button></footer>
    </section>
  </div>
</template>
