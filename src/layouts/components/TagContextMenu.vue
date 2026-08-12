<script setup lang="ts">
interface Props {
  x: number;
  y: number;
}

const props = defineProps<Props>();
const emit = defineEmits<{
  edit: [];
  delete: [];
}>();
</script>

<template>
  <div
    class="tag-context-menu"
    :style="{ left: `${props.x}px`, top: `${props.y}px` }"
    role="menu"
    @click.stop
  >
    <button type="button" class="tag-context-menu-item" @click="emit('edit')">编辑</button>
    <button type="button" class="tag-context-menu-item tag-context-menu-item-danger" @click="emit('delete')">
      删除
    </button>
  </div>
</template>

<style scoped>
.tag-context-menu {
  position: fixed;
  z-index: 20;
  min-width: 104px;
  padding: 4px;
  border: 1px solid #e5e7eb;
  border-radius: 8px;
  background: #ffffff;
  box-shadow: 0 8px 24px rgb(15 23 42 / 0.14);
}

.tag-context-menu-item {
  display: block;
  width: 100%;
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

.tag-context-menu-item:hover {
  background: #f1f5f9;
}

.tag-context-menu-item-danger {
  color: #dc2626;
}

.tag-context-menu-item-danger:hover {
  background: #fef2f2;
}
</style>
