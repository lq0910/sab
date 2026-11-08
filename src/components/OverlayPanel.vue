<script setup lang="ts">
// 功能：使用原生 dialog 实现焦点约束、Escape 关闭和焦点恢复；时间：2026-11-08；作者：lq。
import { nextTick, ref, watch } from 'vue';
const props = defineProps<{ open: boolean; label: string; compact?: boolean }>();
const emit = defineEmits<{ close: [] }>();
const element = ref<HTMLDialogElement>();
let previous: HTMLElement | null = null;
watch(() => props.open, async (open) => {
  await nextTick();
  if (open && !element.value?.open) {
    previous = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    element.value?.showModal();
  } else if (!open && element.value?.open) {
    element.value.close();
    previous?.focus();
  }
});
</script>

<template>
  <dialog ref="element" :aria-label="label" :class="['overlay-panel', { compact }]" @cancel.prevent="emit('close')" @click="event => { if (event.target === element) emit('close'); }">
    <slot />
  </dialog>
</template>
