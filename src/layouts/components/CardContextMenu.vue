<script setup lang="ts">
import type { ClipboardTag } from "../types/settings";

interface Props {
  x: number;
  y: number;
  tags: readonly ClipboardTag[];
  assignedTagIds: readonly number[];
}

const props = defineProps<Props>();
const emit = defineEmits<{
  toggleTag: [tagId: number];
  delete: [];
}>();
</script>

<template>
  <div
    class="context-menu"
    :style="{ left: `${props.x}px`, top: `${props.y}px` }"
    role="menu"
    @click.stop
  >
    <template v-if="props.tags.length > 0">
      <p class="context-menu-label">标签</p>
      <button
        v-for="tag in props.tags"
        :key="tag.id"
        type="button"
        class="context-menu-item"
        :class="{ 'context-menu-item-selected': props.assignedTagIds.includes(tag.id) }"
        @click="emit('toggleTag', tag.id)"
      >
        <span class="tag-color" :style="{ backgroundColor: tag.color }"></span>
        <span>{{ tag.name }}</span>
        <span class="tag-selection">{{ props.assignedTagIds.includes(tag.id) ? "✓" : "" }}</span>
      </button>
      <div class="context-menu-divider"></div>
    </template>
    <button type="button" class="context-menu-item context-menu-item-danger" @click="emit('delete')">
      删除
    </button>
  </div>
</template>

<style scoped>
.context-menu {
  position: fixed;
  z-index: 20;
  min-width: 140px;
  max-height: calc(100vh - 16px);
  overflow-y: auto;
  padding: 4px;
  border: 1px solid #e5e7eb;
  border-radius: 8px;
  background: #ffffff;
  box-shadow: 0 8px 24px rgb(15 23 42 / 0.14);
}

.context-menu-item {
  display: flex;
  width: 100%;
  align-items: center;
  gap: 7px;
  border: 0;
  border-radius: 5px;
  padding: 7px 9px;
  color: #334155;
  background: transparent;
  font: inherit;
  font-size: 12px;
  text-align: left;
  cursor: default;
}

.context-menu-item:hover {
  background: #f1f5f9;
}

.context-menu-item-selected {
  background: #f8fafc;
}

.context-menu-label {
  margin: 3px 5px 5px;
  color: #94a3b8;
  font-size: 11px;
}

.context-menu-divider {
  height: 1px;
  margin: 4px;
  background: #eef2f7;
}

.tag-color {
  width: 8px;
  height: 8px;
  flex: 0 0 auto;
  border-radius: 999px;
}

.tag-selection {
  min-width: 10px;
  margin-left: auto;
  color: #4f7cff;
  text-align: right;
}

.context-menu-item-danger {
  color: #dc2626;
}

.context-menu-item-danger:hover {
  background: #fef2f2;
}
</style>
