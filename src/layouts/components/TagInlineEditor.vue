<script setup lang="ts">
import { computed, onMounted, shallowRef, useTemplateRef } from "vue";
import { TAG_COLORS } from "../constants/tagColors";
import type { ClipboardTagInput } from "../types/settings";

interface Props {
  readonly name: string;
  readonly color: string;
  readonly mode: "create" | "edit";
}

const props = defineProps<Props>();
const emit = defineEmits<{
  save: [input: ClipboardTagInput];
  cancel: [];
}>();
const name = shallowRef(props.name);
const color = shallowRef(props.color);
const isValid = computed(() => name.value.trim().length > 0);
const nameInput = useTemplateRef<HTMLInputElement>("nameInput");

function save() {
  if (isValid.value) {
    emit("save", { name: name.value.trim(), color: color.value });
  }
}

onMounted(() => {
  nameInput.value?.focus();
});
</script>

<template>
  <div class="tag-inline-editor" @click.stop>
    <input
      v-model="name"
      ref="nameInput"
      class="tag-name-input"
      maxlength="32"
      :placeholder="props.mode === 'create' ? '新标签' : '标签名称'"
      aria-label="标签名称"
      @keydown.enter.prevent="save"
      @keydown.esc.prevent="emit('cancel')"
    />
    <div class="tag-color-list" aria-label="标签颜色">
      <button
        v-for="tagColor in TAG_COLORS"
        :key="tagColor"
        type="button"
        class="tag-color-option"
        :class="{ 'tag-color-option-active': color === tagColor }"
        :style="{ backgroundColor: tagColor }"
        :aria-label="`选择颜色 ${tagColor}`"
        @click="color = tagColor"
      ></button>
    </div>
    <button
      type="button"
      class="tag-editor-action tag-editor-save"
      :disabled="!isValid"
      aria-label="保存标签"
      @click="save"
    >
      ✓
    </button>
    <button type="button" class="tag-editor-action" aria-label="取消编辑" @click="emit('cancel')">
      ×
    </button>
  </div>
</template>

<style scoped>
.tag-inline-editor {
  display: inline-flex;
  flex: 0 0 auto;
  align-items: center;
  gap: 5px;
  border: 1px solid #cbd5e1;
  border-radius: 7px;
  padding: 2px 5px;
  background: #ffffff;
  box-shadow: 0 2px 8px rgb(15 23 42 / 0.08);
}

.tag-name-input {
  width: 72px;
  border: 0;
  outline: none;
  color: #334155;
  background: transparent;
  font: inherit;
  font-size: 12px;
  user-select: text;
  -webkit-user-select: text;
}

.tag-color-list {
  display: inline-flex;
  align-items: center;
  gap: 3px;
}

.tag-color-option {
  width: 10px;
  height: 10px;
  flex: 0 0 auto;
  border: 1px solid transparent;
  border-radius: 999px;
  padding: 0;
  cursor: default;
}

.tag-color-option-active {
  border-color: #1f2937;
  box-shadow: 0 0 0 1px #ffffff;
}

.tag-editor-action {
  width: 18px;
  height: 18px;
  border: 0;
  border-radius: 4px;
  padding: 0;
  color: #64748b;
  background: transparent;
  font-size: 13px;
  line-height: 18px;
  cursor: default;
}

.tag-editor-action:hover {
  background: #f1f5f9;
}

.tag-editor-action:disabled {
  color: #cbd5e1;
  background: transparent;
}

.tag-editor-save {
  color: #2563eb;
}
</style>
