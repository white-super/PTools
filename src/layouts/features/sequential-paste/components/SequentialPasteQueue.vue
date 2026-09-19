<script setup lang="ts">
import { shallowRef } from "vue";
import SequentialPasteItem from "./SequentialPasteItem.vue";
import type { SequentialPasteError, SequentialPasteItem as QueueItem } from "../types";

const DRAG_END_GRACE_MS = 250;

interface Props {
  readonly items: readonly QueueItem[];
  readonly editable: boolean;
  readonly nextItemId?: number;
  readonly error?: SequentialPasteError;
}

interface DropPayload {
  readonly event: DragEvent;
  readonly after: boolean;
}

const props = defineProps<Props>();
const emit = defineEmits<{
  reorder: [itemIds: readonly number[]];
  remove: [itemId: number];
}>();
const draggedId = shallowRef<number>();
const dropTargetId = shallowRef<number>();
let lastDraggedId: number | undefined;
let dragEndedAt: number | undefined;

function startDrag(itemId: number, event: DragEvent) {
  if (!props.editable) return;
  draggedId.value = itemId;
  lastDraggedId = itemId;
  dragEndedAt = undefined;
  if (event.dataTransfer) {
    event.dataTransfer.setData("text/plain", String(itemId));
    event.dataTransfer.effectAllowed = "move";
  }
}

function setDropTarget(itemId: number) {
  if (!props.editable || draggedId.value === undefined) return;
  dropTargetId.value = itemId;
}

function resetDragDisplay() {
  draggedId.value = undefined;
  dropTargetId.value = undefined;
}

function queueContains(itemId: number | undefined): itemId is number {
  return itemId !== undefined && props.items.some((item) => item.id === itemId);
}

function draggedItemId(event: DragEvent) {
  const transferredId = Number(event.dataTransfer?.getData("text/plain"));
  if (Number.isSafeInteger(transferredId) && queueContains(transferredId)) return transferredId;
  if (!queueContains(lastDraggedId)) return undefined;
  if (dragEndedAt === undefined) return lastDraggedId;
  return performance.now() - dragEndedAt <= DRAG_END_GRACE_MS ? lastDraggedId : undefined;
}

function finishDrag() {
  dragEndedAt = performance.now();
  resetDragDisplay();
}

function finishDrop() {
  lastDraggedId = undefined;
  dragEndedAt = undefined;
  resetDragDisplay();
}

function commitDrop(targetId: number, payload: DropPayload) {
  const sourceId = draggedItemId(payload.event);
  if (sourceId === undefined || sourceId === targetId) {
    finishDrop();
    return;
  }
  const nextIds = props.items.map((item) => item.id).filter((id) => id !== sourceId);
  const targetIndex = nextIds.indexOf(targetId);
  if (targetIndex < 0) {
    finishDrop();
    return;
  }
  nextIds.splice(targetIndex + (payload.after ? 1 : 0), 0, sourceId);
  emit("reorder", nextIds);
  finishDrop();
}

function itemError(itemId: number) {
  return props.error?.itemId === itemId ? props.error.message : undefined;
}
</script>

<template>
  <ol v-if="props.items.length" class="queue-list" aria-label="顺序粘贴队列">
    <SequentialPasteItem
      v-for="(item, index) in props.items"
      :key="item.id"
      :item="item"
      :index="index"
      :editable="props.editable"
      :next="props.nextItemId === item.id"
      :error="itemError(item.id)"
      :dragging="draggedId === item.id"
      :drop-target="dropTargetId === item.id"
      @drag-start="startDrag(item.id, $event)"
      @drag-end="finishDrag"
      @drag-over="setDropTarget(item.id)"
      @drop="commitDrop(item.id, $event)"
      @remove="emit('remove', item.id)"
    />
  </ol>
  <div v-else class="empty-queue" role="status">
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <path d="M6 7h12M6 12h8M6 17h5" />
      <path d="M18 14v6M15 17h6" />
    </svg>
    <strong>{{ props.editable ? "等待采集" : "队列已完成" }}</strong>
  </div>
</template>

<style scoped>
.queue-list { min-height: 0; margin: 0; padding: 0; list-style: none; }
.empty-queue { display: flex; height: 100%; min-height: 220px; align-items: center; justify-content: center; flex-direction: column; gap: 7px; color: var(--app-muted); text-align: center; }
.empty-queue svg { width: 30px; height: 30px; margin-bottom: 3px; fill: none; stroke: currentColor; stroke-linecap: round; stroke-linejoin: round; stroke-width: 1.35; }
.empty-queue strong { color: var(--app-secondary); font-size: 13px; font-weight: 600; }
</style>
