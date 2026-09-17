<script setup lang="ts">
import QuickToolIcon from "./QuickToolIcon.vue";
import type { QuickToolDefinition } from "../types";

interface Props {
  readonly tool: QuickToolDefinition;
  readonly shortcut: string;
  readonly active: boolean;
  readonly dragging: boolean;
  readonly dropTarget: boolean;
  readonly draggable: boolean;
}

const props = defineProps<Props>();
const emit = defineEmits<{
  activate: [];
  dragStart: [event: DragEvent];
  dragEnd: [];
  dragOver: [after: boolean];
  drop: [];
}>();

function handleDragOver(event: DragEvent) {
  event.preventDefault();
  const slot = event.currentTarget as HTMLElement;
  emit("dragOver", event.clientX > slot.getBoundingClientRect().left + slot.offsetWidth / 2);
}
</script>

<template>
  <div
    class="quick-tool-slot"
    :class="{
      'is-active': props.active,
      'is-dragging': props.dragging,
      'is-drop-target': props.dropTarget,
    }"
    :data-tone="props.tool.tone"
    :data-tool-id="props.tool.id"
    @dragover="handleDragOver"
    @drop.prevent="emit('drop')"
  >
    <button
      type="button"
      class="quick-tool-button"
      :draggable="props.draggable"
      :aria-label="`${props.tool.label}，快捷键 ${props.shortcut}`"
      :aria-pressed="props.active"
      @click="emit('activate')"
      @dragstart="emit('dragStart', $event)"
      @dragend="emit('dragEnd')"
    >
      <QuickToolIcon :tool-id="props.tool.id" />
    </button>
    <span class="quick-tool-tooltip" aria-hidden="true">
      {{ props.tool.label }}（{{ props.shortcut }}）
    </span>
  </div>
</template>

<style scoped>
.quick-tool-slot {
  --tool-color: var(--quick-tool-blue-color);
  --tool-background: var(--quick-tool-blue-background);
  display: grid;
  position: relative;
  box-sizing: border-box;
  width: 30px;
  height: 28px;
  flex: 0 0 30px;
  color: var(--tool-color);
  transition: opacity 120ms ease;
  user-select: none;
}

.quick-tool-slot[data-tone="purple"] { --tool-color: var(--quick-tool-purple-color); --tool-background: var(--quick-tool-purple-background); }
.quick-tool-slot[data-tone="orange"] { --tool-color: var(--quick-tool-orange-color); --tool-background: var(--quick-tool-orange-background); }
.quick-tool-slot[data-tone="cyan"] { --tool-color: var(--quick-tool-cyan-color); --tool-background: var(--quick-tool-cyan-background); }
.quick-tool-slot[data-tone="green"] { --tool-color: var(--quick-tool-green-color); --tool-background: var(--quick-tool-green-background); }

.quick-tool-button {
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
  color: var(--tool-color);
  background: var(--tool-background);
  box-shadow: none;
  cursor: pointer;
  transition: transform 80ms ease, border-color 120ms ease, background-color 120ms ease, color 120ms ease;
}

.quick-tool-slot:hover .quick-tool-button {
  border-color: color-mix(in srgb, var(--tool-color) 30%, transparent);
  color: color-mix(in srgb, var(--tool-color) 88%, var(--app-heading));
  background: color-mix(in srgb, var(--tool-background) 84%, var(--tool-color));
}

.quick-tool-slot.is-active .quick-tool-button {
  border-color: color-mix(in srgb, var(--tool-color) 56%, transparent);
  background: color-mix(in srgb, var(--tool-background) 75%, var(--tool-color));
}

.quick-tool-slot.is-active .quick-tool-button::after {
  position: absolute;
  bottom: 2px;
  left: 50%;
  width: 10px;
  height: 2px;
  border-radius: 2px;
  background: currentColor;
  content: "";
  transform: translateX(-50%);
}

.quick-tool-button:active { transform: scale(0.955); }
.quick-tool-button:focus-visible { outline: 1px solid var(--tool-color); outline-offset: 1px; }
.quick-tool-slot.is-dragging { opacity: 0.45; }
.quick-tool-slot.is-drop-target .quick-tool-button { border-style: dashed; border-color: var(--tool-color); }

.quick-tool-tooltip {
  position: absolute;
  z-index: 20;
  top: calc(100% + 5px);
  left: 50%;
  padding: 3px 7px;
  border: 1px solid var(--app-border);
  border-radius: 5px;
  opacity: 0;
  color: var(--app-text);
  background: var(--app-menu-background);
  font-size: 11px;
  line-height: 1.4;
  pointer-events: none;
  white-space: nowrap;
  transform: translate(-50%, -2px);
  transition: opacity 60ms ease, transform 60ms ease;
}

.quick-tool-slot:hover,
.quick-tool-slot:focus-within {
  z-index: 20;
}

.quick-tool-slot:hover .quick-tool-tooltip,
.quick-tool-button:focus-visible + .quick-tool-tooltip {
  opacity: 1;
  transform: translate(-50%, 0);
  transition-delay: 100ms;
}

@media (prefers-reduced-motion: reduce) {
  .quick-tool-slot,
  .quick-tool-button,
  .quick-tool-tooltip { transition: none; }
}
</style>
