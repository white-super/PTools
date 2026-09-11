<script setup lang="ts">
import { computed } from "vue";
import type { DiffSide } from "../../features/text-diff/types";
import type { ClipboardFormat } from "../../types/settings";

const props = defineProps<{
  format: ClipboardFormat;
  quickKey?: number;
  diffSide?: DiffSide;
  isSelected?: boolean;
}>();
const emit = defineEmits<{ cancelDiff: [] }>();
const FORMAT_LABELS: Record<ClipboardFormat, string> = { text: "文本", image: "图片", file: "文件" };
const DIFF_MARKERS = {
  left: {
    label: "Diff-L",
    title: "已标记为左侧对比源；选择另一张卡片按 D 对比，点击清除标记",
    ariaLabel: "清除左侧文本对比标记",
  },
  right: {
    label: "DIFF-R",
    title: "当前右侧对比目标；按 D 开始对比，点击清除标记",
    ariaLabel: "清除右侧文本对比标记",
  },
} as const;
const diffMarker = computed(() => props.diffSide ? DIFF_MARKERS[props.diffSide] : undefined);
</script>

<template>
  <div class="card-header" :class="{ 'card-header-selected': isSelected }">
    <div class="card-header-leading">
      <span class="format-label">
        <svg viewBox="0 0 16 16" width="14" height="14" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <path v-if="format === 'text'" d="M3 4h10M8 4v8M5.5 12h5" />
          <template v-else-if="format === 'image'"><rect x="2" y="2" width="12" height="12" rx="3" /><path d="m3 12 3.5-4 2.5 2 2-2 2 3M5 5h.01" /></template>
          <path v-else d="M9 2H4a1 1 0 0 0-1 1v10a1 1 0 0 0 1 1h8a1 1 0 0 0 1-1V6L9 2Zm0 0v4h4" />
        </svg>
        {{ FORMAT_LABELS[format] }}
      </span>
      <div v-if="diffMarker" class="card-indicators" aria-label="卡片功能标签">
        <button
          type="button" class="card-indicator diff-indicator"
          :class="`diff-indicator-${props.diffSide}`"
          :title="diffMarker.title"
          :aria-label="diffMarker.ariaLabel" @click.stop="emit('cancelDiff')" @dblclick.stop
        >
          <span>{{ diffMarker.label }}</span>
        </button>
      </div>
    </div>
    <span v-if="quickKey" class="quick-key">{{ quickKey }}</span>
  </div>
</template>

<style scoped>
.card-header {
  position: relative;
  box-sizing: border-box;
  display: flex;
  flex: 0 0 34px;
  height: 34px;
  align-items: center;
  justify-content: space-between;
  margin: -12px -12px 9px;
  padding: 0 10px;
  border-radius: 13px 13px 0 0;
  border-bottom: 1px solid var(--card-header-border-current, var(--panel-card-header-border));
  color: var(--card-header-text-current, var(--panel-card-header-text));
  background: var(--card-header-background-current, var(--panel-card-header-background));
  transition: background-color 150ms ease, border-color 150ms ease, color 150ms ease;
  overflow: hidden;
}

.card-header-leading {
  display: flex;
  min-width: 0;
  flex: 1;
  align-items: center;
  gap: 8px;
  overflow: hidden;
}

.quick-key {
  display: inline-flex;
  box-sizing: border-box;
  width: 22px;
  height: 22px;
  flex: 0 0 22px;
  align-items: center;
  justify-content: center;
  border: 1px solid var(--card-header-key-border, var(--panel-card-header-border));
  border-radius: 7px;
  color: var(--card-header-key-text, var(--panel-card-header-text));
  background: var(--card-header-key-background, var(--app-menu-background));
  margin-left: 8px;
  font-size: 12px;
  font-variant-numeric: tabular-nums;
  font-weight: 600;
  line-height: 1;
  transition: background-color 150ms ease, border-color 150ms ease, color 150ms ease;
}

.card-indicators {
  display: flex;
  min-width: 0;
  flex: 0 0 auto;
  align-items: center;
  gap: 5px;
  margin-left: auto;
  overflow: hidden;
}

.card-indicator {
  display: inline-flex;
  min-width: 0;
  max-width: 100%;
  height: 20px;
  align-items: center;
  justify-content: center;
  gap: 4px;
  border: 1px solid var(--panel-card-tag-border, transparent);
  border-radius: 6px;
  padding: 0 7px;
  font: inherit;
  font-size: 11px;
  font-weight: 600;
  line-height: 1;
  overflow: hidden;
  white-space: nowrap;
  cursor: pointer;
  transition: background-color 150ms ease, border-color 150ms ease, color 150ms ease;
}

.card-indicator span {
  overflow: hidden;
  text-overflow: ellipsis;
}

.diff-indicator {
  border-color: var(--panel-card-functional-tag-border, transparent);
  color: var(--panel-card-functional-tag-text, var(--app-primary));
  background: var(--panel-card-functional-tag-background, var(--app-primary-ring));
  box-shadow: 0 1px 3px rgb(30 58 86 / 18%);
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.06em;
}

.diff-indicator:hover {
  border-color: var(--panel-card-functional-tag-hover-border, var(--panel-card-functional-tag-border, transparent));
  background: var(--panel-card-functional-tag-hover-background, var(--panel-card-functional-tag-background, var(--app-primary-ring)));
}

.diff-indicator-right {
  border-color: var(--panel-card-diff-right-border, var(--panel-card-functional-tag-border));
  color: var(--panel-card-diff-right-text, var(--panel-card-functional-tag-text));
  background: var(--panel-card-diff-right-background, var(--panel-card-functional-tag-background));
}

.diff-indicator-right:hover {
  border-color: var(--panel-card-diff-right-hover-border, var(--panel-card-diff-right-border));
  background: var(--panel-card-diff-right-hover-background, var(--panel-card-diff-right-background));
}

.diff-indicator:focus-visible { outline: 2px solid var(--app-primary); outline-offset: 1px; }

.format-label {
  display: inline-flex;
  flex: 0 0 auto;
  align-items: center;
  gap: 6px;
  color: inherit;
  font-size: 12px;
  font-weight: 500;
  line-height: 1;
  white-space: nowrap;
}

.card-header-selected .quick-key {
  border-color: var(--card-header-selected-key-border, var(--app-primary));
  color: var(--card-header-selected-key-text, var(--app-primary));
  background: var(--card-header-selected-key-background, var(--app-menu-background));
}

@media (prefers-reduced-motion: reduce) {
  .card-header,
  .quick-key,
  .card-indicator {
    transition: none;
  }
}
</style>
