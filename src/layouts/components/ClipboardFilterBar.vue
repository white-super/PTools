<script setup lang="ts">
import { shallowRef } from "vue";
import { TAG_COLORS } from "../constants/tagColors";
import type { ClipboardFilter, ClipboardFormat, ClipboardTag, ClipboardTagInput } from "../types/settings";
import TagContextMenu from "./TagContextMenu.vue";
import TagInlineEditor from "./TagInlineEditor.vue";

interface FilterOption {
  readonly key: "all" | ClipboardFormat;
  readonly label: string;
  readonly icon: "all" | ClipboardFormat;
}

interface TagContextMenuState {
  readonly tag: ClipboardTag;
  readonly x: number;
  readonly y: number;
}

interface Props {
  readonly tags: readonly ClipboardTag[];
  readonly activeFilter: ClipboardFilter;
}

const FORMAT_FILTERS: readonly FilterOption[] = [
  { key: "all", label: "全部", icon: "all" },
  { key: "text", label: "文本", icon: "text" },
  { key: "image", label: "图片", icon: "image" },
  { key: "file", label: "文件", icon: "file" },
];
const CONTEXT_MENU_WIDTH = 104;
const CONTEXT_MENU_HEIGHT = 74;
const CONTEXT_MENU_MARGIN = 8;

const props = defineProps<Props>();
const emit = defineEmits<{
  filterChange: [filter: ClipboardFilter];
  createTag: [input: ClipboardTagInput];
  updateTag: [id: number, input: ClipboardTagInput];
  deleteTag: [id: number];
}>();
const contextMenu = shallowRef<TagContextMenuState>();
const editingTagId = shallowRef<number>();
const isCreatingTag = shallowRef(false);
const editorName = shallowRef("");
const editorColor = shallowRef<string>(TAG_COLORS[0]);

function closeTagContextMenu() {
  contextMenu.value = undefined;
}

function selectFilter(filter: ClipboardFilter) {
  closeTagContextMenu();
  emit("filterChange", filter);
}

function openTagContextMenu(tag: ClipboardTag, event: MouseEvent) {
  contextMenu.value = {
    tag,
    x: Math.min(
      event.clientX,
      Math.max(CONTEXT_MENU_MARGIN, window.innerWidth - CONTEXT_MENU_WIDTH - CONTEXT_MENU_MARGIN),
    ),
    y: Math.min(
      event.clientY,
      Math.max(CONTEXT_MENU_MARGIN, window.innerHeight - CONTEXT_MENU_HEIGHT - CONTEXT_MENU_MARGIN),
    ),
  };
}

function openTagEditor(tag?: ClipboardTag) {
  closeTagContextMenu();
  editingTagId.value = tag?.id;
  editorName.value = tag?.name ?? "";
  editorColor.value = normalizeTagColor(tag?.color);
  isCreatingTag.value = !tag;
}

function normalizeTagColor(color?: string) {
  const normalizedColor = color?.toUpperCase();
  return TAG_COLORS.find((tagColor) => tagColor === normalizedColor) ?? TAG_COLORS[0];
}

function saveTag(input: ClipboardTagInput) {
  if (editingTagId.value === undefined) {
    emit("createTag", input);
  } else {
    emit("updateTag", editingTagId.value, input);
  }
  cancelTagEditor();
}

function cancelTagEditor() {
  editingTagId.value = undefined;
  isCreatingTag.value = false;
}

function deleteTag() {
  const tag = contextMenu.value?.tag;
  closeTagContextMenu();
  if (tag) {
    emit("deleteTag", tag.id);
  }
}

function editContextTag() {
  const tag = contextMenu.value?.tag;
  if (tag) {
    openTagEditor(tag);
  }
}

defineExpose({ closeTagContextMenu });
</script>

<template>
  <nav class="filter-list" aria-label="按剪贴板格式或标签筛选" @click="closeTagContextMenu">
    <button
      v-for="filter in FORMAT_FILTERS"
      :key="filter.key"
      type="button"
      class="filter-button"
      :class="{ 'filter-button-active': props.activeFilter === filter.key }"
      @click="selectFilter(filter.key)"
    >
      <svg class="filter-icon" viewBox="0 0 16 16" aria-hidden="true">
        <path v-if="filter.icon === 'all'" d="M2.5 2.5h4v4h-4zm7 0h4v4h-4zm-7 7h4v4h-4zm7 0h4v4h-4z" />
        <path v-if="filter.icon === 'text'" d="M2.5 3.5h11v1.5h-11zm0 3.75h8v1.5h-8zm0 3.75h11v1.5h-11z" />
        <path v-if="filter.icon === 'image'" d="M2.5 3h11v10h-11zm1.5 7 2.25-2.25 1.5 1.5 1.75-2.25L12 10zM5 5.5a1 1 0 1 0 0 2 1 1 0 0 0 0-2z" />
        <path v-if="filter.icon === 'file'" d="M4 2.5h5l3 3v8H4zm4.5 0v3h3" fill="none" stroke="currentColor" stroke-width="1.5" />
      </svg>
      {{ filter.label }}
    </button>
    <template v-for="tag in props.tags" :key="tag.id">
      <TagInlineEditor
        v-if="editingTagId === tag.id"
        :name="editorName"
        :color="editorColor"
        mode="edit"
        @save="saveTag"
        @cancel="cancelTagEditor"
      />
      <button
        v-else
        type="button"
        class="filter-button filter-button-tag"
        :class="{ 'filter-button-active': props.activeFilter === tag.id }"
        @click="selectFilter(tag.id)"
        @contextmenu.prevent.stop="openTagContextMenu(tag, $event)"
      >
        <span class="tag-color" :style="{ backgroundColor: tag.color }"></span>
        {{ tag.name }}
      </button>
    </template>
    <TagInlineEditor
      v-if="isCreatingTag"
      :name="editorName"
      :color="editorColor"
      mode="create"
      @save="saveTag"
      @cancel="cancelTagEditor"
    />
    <button type="button" class="add-tag-button" aria-label="新增标签" title="新增标签" @click="openTagEditor()">
      +
    </button>
    <TagContextMenu
      v-if="contextMenu"
      :x="contextMenu.x"
      :y="contextMenu.y"
      @edit="editContextTag"
      @delete="deleteTag"
    />
  </nav>
</template>

<style scoped>
.filter-list {
  display: flex;
  flex: 0 0 auto;
  gap: 2px;
  overflow-x: auto;
  scrollbar-width: none;
  user-select: none;
  -webkit-user-select: none;
}

.filter-list::-webkit-scrollbar {
  display: none;
}

.filter-button,
.add-tag-button {
  display: inline-flex;
  flex: 0 0 auto;
  align-items: center;
  appearance: none;
  border: 0;
  border-radius: 6px;
  padding: 5px 9px;
  color: #6b7280;
  background: transparent;
  font: inherit;
  font-size: 12px;
  outline: none;
  cursor: default;
  transition: color 0.16s ease, background 0.16s ease;
  user-select: none;
  -webkit-appearance: none;
  -webkit-user-select: none;
}

.add-tag-button {
  width: 26px;
  justify-content: center;
  padding: 0;
  color: #64748b;
  font-size: 17px;
  line-height: 1;
}

.filter-button:hover,
.add-tag-button:hover {
  color: #374151;
  background: #eef2f7;
}

.filter-button-active {
  color: #334155;
  background: #e5eaf1;
}

.filter-button-tag {
  gap: 5px;
}

.filter-icon {
  width: 13px;
  height: 13px;
  fill: currentcolor;
}

.tag-color {
  width: 7px;
  height: 7px;
  flex: 0 0 auto;
  border-radius: 999px;
}
</style>
