<script setup lang="ts">
import { shallowRef } from "vue";
import {
  isKeyboardModifierKey,
  keyboardShortcutFromEvent,
} from "../../utils/keyboardShortcut";

interface Props {
  readonly title: string;
  readonly description: string;
}

const props = defineProps<Props>();
const shortcut = defineModel<string>({ required: true });
const captureHint = shallowRef(props.description);

function captureShortcut(event: KeyboardEvent) {
  event.preventDefault();
  if (isKeyboardModifierKey(event)) {
    captureHint.value = "请同时按下一个非修饰键";
    return;
  }
  const capturedShortcut = keyboardShortcutFromEvent(event);
  if (!capturedShortcut) {
    captureHint.value = "快捷键必须包含 Ctrl、Option、Shift 或 Command";
    return;
  }
  shortcut.value = capturedShortcut;
  captureHint.value = "已录入并自动生效";
}
</script>

<template>
  <section class="settings-section">
    <div class="shortcut-row">
      <div class="setting-copy">
        <h2 class="setting-title">{{ props.title }}</h2>
        <p class="field-hint">{{ captureHint }}</p>
      </div>
      <el-input
        v-model="shortcut"
        class="shortcut-input"
        size="small"
        readonly
        :aria-label="props.title"
        @keydown="captureShortcut"
      />
    </div>
  </section>
</template>

<style scoped>
.settings-section,
.setting-copy {
  min-width: 0;
}

.shortcut-input {
  flex: 0 0 180px;
}

.field-hint {
  margin: 6px 0 0;
  color: var(--app-muted);
  font-size: 13px;
}

.shortcut-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 24px;
}

.setting-title {
  margin: 0;
  color: var(--app-heading);
  font-size: 15px;
  font-weight: 650;
}

@media (max-width: 560px) {
  .shortcut-row {
    align-items: flex-start;
    flex-direction: column;
    gap: 12px;
  }

  .shortcut-input {
    width: 180px;
  }
}
</style>
