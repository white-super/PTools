<script setup lang="ts">
import type { FormatterToolbarAction, TextTransformAction } from "../types";

interface ToolbarOption {
  readonly action: FormatterToolbarAction;
  readonly label: string;
}

interface Props {
  supportedActions: readonly TextTransformAction[];
}

const TOOLBAR_OPTIONS: readonly ToolbarOption[] = [
  { action: "format", label: "重新格式化" },
  { action: "fold-all", label: "折叠全部" },
  { action: "unfold-all", label: "展开全部" },
  { action: "remove-comments", label: "去除注释" },
  { action: "minify-copy", label: "压缩并复制" },
  { action: "escape-copy", label: "压缩转义并复制" },
  { action: "to-xml-copy", label: "JSON 转 XML 并复制" },
  { action: "to-typescript-copy", label: "JSON 转 TypeScript 并复制" },
];

const props = defineProps<Props>();
const emit = defineEmits<{ action: [action: FormatterToolbarAction] }>();

function isDisabled(action: FormatterToolbarAction) {
  if (action === "fold-all" || action === "unfold-all") {
    return false;
  }
  return !props.supportedActions.includes(action);
}
</script>

<template>
  <footer class="formatter-toolbar">
    <button
      v-for="option in TOOLBAR_OPTIONS"
      :key="option.action"
      type="button"
      class="toolbar-action"
      :disabled="isDisabled(option.action)"
      @click="emit('action', option.action)"
    >
      {{ option.label }}
    </button>
  </footer>
</template>

<style scoped>
.formatter-toolbar {
  display: flex;
  min-width: 0;
  overflow-x: auto;
  align-items: center;
  gap: 4px;
  padding: 10px 12px;
  border-top: 1px solid var(--app-border);
  background: var(--app-surface);
  scrollbar-width: thin;
}

.toolbar-action {
  flex: 0 0 auto;
  border: 0;
  border-radius: 8px;
  padding: 8px 10px;
  color: var(--app-secondary);
  background: transparent;
  font: inherit;
  font-size: 13px;
  cursor: pointer;
  transition: color 0.15s ease, background-color 0.15s ease;
}

.toolbar-action:hover:not(:disabled) {
  color: var(--app-heading);
  background: var(--app-hover);
}

.toolbar-action:focus-visible {
  outline: 2px solid var(--app-primary-ring);
  outline-offset: 1px;
}

.toolbar-action:disabled {
  color: var(--app-muted);
  cursor: not-allowed;
  opacity: 0.55;
}
</style>
