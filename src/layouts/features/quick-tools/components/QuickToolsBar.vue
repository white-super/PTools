<script setup lang="ts">
import { computed, onMounted, onUnmounted, shallowRef, useTemplateRef } from "vue";
import QuickToolManagerPopover from "./QuickToolManagerPopover.vue";
import QuickToolSlot from "./QuickToolSlot.vue";
import { getQuickToolDefinition, QUICK_TOOL_DEFINITIONS } from "../quickToolRegistry";
import { placeQuickTool, removeQuickTool } from "../quickToolOrder";
import type { QuickToolId } from "../types";

interface Props {
  readonly toolIds: readonly QuickToolId[];
  readonly toolShortcuts: readonly string[];
  readonly activeToolIds: readonly QuickToolId[];
  readonly saving: boolean;
}

const props = defineProps<Props>();
const managerOpen = defineModel<boolean>({ required: true });
const emit = defineEmits<{
  execute: [toolId: QuickToolId];
  orderChange: [toolIds: readonly QuickToolId[]];
  error: [error: unknown];
}>();
const root = useTemplateRef<HTMLElement>("root");
const draggedToolId = shallowRef<QuickToolId>();
const dropIndex = shallowRef<number>();
const dropTargetToolId = shallowRef<QuickToolId>();
const visibleTools = computed(() => props.toolIds.map(getQuickToolDefinition));

function closeManager() {
  managerOpen.value = false;
}

function handleOutsidePointer(event: PointerEvent) {
  if (event.target instanceof Node && !root.value?.contains(event.target)) closeManager();
}

function startDrag(toolId: QuickToolId, event: DragEvent) {
  if (props.saving) return;
  draggedToolId.value = toolId;
  event.dataTransfer?.setData("text/plain", toolId);
  if (event.dataTransfer) {
    event.dataTransfer.effectAllowed = props.toolIds.includes(toolId) ? "move" : "copy";
  }
}

function resetDrag() {
  draggedToolId.value = undefined;
  dropIndex.value = undefined;
  dropTargetToolId.value = undefined;
}

function setDropTarget(toolId: QuickToolId, index: number, after: boolean) {
  if (!draggedToolId.value || props.saving) return;
  dropTargetToolId.value = toolId;
  dropIndex.value = index + (after ? 1 : 0);
}

function setEmptyDropTarget(event: DragEvent) {
  if (!draggedToolId.value || props.saving) return;
  event.preventDefault();
  dropIndex.value = 0;
}

function applyOrderChange(nextToolIds: readonly QuickToolId[]) {
  if (nextToolIds.length === props.toolIds.length
    && nextToolIds.every((toolId, index) => toolId === props.toolIds[index])) return;
  emit("orderChange", nextToolIds);
}

function commitDrop() {
  const toolId = draggedToolId.value;
  const targetIndex = dropIndex.value;
  if (!toolId || targetIndex === undefined) return;
  try {
    applyOrderChange(placeQuickTool(props.toolIds, toolId, targetIndex));
  } catch (error) {
    emit("error", error);
  } finally {
    resetDrag();
  }
}

function toggleTool(toolId: QuickToolId) {
  try {
    const nextToolIds = props.toolIds.includes(toolId)
      ? removeQuickTool(props.toolIds, toolId)
      : placeQuickTool(props.toolIds, toolId, props.toolIds.length);
    applyOrderChange(nextToolIds);
  } catch (error) {
    emit("error", error);
  }
}

function execute(toolId: QuickToolId) {
  closeManager();
  emit("execute", toolId);
}

function handleKeydown(event: KeyboardEvent) {
  if (event.key !== "Escape" || !managerOpen.value) return;
  event.preventDefault();
  event.stopPropagation();
  closeManager();
}

onMounted(() => document.addEventListener("pointerdown", handleOutsidePointer, true));
onUnmounted(() => document.removeEventListener("pointerdown", handleOutsidePointer, true));
</script>

<template>
  <div ref="root" class="quick-tools-bar" role="toolbar" aria-label="快捷工具" @click.stop @keydown="handleKeydown">
    <div
      v-if="draggedToolId && visibleTools.length === 0"
      class="empty-drop-slot"
      aria-label="放置快捷工具"
      @dragover="setEmptyDropTarget"
      @drop.prevent="commitDrop"
    ></div>
    <div v-if="visibleTools.length" class="quick-tools-list">
      <QuickToolSlot
        v-for="(tool, index) in visibleTools"
        :key="tool.id"
        :tool="tool"
        :shortcut="props.toolShortcuts[index]"
        :active="props.activeToolIds.includes(tool.id)"
        :dragging="draggedToolId === tool.id"
        :drop-target="dropTargetToolId === tool.id"
        :draggable="!props.saving"
        @activate="execute(tool.id)"
        @drag-start="startDrag(tool.id, $event)"
        @drag-end="resetDrag"
        @drag-over="setDropTarget(tool.id, index, $event)"
        @drop="commitDrop"
      />
    </div>
    <span v-if="visibleTools.length" class="quick-tools-separator" aria-hidden="true"></span>
    <div class="manager-slot">
      <button
        type="button"
        class="manager-trigger"
        :class="{ 'is-active': managerOpen }"
        aria-label="管理快捷工具"
        title="管理快捷工具"
        aria-haspopup="dialog"
        :aria-expanded="managerOpen"
        @click="managerOpen = !managerOpen"
      >
        <svg viewBox="0 0 20 20" aria-hidden="true">
          <rect x="2.5" y="2.5" width="6" height="6" rx="1" />
          <rect x="11.5" y="2.5" width="6" height="6" rx="1" />
          <rect x="2.5" y="11.5" width="6" height="6" rx="1" />
          <rect x="11.5" y="11.5" width="6" height="6" rx="1" />
        </svg>
      </button>
    </div>
    <QuickToolManagerPopover
      v-if="managerOpen"
      :tools="QUICK_TOOL_DEFINITIONS"
      :selected-tool-ids="props.toolIds"
      :shortcuts="props.toolShortcuts"
      :dragged-tool-id="draggedToolId"
      :saving="props.saving"
      @toggle="toggleTool"
      @drag-start="startDrag"
      @drag-end="resetDrag"
    />
  </div>
</template>

<style scoped>
.quick-tools-bar {
  display: flex;
  position: relative;
  min-width: 0;
  height: 28px;
  align-items: flex-start;
  gap: 0;
  border: 0;
  background: transparent;
  box-shadow: none;
}

.quick-tools-list {
  display: flex;
  min-width: 0;
  align-items: flex-start;
  gap: 4px;
}

.empty-drop-slot {
  box-sizing: border-box;
  width: 30px;
  height: 28px;
  flex: 0 0 30px;
  border: 1px dashed var(--quick-tool-neutral-color);
  border-radius: 8px;
}

.quick-tools-separator {
  width: 1px;
  height: 14px;
  margin: 7px 2px 0 3px;
  flex: 0 0 1px;
  background: var(--quick-tool-divider);
}

.manager-slot {
  display: grid;
  width: 30px;
  height: 28px;
  flex: 0 0 30px;
}

.manager-trigger {
  display: inline-flex;
  position: relative;
  box-sizing: border-box;
  width: 30px;
  height: 28px;
  align-items: center;
  justify-content: center;
  padding: 0;
  border: 1px solid transparent;
  border-radius: 8px;
  color: var(--quick-tool-neutral-color);
  background: var(--quick-tool-neutral-background);
  box-shadow: none;
  cursor: pointer;
  transition: transform 80ms ease, border-color 120ms ease, background-color 120ms ease;
}

.manager-trigger svg { width: 15px; height: 15px; fill: none; stroke: currentColor; stroke-width: 1.4; }
.manager-trigger:hover { background: var(--quick-tool-neutral-hover); }
.manager-trigger.is-active { border-color: var(--quick-tool-neutral-border); background: var(--quick-tool-neutral-active); }
.manager-trigger.is-active::after { position: absolute; bottom: 2px; left: 9px; width: 10px; height: 2px; border-radius: 2px; background: currentColor; content: ""; }
.manager-trigger:active { transform: scale(0.955); }
.manager-trigger:focus-visible { outline: 1px solid var(--quick-tool-neutral-color); outline-offset: 1px; }

@media (prefers-reduced-motion: reduce) {
  .manager-trigger { transition: none; }
}
</style>
