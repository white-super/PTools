<script setup lang="ts">
import { useTemplateRef } from "vue";
defineProps<{ side: string; name: string; loading: boolean; canCopy: boolean }>();
const emit = defineEmits<{ import: [file: File]; copy: []; restore: [] }>();
const picker = useTemplateRef<HTMLInputElement>("picker");
function selectFile(event: Event) {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];
  input.value = "";
  if (file) emit("import", file);
}
</script>

<template>
  <div class="source-header">
    <strong>{{ side }}</strong><span class="source-name" :title="name">{{ loading ? '正在读取文件…' : name }}</span>
    <input ref="picker" type="file" hidden @change="selectFile" />
    <button type="button" :title="`${side}导入 UTF-8 文本文件`" :aria-label="`${side}导入文件`" :disabled="loading" @click="picker?.click()">导入</button>
    <button type="button" :title="`${side}复制当前显示内容`" :aria-label="`${side}复制内容`" :disabled="!canCopy" @click="emit('copy')">复制</button>
    <button type="button" :title="`${side}恢复导入时的原文`" :aria-label="`${side}恢复原文`" :disabled="loading" @click="emit('restore')">恢复</button>
  </div>
</template>

<style scoped>
.source-header { display: flex; min-width: 0; align-items: center; gap: 8px; padding: 6px 10px; font-size: 12px; background: var(--app-surface); border-bottom: 1px solid var(--app-border); }
.source-name { min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; flex: 1; color: var(--app-muted); }
strong, button { flex: 0 0 auto; }
button { background: transparent; color: var(--app-secondary); font: inherit; padding: 3px; border: 0; border-radius: 4px; cursor: pointer; }
button:hover { background: var(--app-active); }
button:disabled { opacity: .45; cursor: default; }
</style>
