<script setup lang="ts">
import { computed } from "vue";
import FormatterActionIcon from "./FormatterActionIcon.vue";
import { formatterShortcutLabel } from "../formatterShortcuts";
import { LINE_WRAPPING_SHORTCUT_LABEL } from "../../../utils/toolWindowShortcuts";
import type { FormatterToolbarAction, TextTransformAction } from "../types";

interface ToolbarOption {
  readonly action: FormatterToolbarAction;
  readonly label: string;
}

interface Props {
  supportedActions: readonly TextTransformAction[];
  supportsFolding: boolean;
  statusMessage: string;
  lineWrapping: boolean;
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
  { action: "url-encode", label: "URL 编码" },
  { action: "url-decode", label: "URL 解码" },
  { action: "base64-encode", label: "Base64 编码" },
  { action: "base64-decode", label: "Base64 解码" },
  { action: "date-to-timestamp", label: "转为时间戳" },
  { action: "timestamp-to-date", label: "转为 ISO 日期" },
];

const props = defineProps<Props>();
const emit = defineEmits<{
  action: [action: FormatterToolbarAction];
  toggleLineWrapping: [];
}>();

const visibleOptions = computed(() => TOOLBAR_OPTIONS
  .filter((option) =>
    (props.supportsFolding && (option.action === "fold-all" || option.action === "unfold-all"))
    || isTextTransformAction(option.action) && props.supportedActions.includes(option.action))
  .map((option) => ({
    ...option,
    shortcutLabel: isTextTransformAction(option.action)
      ? formatterShortcutLabel(option.action)
      : undefined,
  })));

function isTextTransformAction(action: FormatterToolbarAction): action is TextTransformAction {
  return action !== "fold-all" && action !== "unfold-all";
}
</script>

<template>
  <div class="formatter-toolbar" role="toolbar" aria-label="格式化工具">
    <button
      v-for="option in visibleOptions"
      :key="option.action"
      type="button"
      class="toolbar-action"
      :aria-label="option.label"
      :title="option.shortcutLabel ? `${option.label} (${option.shortcutLabel})` : option.label"
      @click="emit('action', option.action)"
    >
      <FormatterActionIcon :action="option.action" />
      <kbd v-if="option.shortcutLabel" class="toolbar-shortcut" aria-hidden="true">
        {{ option.shortcutLabel }}
      </kbd>
    </button>
    <label class="toolbar-switch" :title="`切换自动换行 (${LINE_WRAPPING_SHORTCUT_LABEL})`">
      <input
        class="toolbar-switch-input"
        type="checkbox"
        :checked="props.lineWrapping"
        aria-label="自动换行"
        @change="emit('toggleLineWrapping')"
      />
      <span class="toolbar-switch-track" aria-hidden="true">
        <span class="toolbar-switch-thumb"></span>
      </span>
      <span class="toolbar-switch-label">自动换行</span>
      <kbd class="toolbar-shortcut" aria-hidden="true">{{ LINE_WRAPPING_SHORTCUT_LABEL }}</kbd>
    </label>
    <span
      v-if="props.statusMessage" class="toolbar-status" role="status" aria-live="polite"
      :title="props.statusMessage"
    >
      <span class="toolbar-status-mark" aria-hidden="true">✓</span>
      <span class="toolbar-status-text">{{ props.statusMessage }}</span>
    </span>
  </div>
</template>

<style scoped>
.formatter-toolbar {
  display: flex;
  min-width: 0;
  overflow: visible;
  align-items: center;
  flex-wrap: nowrap;
  gap: 4px;
  min-height: 34px;
  padding: 3px 8px;
  border-bottom: 1px solid var(--app-border);
  background: var(--app-surface);
  white-space: nowrap;
}

.toolbar-action {
  box-sizing: border-box;
  display: inline-flex;
  flex: 0 0 auto;
  min-height: 26px;
  align-items: center;
  gap: 4px;
  border: 0;
  border-radius: 8px;
  padding: 4px 6px;
  color: var(--app-secondary);
  background: transparent;
  font: inherit;
  font-size: 13px;
  cursor: pointer;
  transition: color 0.15s ease, background-color 0.15s ease;
}

.toolbar-switch {
  box-sizing: border-box;
  display: inline-flex;
  flex: 0 0 auto;
  min-height: 26px;
  align-items: center;
  gap: 5px;
  padding: 4px 7px;
  color: var(--app-muted);
  font: inherit;
  font-size: 12px;
  cursor: pointer;
}

.toolbar-switch:hover {
  color: var(--app-heading);
}

.toolbar-switch-input {
  position: absolute;
  width: 1px;
  height: 1px;
  opacity: 0;
  pointer-events: none;
}

.toolbar-switch-track {
  display: inline-flex;
  width: 28px;
  height: 16px;
  align-items: center;
  padding: 2px;
  border-radius: 999px;
  background: var(--app-border);
  transition: background-color 0.15s ease;
}

.toolbar-switch-thumb {
  width: 12px;
  height: 12px;
  border-radius: 999px;
  background: var(--app-surface);
  box-shadow: 0 1px 2px rgb(15 23 42 / 20%);
  transform: translateX(0);
  transition: transform 0.15s ease;
}

.toolbar-switch-input:checked + .toolbar-switch-track {
  background: var(--app-primary);
}

.toolbar-switch-input:checked + .toolbar-switch-track .toolbar-switch-thumb {
  transform: translateX(12px);
}

.toolbar-switch-input:focus-visible + .toolbar-switch-track {
  outline: 2px solid var(--app-primary-ring);
  outline-offset: 2px;
}

.toolbar-shortcut {
  border: 1px solid var(--app-border);
  border-radius: 5px;
  padding: 1px 5px;
  color: var(--app-muted);
  background: var(--app-active);
  font-family: inherit;
  font-size: 10px;
  line-height: 1.4;
}

.toolbar-status {
  display: inline-flex;
  flex: 0 1 auto;
  min-width: 0;
  align-items: center;
  gap: 5px;
  margin-left: auto;
  padding: 0 4px;
  color: var(--app-muted);
  font-size: 11px;
  white-space: nowrap;
}

.toolbar-status-text {
  overflow: hidden;
  text-overflow: ellipsis;
}

.toolbar-status-mark {
  flex: 0 0 auto;
  color: var(--app-primary);
  font-size: 12px;
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
