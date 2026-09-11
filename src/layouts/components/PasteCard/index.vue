<script setup lang="ts">
import CardContent from "./CardContent.vue";
import CardHeader from "./CardHeader.vue";
import CardNotice from "./CardNotice.vue";
import type { PanelNoticeState } from "../../composables/usePanelNotice";
import type { DiffSide } from "../../features/text-diff/types";
import type { ClipboardFormat } from "../../types/settings";

interface Props {
  content: string;
  format: ClipboardFormat;
  isSelected: boolean;
  filePaths: readonly string[];
  quickKey?: number;
  diffSide?: DiffSide;
  notice?: PanelNoticeState;
}

const props = defineProps<Props>();
const emit = defineEmits<{
  select: [];
  paste: [];
  contextMenu: [event: MouseEvent];
  cancelDiff: [];
}>();

function handleDoubleClick() {
  emit("paste");
}
</script>

<template>
  <div
    class="card"
    :class="{ 'card-selected': props.isSelected }"
    @click="emit('select')"
    @dblclick.prevent="handleDoubleClick"
    @contextmenu.prevent="emit('contextMenu', $event)"
  >
    <CardHeader
      :format="format"
      :quick-key="quickKey"
      :diff-side="diffSide"
      :is-selected="props.isSelected"
      @cancel-diff="emit('cancelDiff')"
    />
    <CardContent :content="content" :format="format" :file-paths="filePaths" />
    <div class="card-notice-host">
      <CardNotice :notice="notice" />
    </div>
  </div>
</template>

<style scoped>
.card {
  position: relative;
  display: flex;
  box-sizing: border-box;
  height: 100%;
  overflow: hidden;
  flex-direction: column;
  border: 1px solid var(--panel-card-border, #e5e7eb);
  border-radius: 14px;
  padding: 12px;
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", "PingFang SC", "Microsoft YaHei", sans-serif;
  -webkit-font-smoothing: antialiased;
  color: var(--panel-card-text, #374151);
  background: var(--panel-card-background, #ffffff);
  box-shadow: var(--panel-card-shadow, 0 1px 2px rgba(15, 23, 42, 0.06));
  backdrop-filter: var(--panel-card-backdrop, none);
  -webkit-backdrop-filter: var(--panel-card-backdrop, none);
  cursor: default;
  --card-header-background-current: var(--panel-card-header-background);
  --card-header-border-current: var(--panel-card-header-border);
  --card-header-text-current: var(--panel-card-header-text);
  --card-header-key-background: var(--panel-card-header-key-background);
  --card-header-key-border: var(--panel-card-header-key-border);
  --card-header-key-text: var(--panel-card-header-key-text);
  --card-header-selected-key-background: var(--panel-card-header-selected-key-background);
  --card-header-selected-key-border: var(--panel-card-header-selected-key-border);
  --card-header-selected-key-text: var(--panel-card-header-selected-key-text);
  transition: border-color 150ms ease, box-shadow 150ms ease, background-color 150ms ease;
  user-select: none;
  -webkit-user-select: none;
}

.card :deep(*) {
  user-select: none;
  -webkit-user-select: none;
}

.card-notice-host {
  position: absolute;
  right: 9px;
  bottom: 9px;
  left: 9px;
  z-index: 4;
  pointer-events: none;
}

.card:hover {
  --card-header-background-current: var(--panel-card-header-hover-background, var(--panel-card-header-background));
  --card-header-border-current: var(--panel-card-header-hover-border, var(--panel-card-header-border));
  --card-header-text-current: var(--panel-card-header-hover-text, var(--panel-card-header-text));
  border-color: var(--panel-card-hover-border, #bcc7d2);
  background: var(--panel-card-hover-background, #ffffff);
  box-shadow: var(--panel-card-hover-shadow, 0 8px 18px rgba(35, 48, 62, 0.11));
  z-index: 1;
}

.card-selected {
  --card-header-background-current: var(--panel-card-header-selected-background, var(--panel-card-header-background));
  --card-header-border-current: var(--panel-card-header-selected-border, var(--panel-card-header-border));
  --card-header-text-current: var(--panel-card-header-selected-text, var(--panel-card-header-text));
  border-color: var(--panel-card-selected-border, #347dc2);
  background: var(--panel-card-selected-background, var(--panel-card-background, #fbfcfd));
  box-shadow: var(--panel-card-selected-shadow, 0 0 0 3px rgba(52, 125, 194, 0.18), 0 7px 18px rgba(39, 76, 112, 0.12));
  z-index: 1;
}

.card-selected:hover {
  --card-header-background-current: var(--panel-card-header-selected-background, var(--panel-card-header-background));
  --card-header-border-current: var(--panel-card-header-selected-border, var(--panel-card-header-border));
  --card-header-text-current: var(--panel-card-header-selected-text, var(--panel-card-header-text));
  background: var(--panel-card-selected-background, var(--panel-card-background, #fbfcfd));
  box-shadow: var(--panel-card-selected-hover-shadow, 0 0 0 3px rgba(52, 125, 194, 0.18), 0 9px 21px rgba(39, 76, 112, 0.14));
  z-index: 2;
}

.card:active {
  box-shadow: var(--panel-card-pressed-shadow, 0 1px 2px rgba(25, 35, 45, 0.06));
}

.card-selected:active,
.card-selected:hover:active {
  box-shadow: var(--panel-card-selected-pressed-shadow, 0 0 0 2px rgba(52, 125, 194, 0.14), 0 2px 5px rgba(39, 76, 112, 0.08));
}

@media (prefers-reduced-motion: reduce) {
  .card {
    transition: border-color 0.01ms linear, box-shadow 0.01ms linear, background-color 0.01ms linear;
  }
}
</style>
