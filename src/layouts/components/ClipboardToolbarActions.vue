<script setup lang="ts">
import ClipboardMoreMenu from "./ClipboardMoreMenu.vue";

const moreMenuOpen = defineModel<boolean>({ required: true });
const emit = defineEmits<{ settings: []; textDiff: [] }>();
</script>

<template>
  <div class="toolbar-actions">
    <button
      type="button"
      class="text-diff-trigger"
      aria-label="打开空白文本对比窗口"
      title="文本对比（打开后默认置顶）"
      @click="emit('textDiff')"
    >
      <svg viewBox="0 0 20 20" aria-hidden="true">
        <rect x="2.5" y="3" width="5.5" height="14" rx="1.5" />
        <rect x="12" y="3" width="5.5" height="14" rx="1.5" />
        <path d="M9.5 7.25h1M9.5 12.75h1" />
      </svg>
      <span>文本对比</span>
    </button>
    <ClipboardMoreMenu v-model="moreMenuOpen" @settings="emit('settings')" />
  </div>
</template>

<style scoped>
.toolbar-actions {
  display: flex;
  min-width: 0;
  grid-column: 3;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
}

.text-diff-trigger {
  display: inline-flex;
  height: 28px;
  flex: 0 0 auto;
  align-items: center;
  gap: 6px;
  padding: 0 10px;
  border: 1px solid var(--panel-search-border);
  border-radius: 7px;
  color: var(--panel-filter-hover-text);
  background: var(--panel-control);
  box-shadow: var(--panel-search-shadow);
  backdrop-filter: var(--panel-control-backdrop);
  font: inherit;
  font-size: 12px;
  font-weight: 550;
  cursor: pointer;
  transition: border-color 150ms ease, color 150ms ease, background-color 150ms ease;
  -webkit-backdrop-filter: var(--panel-control-backdrop);
}

.text-diff-trigger svg {
  width: 15px;
  height: 15px;
  fill: none;
  stroke: currentColor;
  stroke-linecap: round;
  stroke-linejoin: round;
  stroke-width: 1.35;
}

.text-diff-trigger:hover {
  border-color: var(--panel-search-focus-border);
  color: var(--panel-filter-active-text);
  background: var(--panel-control-active);
}

.text-diff-trigger:focus-visible {
  outline: 2px solid var(--panel-search-focus-border);
  outline-offset: 2px;
}
</style>
