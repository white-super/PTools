<script setup lang="ts">
import QuickToolIcon from "./QuickToolIcon.vue";
import { MAX_QUICK_TOOLS } from "../quickToolOrder";
import { formatQuickToolShortcut } from "../quickToolShortcut";
import type { QuickToolDefinition, QuickToolId } from "../types";

interface Props {
  readonly tools: readonly QuickToolDefinition[];
  readonly selectedToolIds: readonly QuickToolId[];
  readonly shortcuts: readonly string[];
  readonly draggedToolId?: QuickToolId;
  readonly saving: boolean;
}

const props = defineProps<Props>();
const emit = defineEmits<{
  toggle: [toolId: QuickToolId];
  dragStart: [toolId: QuickToolId, event: DragEvent];
  dragEnd: [];
}>();

function shortcutForTool(toolId: QuickToolId) {
  const index = props.selectedToolIds.indexOf(toolId);
  const shortcut = props.shortcuts[index];
  return index < 0 || !shortcut ? undefined : formatQuickToolShortcut(shortcut);
}

function isAddDisabled(toolId: QuickToolId) {
  return props.saving
    || (!props.selectedToolIds.includes(toolId) && props.selectedToolIds.length >= MAX_QUICK_TOOLS);
}

</script>

<template>
  <section class="quick-tool-manager" role="dialog" aria-label="管理快捷工具">
    <header class="manager-header">
      <strong>快捷工具</strong>
      <span>拖动排序，最多 {{ MAX_QUICK_TOOLS }} 个</span>
    </header>
    <div class="manager-grid">
      <button
        v-for="tool in props.tools"
        :key="tool.id"
        type="button"
        class="manager-tool"
        :class="{
          'is-selected': props.selectedToolIds.includes(tool.id),
          'is-dragging': props.draggedToolId === tool.id,
        }"
        :data-tone="tool.tone"
        :data-tool-id="tool.id"
        :disabled="isAddDisabled(tool.id)"
        :draggable="!isAddDisabled(tool.id)"
        :aria-pressed="props.selectedToolIds.includes(tool.id)"
        :title="props.selectedToolIds.includes(tool.id) ? `移除${tool.label}` : `添加${tool.label}`"
        @click="emit('toggle', tool.id)"
        @dragstart="emit('dragStart', tool.id, $event)"
        @dragend="emit('dragEnd')"
      >
        <span class="manager-icon"><QuickToolIcon :tool-id="tool.id" /></span>
        <span class="manager-label">{{ tool.label }}</span>
        <kbd v-if="shortcutForTool(tool.id)" class="manager-shortcut">{{ shortcutForTool(tool.id) }}</kbd>
      </button>
    </div>
  </section>
</template>

<style scoped>
.quick-tool-manager {
  display: grid;
  position: absolute;
  top: calc(100% + 6px);
  right: 0;
  z-index: 60;
  box-sizing: border-box;
  width: min(310px, calc(100vw - 20px));
  gap: 8px;
  padding: 10px;
  border: 1px solid var(--quick-tool-manager-border);
  border-radius: 11px;
  color: var(--app-text);
  background: var(--quick-tool-manager-background);
  box-shadow: none;
}

.manager-header {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 10px;
}

.manager-header strong { color: var(--app-heading); font-size: 12px; font-weight: 650; }
.manager-header span { color: var(--app-muted); font-size: 10px; }

.manager-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 4px;
}

.manager-tool {
  --tool-color: var(--quick-tool-blue-color);
  --tool-background: var(--quick-tool-blue-background);
  display: grid;
  min-width: 0;
  height: 34px;
  grid-template-columns: 24px minmax(0, 1fr) auto;
  align-items: center;
  gap: 6px;
  padding: 3px 6px;
  border: 1px solid transparent;
  border-radius: 8px;
  color: var(--app-secondary);
  background: transparent;
  box-shadow: none;
  font: inherit;
  cursor: pointer;
  transition: border-color 120ms ease, background-color 120ms ease, opacity 120ms ease;
}

.manager-tool[data-tone="purple"] { --tool-color: var(--quick-tool-purple-color); --tool-background: var(--quick-tool-purple-background); }
.manager-tool[data-tone="orange"] { --tool-color: var(--quick-tool-orange-color); --tool-background: var(--quick-tool-orange-background); }
.manager-tool[data-tone="cyan"] { --tool-color: var(--quick-tool-cyan-color); --tool-background: var(--quick-tool-cyan-background); }
.manager-tool[data-tone="green"] { --tool-color: var(--quick-tool-green-color); --tool-background: var(--quick-tool-green-background); }

.manager-tool:hover:not(:disabled),
.manager-tool.is-selected {
  border-color: color-mix(in srgb, var(--tool-color) 30%, transparent);
  background: color-mix(in srgb, var(--tool-background) 70%, transparent);
}

.manager-tool:focus-visible { outline: 1px solid var(--tool-color); outline-offset: -1px; }
.manager-tool:disabled { cursor: not-allowed; opacity: 0.4; }
.manager-tool.is-dragging { opacity: 0.45; }

.manager-icon {
  display: inline-flex;
  width: 24px;
  height: 24px;
  align-items: center;
  justify-content: center;
  border-radius: 6px;
  color: var(--tool-color);
  background: var(--tool-background);
}

.manager-label { overflow: hidden; font-size: 11px; text-align: left; text-overflow: ellipsis; white-space: nowrap; }
.manager-shortcut { border: 0; color: var(--app-muted); background: transparent; font: 9px ui-monospace, monospace; }
</style>
