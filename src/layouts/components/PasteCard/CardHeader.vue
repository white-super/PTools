<script setup lang="ts">
import type { ClipboardFormat } from "../../types/settings";

defineProps<{
  format: ClipboardFormat;
  quickKey?: number;
  isDiffSource?: boolean;
  isSelected?: boolean;
}>();
const emit = defineEmits<{ cancelDiff: [] }>();
const FORMAT_LABELS: Record<ClipboardFormat, string> = { text: "文本", image: "图片", file: "文件" };
</script>

<template>
  <div class="card-header" :class="{ 'card-header-selected': isSelected }">
    <span class="format-label">
      <svg viewBox="0 0 16 16" width="14" height="14" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
        <path v-if="format === 'text'" d="M3 4h10M8 4v8M5.5 12h5" />
        <template v-else-if="format === 'image'"><rect x="2" y="2" width="12" height="12" rx="3" /><path d="m3 12 3.5-4 2.5 2 2-2 2 3M5 5h.01" /></template>
        <path v-else d="M9 2H4a1 1 0 0 0-1 1v10a1 1 0 0 0 1 1h8a1 1 0 0 0 1-1V6L9 2Zm0 0v4h4" />
      </svg>
      {{ FORMAT_LABELS[format] }}
    </span>
    <div class="card-indicators" aria-label="卡片功能标签">
      <button
        v-if="isDiffSource" type="button" class="card-indicator diff-indicator"
        title="已标记为左侧对比源，选择另一张卡片按 D 对比；点击取消"
        aria-label="取消 Diff 对比标记" @click.stop="emit('cancelDiff')" @dblclick.stop
      >
        <svg viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" stroke-width="1.7" aria-hidden="true">
          <path d="M3 4h7v16H3zM14 4h7v16h-7zM5 9h3M16 9h3M16 14h3M17.5 12.5v3" />
        </svg>
        <span>Diff</span>
      </button>
      <span v-if="quickKey" class="quick-key">{{ quickKey }}</span>
    </div>
  </div>
</template>

<style scoped>
.card-header {
  position: relative;
  box-sizing: border-box;
  display: flex;
  flex: 0 0 32px;
  height: 32px;
  align-items: center;
  justify-content: space-between;
  margin: -12px -12px 9px;
  padding: 0 9px;
  border-radius: 11px 11px 0 0;
  border-bottom: 1px solid var(--card-header-border-current, var(--panel-card-header-border));
  color: var(--card-header-text-current, var(--panel-card-header-text));
  background: var(--card-header-background-current, var(--panel-card-header-background));
  box-shadow: inset 0 1px 0 var(--panel-card-header-highlight, transparent);
  transition: border-color 180ms ease, color 180ms ease;
  overflow: hidden;
}

.card-header::before {
  position: absolute;
  top: 0;
  bottom: 0;
  left: 0;
  width: 3px;
  border-radius: 11px 0 0 0;
  background: var(--app-primary);
  content: "";
  opacity: 0;
  transform: scaleY(0.35);
  transform-origin: center;
  transition: opacity 180ms ease, transform 180ms ease;
}

.card-header::after {
  position: absolute;
  inset: 0;
  border-radius: inherit;
  background: var(--card-header-overlay-current, transparent);
  box-shadow: inset 0 -2px 0 var(--card-header-accent-current, transparent);
  content: "";
  opacity: var(--card-header-overlay-opacity, 0);
  pointer-events: none;
  transition: opacity 200ms ease;
}

.card-header > * {
  position: relative;
  z-index: 1;
}

.card-header-selected::before {
  opacity: 1;
  transform: scaleY(1);
}
.quick-key {
  display: inline-flex;
  box-sizing: border-box;
  width: 23px;
  height: 22px;
  align-items: center;
  justify-content: center;
  border: 1px solid var(--card-header-key-border, var(--panel-card-header-border));
  border-radius: 6px;
  color: var(--card-header-key-text, var(--panel-card-header-text));
  background: var(--card-header-key-background, var(--app-menu-background));
  box-shadow: 0 1px 1px rgba(15, 23, 42, 0.08);
  font-size: 12px;
  font-weight: 750;
  line-height: 1;
  transition: background-color 180ms ease, border-color 180ms ease, color 180ms ease, transform 200ms ease;
}

.card-indicators {
  display: flex;
  min-width: 0;
  height: 100%;
  align-items: center;
  justify-content: flex-end;
  gap: 6px;
}

.card-indicator {
  display: inline-flex;
  flex: 0 0 auto;
  height: 23px;
  align-items: center;
  justify-content: center;
  gap: 4px;
  border: 0;
  border-radius: 7px;
  padding: 0 7px;
  font: inherit;
  font-size: 11px;
  font-weight: 700;
  cursor: pointer;
  transition: background-color 180ms ease, box-shadow 180ms ease, transform 180ms ease;
}

.diff-indicator {
  color: var(--app-on-primary);
  background: var(--app-primary);
  box-shadow: 0 1px 2px color-mix(in srgb, var(--app-primary) 34%, transparent);
}

.diff-indicator:hover {
  background: color-mix(in srgb, var(--app-primary) 86%, var(--app-heading));
  box-shadow: 0 3px 7px color-mix(in srgb, var(--app-primary) 32%, transparent);
  transform: translateY(-1px);
}

.diff-indicator:focus-visible { outline: 2px solid var(--app-primary); outline-offset: 1px; }

.format-label {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  border: 1px solid color-mix(in srgb, currentColor 14%, transparent);
  border-radius: 7px;
  padding: 4px 8px;
  color: inherit;
  background: color-mix(in srgb, currentColor 8%, transparent);
  font-size: 12px;
  font-weight: 700;
  line-height: 1;
  white-space: nowrap;
}

.card-header-selected .quick-key {
  border-color: var(--card-header-selected-key-border, var(--app-primary));
  color: var(--card-header-selected-key-text, var(--app-primary));
  background: var(--card-header-selected-key-background, var(--app-menu-background));
}

.card-header-selected .diff-indicator {
  box-shadow: 0 2px 5px color-mix(in srgb, var(--app-primary) 38%, transparent);
}

@media (prefers-reduced-motion: reduce) {
  .card-header,
  .card-header::before,
  .card-header::after,
  .quick-key,
  .card-indicator {
    transition: none;
  }
}
</style>
