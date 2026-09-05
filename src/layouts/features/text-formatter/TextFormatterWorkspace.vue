<script setup lang="ts">
import { shallowRef, useTemplateRef } from "vue";
import FormatterToolbar from "./components/FormatterToolbar.vue";
import TextCodeEditor from "./components/TextCodeEditor.vue";
import { useTextFormatter } from "./composables/useTextFormatter";
import type { FormatterToolbarAction } from "./types";

interface Props {
  readonly windowId: string;
}

const props = defineProps<Props>();
const editor = useTemplateRef<InstanceType<typeof TextCodeEditor>>("editor");
const lineWrapping = shallowRef(true);
const {
  applyTransform,
  close,
  content,
  editorLanguage,
  format,
  isPinned,
  statusMessage,
  supportedActions,
  title,
  togglePinned,
} = useTextFormatter({
  windowId: props.windowId,
  getSelectedText: () => editor.value?.getSelectedText(),
  replaceSelectedText: (value) => editor.value?.replaceSelectedText(value) ?? false,
});

function handleToolbarAction(action: FormatterToolbarAction) {
  if (action === "fold-all") {
    editor.value?.foldAll();
    return;
  }
  if (action === "unfold-all") {
    editor.value?.unfoldAll();
    return;
  }
  void applyTransform(action);
}

function toggleLineWrapping() {
  lineWrapping.value = !lineWrapping.value;
}

</script>

<template>
  <main class="formatter-page">
    <header class="formatter-header" data-tauri-drag-region>
      <button type="button" class="close-button" aria-label="关闭格式化窗口" title="关闭" @mousedown.stop @click="close">
        <span aria-hidden="true">×</span>
      </button>
      <h1 class="formatter-title" data-tauri-drag-region>{{ title }}</h1>
      <button
        type="button"
        class="pin-button"
        :class="{ 'pin-button-active': isPinned }"
        :aria-label="isPinned ? '取消固定窗口' : '固定窗口'"
        :aria-pressed="isPinned"
        :title="isPinned ? '取消固定窗口' : '固定窗口'"
        @mousedown.stop
        @click="togglePinned"
      >
        <svg class="pin-icon" viewBox="0 0 24 24" aria-hidden="true">
          <path d="M9 3h6l-.8 5 3.3 3v2H13v7l-1 1-1-1v-7H6.5v-2l3.3-3L9 3Z" />
        </svg>
      </button>
    </header>
    <section class="formatter-workspace">
      <FormatterToolbar
        :supported-actions="supportedActions"
        :supports-folding="format === 'json'"
        :status-message="statusMessage"
        :line-wrapping="lineWrapping"
        @action="handleToolbarAction"
        @toggle-line-wrapping="toggleLineWrapping"
      />
      <TextCodeEditor
        ref="editor"
        v-model="content"
        class="formatter-editor"
        :format="format"
        :editor-language="editorLanguage"
        :line-wrapping="lineWrapping"
      />
    </section>
  </main>
</template>

<style scoped>
:global(html),
:global(body),
:global(#app) {
  overflow: hidden;
}

.formatter-page {
  display: grid;
  box-sizing: border-box;
  height: 100%;
  min-height: 0;
  grid-template-rows: auto minmax(0, 1fr);
  color: var(--app-text);
  background: var(--formatter-editor-background);
}

.formatter-header {
  position: relative;
  display: flex;
  min-height: 38px;
  align-items: center;
  justify-content: center;
  border-bottom: 1px solid var(--app-border);
  background: var(--app-surface);
  user-select: none;
}

.formatter-title {
  margin: 0;
  color: var(--app-heading);
  font-size: 15px;
  font-weight: 700;
  letter-spacing: -0.01em;
}

.close-button {
  position: absolute;
  top: 5px;
  left: 8px;
  display: inline-flex;
  width: 28px;
  height: 28px;
  align-items: center;
  justify-content: center;
  padding: 0;
  border: 0;
  border-radius: 7px;
  color: var(--app-muted);
  background: transparent;
  font: inherit;
  font-size: 22px;
  line-height: 1;
  cursor: pointer;
}

.close-button:hover {
  color: var(--app-danger);
  background: var(--app-danger-hover);
}

.close-button:focus-visible {
  outline: 2px solid var(--app-primary-ring);
  outline-offset: 1px;
}

.pin-button {
  position: absolute;
  top: 5px;
  right: 8px;
  display: inline-flex;
  width: 28px;
  height: 28px;
  align-items: center;
  justify-content: center;
  padding: 0;
  border: 0;
  border-radius: 7px;
  color: var(--app-muted);
  background: transparent;
  cursor: pointer;
}

.pin-button:hover {
  color: var(--app-heading);
  background: var(--app-hover);
}

.pin-button:focus-visible {
  outline: 2px solid var(--app-primary-ring);
  outline-offset: 1px;
}

.pin-button-active {
  color: var(--app-primary);
  background: var(--app-active);
}

.pin-icon {
  width: 17px;
  height: 17px;
  fill: none;
  stroke: currentColor;
  stroke-linecap: round;
  stroke-linejoin: round;
  stroke-width: 1.7;
}

.pin-button-active .pin-icon {
  fill: currentColor;
}

.formatter-workspace {
  display: grid;
  min-height: 0;
  overflow: hidden;
  grid-template-rows: auto minmax(0, 1fr);
  background: var(--formatter-editor-background);
}

.formatter-editor {
  min-height: 0;
}
</style>
