<script setup lang="ts">
import { useTemplateRef } from "vue";

const searchQuery = defineModel<string>({ required: true });
const emit = defineEmits<{
  submit: [];
}>();
const searchInput = useTemplateRef<HTMLInputElement>("searchInput");

function focus() {
  searchInput.value?.focus();
  searchInput.value?.select();
}

function handleEnter(event: KeyboardEvent) {
  if (event.isComposing) {
    return;
  }
  event.preventDefault();
  emit("submit");
}

defineExpose({ focus });
</script>

<template>
  <label class="search-box">
    <svg class="search-icon" viewBox="0 0 20 20" aria-hidden="true">
      <circle cx="8.5" cy="8.5" r="5.5" fill="none" stroke="currentColor" stroke-width="1.6" />
      <path d="m12.5 12.5 4 4" fill="none" stroke="currentColor" stroke-linecap="round" stroke-width="1.6" />
    </svg>
    <input
      ref="searchInput"
      v-model="searchQuery"
      class="search-input"
      type="search"
      aria-label="搜索剪贴板历史"
      placeholder="搜索历史内容"
      autocomplete="off"
      spellcheck="false"
      @keydown.enter="handleEnter"
    />
  </label>
</template>

<style scoped>
.search-box {
  display: flex;
  box-sizing: border-box;
  width: 100%;
  height: 30px;
  align-items: center;
  gap: 7px;
  border: 1px solid var(--panel-search-border, #d8dee8);
  border-radius: 8px;
  padding: 0 10px;
  color: var(--panel-search-icon, #94a3b8);
  background: var(--panel-control, rgba(249, 249, 250, 0.56));
  box-shadow: var(--panel-search-shadow, 0 1px 2px rgba(15, 23, 42, 0.04));
  backdrop-filter: var(--panel-control-backdrop, none);
  -webkit-backdrop-filter: var(--panel-control-backdrop, none);
  transition: border-color 0.16s ease, box-shadow 0.16s ease;
}

.search-box:focus-within {
  border-color: var(--panel-search-focus-border, #60a5fa);
  box-shadow: var(--panel-search-focus-shadow, 0 0 0 2px rgba(96, 165, 250, 0.16));
}

.search-icon {
  width: 15px;
  height: 15px;
  flex: 0 0 auto;
}

.search-input {
  width: 100%;
  min-width: 0;
  appearance: none;
  border: 0;
  padding: 0;
  color: var(--panel-search-text, #334155);
  background: transparent;
  font: inherit;
  font-size: 12px;
  outline: none;
  -webkit-appearance: none;
}

.search-input::placeholder {
  color: var(--panel-search-placeholder, #9ca3af);
}

.search-input::-webkit-search-cancel-button {
  cursor: default;
}
</style>
