<script setup lang="ts">
import { computed } from "vue";
import type { PasteFlowShortcutSettings } from "../types/settings";

interface Props {
  readonly shortcutSettings?: PasteFlowShortcutSettings;
}

const DEFAULT_SHORTCUTS: PasteFlowShortcutSettings = {
  mainShortcut: "Ctrl+V",
  previousFilterShortcut: "Ctrl+Q",
  nextFilterShortcut: "Ctrl+E",
  previousCardShortcut: "Ctrl+A",
  nextCardShortcut: "Ctrl+D",
};

const props = defineProps<Props>();
const emit = defineEmits<{ close: [] }>();
const shortcutSettings = computed(() => props.shortcutSettings ?? DEFAULT_SHORTCUTS);
const helpSections = computed(() => [
  {
    title: "面板操作",
    items: [
      { label: "唤出粘贴面板", shortcut: shortcutSettings.value.mainShortcut },
      { label: "定位搜索框", shortcut: "Command+F" },
      { label: "粘贴选中内容", shortcut: "Enter" },
      { label: "快速粘贴前五项", shortcut: "1–5" },
    ],
  },
  {
    title: "标签切换",
    items: [
      { label: "上一个标签", shortcut: shortcutSettings.value.previousFilterShortcut },
      { label: "下一个标签", shortcut: shortcutSettings.value.nextFilterShortcut },
    ],
  },
  {
    title: "卡片切换",
    items: [
      { label: "上一张卡片", shortcut: `${shortcutSettings.value.previousCardShortcut} / ←` },
      { label: "下一张卡片", shortcut: `${shortcutSettings.value.nextCardShortcut} / →` },
    ],
  },
]);
</script>

<template>
  <article class="help-card" aria-label="快捷键帮助文档">
    <header class="help-header">
      <strong>快捷键帮助</strong>
      <button type="button" class="help-close" aria-label="关闭帮助" @click="emit('close')">×</button>
    </header>
    <div v-for="section in helpSections" :key="section.title" class="help-section">
      <h2 class="help-section-title">{{ section.title }}</h2>
      <div v-for="item in section.items" :key="item.label" class="help-row">
        <span>{{ item.label }}</span>
        <kbd>{{ item.shortcut }}</kbd>
      </div>
    </div>
    <p class="help-note">组合键可以在设置界面中修改。</p>
  </article>
</template>

<style scoped>
.help-card {
  box-sizing: border-box;
  width: 100%;
  padding: 12px 14px;
  border: 1px solid var(--app-border);
  border-radius: 10px;
  color: var(--app-text);
  background: var(--shortcut-help-background);
  box-shadow: var(--app-shadow);
}

.help-header,
.help-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}

.help-header {
  color: var(--app-heading);
  font-size: 14px;
}

.help-close {
  width: 22px;
  height: 22px;
  padding: 0;
  border: 0;
  color: var(--app-muted);
  background: transparent;
  font: inherit;
  font-size: 18px;
  line-height: 1;
  cursor: default;
}

.help-section {
  margin-top: 12px;
}

.help-section-title {
  margin: 0 0 5px;
  color: var(--app-muted);
  font-size: 11px;
  font-weight: 600;
}

.help-row {
  min-height: 25px;
  font-size: 12px;
}

.help-row kbd {
  flex: 0 0 auto;
  padding: 2px 6px;
  border: 1px solid var(--app-border);
  border-radius: 5px;
  color: var(--app-secondary);
  background: var(--app-surface);
  font: inherit;
  font-size: 11px;
}

.help-note {
  margin: 10px 0 0;
  color: var(--app-muted);
  font-size: 11px;
}
</style>
