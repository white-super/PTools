<script setup lang="ts">
import { computed } from "vue";
import type { SequentialPasteItem } from "../types";

interface Props {
  readonly item: SequentialPasteItem;
  readonly index: number;
  readonly editable: boolean;
  readonly next: boolean;
  readonly error?: string;
  readonly dragging: boolean;
  readonly dropTarget: boolean;
}

const props = defineProps<Props>();
const emit = defineEmits<{
  dragStart: [event: DragEvent];
  dragEnd: [];
  dragOver: [];
  drop: [payload: { readonly event: DragEvent; readonly after: boolean }];
  remove: [];
}>();
const formatLabel = computed(() => ({ text: "文本", image: "图片", file: "文件" })[props.item.format]);
const imageSource = computed(() => props.item.content.startsWith("data:image/")
  ? props.item.content
  : `data:image/png;base64,${props.item.content}`);
const preview = computed(() => {
  if (props.item.format === "file") {
    return props.item.filePaths
      .map((path) => {
        const segments = path.split(/[/\\]/);
        return segments[segments.length - 1] || path;
      })
      .join(" · ");
  }
  return props.item.content.trim() || "空文本";
});

function handleDragOver(event: DragEvent) {
  if (!props.editable) return;
  event.preventDefault();
  if (event.dataTransfer) event.dataTransfer.dropEffect = "move";
  emit("dragOver");
}

function handleDrop(event: DragEvent) {
  const row = event.currentTarget as HTMLElement;
  const after = event.clientY > row.getBoundingClientRect().top + row.offsetHeight / 2;
  emit("drop", { event, after });
}
</script>

<template>
  <li
    class="queue-item"
    :class="{
      'is-next': props.next,
      'is-dragging': props.dragging,
      'is-drop-target': props.dropTarget,
      'has-error': !!props.error,
    }"
    :draggable="props.editable"
    :data-item-id="props.item.id"
    @dragstart="emit('dragStart', $event)"
    @dragend="emit('dragEnd')"
    @dragover="handleDragOver"
    @drop.prevent="handleDrop"
  >
    <span v-if="props.editable" class="drag-handle" aria-hidden="true">
      <svg viewBox="0 0 16 16"><circle cx="5" cy="4" r="1" /><circle cx="11" cy="4" r="1" /><circle cx="5" cy="8" r="1" /><circle cx="11" cy="8" r="1" /><circle cx="5" cy="12" r="1" /><circle cx="11" cy="12" r="1" /></svg>
    </span>
    <span v-else class="queue-index" :aria-label="`队列第 ${props.index + 1} 项`">
      <svg v-if="props.next" viewBox="0 0 16 16" aria-hidden="true"><path d="m5 3 6 5-6 5z" /></svg>
      <span v-else>{{ props.index + 1 }}</span>
    </span>
    <div class="item-body">
      <div class="item-meta">
        <span>{{ formatLabel }}</span>
        <span v-if="props.item.format === 'file'">{{ props.item.filePaths.length }} 个</span>
        <span v-if="props.next" class="next-label">下一项</span>
      </div>
      <img v-if="props.item.format === 'image'" class="item-image" :src="imageSource" alt="队列图片" draggable="false" />
      <p v-else class="item-preview">{{ preview }}</p>
      <p v-if="props.error" class="item-error" role="alert">{{ props.error }}</p>
    </div>
    <button
      v-if="props.editable"
      type="button"
      class="remove-button"
      aria-label="从队列移除"
      title="移除"
      @click.stop="emit('remove')"
    >
      <svg viewBox="0 0 20 20" aria-hidden="true"><path d="m6 6 8 8M14 6l-8 8" /></svg>
    </button>
  </li>
</template>

<style scoped>
.queue-item {
  display: grid;
  position: relative;
  box-sizing: border-box;
  min-height: 64px;
  grid-template-columns: 24px minmax(0, 1fr) 26px;
  align-items: center;
  gap: 7px;
  padding: 8px 9px;
  border-bottom: 1px solid var(--app-border);
  color: var(--app-text);
  background: color-mix(in srgb, var(--app-surface-elevated) 72%, transparent);
  transition: background-color 120ms ease, border-color 120ms ease, opacity 120ms ease;
}
.queue-item:hover { background: var(--app-surface-elevated); }
.queue-item.is-next { background: color-mix(in srgb, var(--app-primary-ring) 65%, var(--app-surface-elevated)); }
.queue-item.is-next::before { position: absolute; inset: 7px auto 7px 0; width: 3px; border-radius: 0 3px 3px 0; background: var(--app-primary); content: ""; }
.queue-item.has-error { background: color-mix(in srgb, var(--app-danger-hover) 80%, var(--app-surface-elevated)); }
.queue-item.is-dragging { opacity: 0.42; }
.queue-item.is-drop-target { border-top: 1px dashed var(--app-primary); }
.drag-handle { display: inline-flex; width: 24px; height: 28px; align-items: center; justify-content: center; color: var(--app-muted); cursor: grab; }
.drag-handle svg { width: 15px; height: 15px; fill: currentColor; }
.queue-index { display: inline-flex; width: 22px; height: 22px; align-items: center; justify-content: center; border: 1px solid var(--app-border); border-radius: 6px; color: var(--app-muted); background: var(--app-surface); font: 10px ui-monospace, monospace; }
.queue-index svg { width: 12px; height: 12px; fill: var(--app-primary); }
.item-body { min-width: 0; }
.item-meta { display: flex; min-width: 0; align-items: center; gap: 7px; color: var(--app-muted); font-size: 10px; }
.next-label { margin-left: auto; color: var(--app-primary); font-weight: 650; }
.item-preview { display: -webkit-box; overflow: hidden; margin: 4px 0 0; color: var(--app-text); font-size: 12px; line-height: 1.45; overflow-wrap: anywhere; white-space: pre-wrap; -webkit-box-orient: vertical; -webkit-line-clamp: 2; }
.item-image { display: block; width: 100%; height: 44px; margin-top: 5px; border-radius: 5px; object-fit: cover; object-position: center; background: var(--app-surface); }
.item-error { margin: 5px 0 0; color: var(--app-danger); font-size: 10px; line-height: 1.35; overflow-wrap: anywhere; }
.remove-button { display: inline-flex; width: 24px; height: 24px; align-items: center; justify-content: center; padding: 0; border: 0; border-radius: 5px; opacity: 0; color: var(--app-muted); background: transparent; cursor: pointer; transition: opacity 100ms ease, color 100ms ease, background-color 100ms ease; }
.queue-item:hover .remove-button, .remove-button:focus-visible { opacity: 1; }
.remove-button:hover { color: var(--app-danger); background: var(--app-danger-hover); }
.remove-button:focus-visible { outline: 2px solid var(--app-primary-ring); }
.remove-button svg { width: 14px; height: 14px; fill: none; stroke: currentColor; stroke-linecap: round; stroke-width: 1.7; }
</style>
