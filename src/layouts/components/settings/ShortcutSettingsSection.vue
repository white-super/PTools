<script setup lang="ts">
import { computed, shallowRef } from "vue";
import type { AppSettings } from "../../types/settings";

const settings = defineModel<AppSettings>({ required: true });
const captureHint = shallowRef("点击输入框后按下新的组合键");

const mainShortcut = computed({
  get: () => settings.value.mainShortcut,
  set: (mainShortcut: string) => {
    settings.value = { ...settings.value, mainShortcut };
  },
});

function isModifierKey(event: KeyboardEvent) {
  return ["Alt", "Control", "Meta", "Shift"].includes(event.key);
}

function shortcutKey(event: KeyboardEvent) {
  if (event.code.startsWith("Key")) {
    return event.code.slice(3);
  }
  if (event.code.startsWith("Digit")) {
    return event.code.slice(5);
  }
  if (event.code === "Space") {
    return "Space";
  }
  return event.code;
}

function captureShortcut(event: KeyboardEvent) {
  event.preventDefault();
  if (isModifierKey(event)) {
    captureHint.value = "请同时按下一个非修饰键";
    return;
  }
  const modifiers = [
    event.ctrlKey && "Ctrl",
    event.altKey && "Option",
    event.shiftKey && "Shift",
    event.metaKey && "Command",
  ].filter(Boolean);
  if (modifiers.length === 0) {
    captureHint.value = "快捷键必须包含 Ctrl、Option、Shift 或 Command";
    return;
  }
  mainShortcut.value = [...modifiers, shortcutKey(event)].join("+");
  captureHint.value = "已录入并自动生效";
}
</script>

<template>
  <section class="settings-section">
    <div class="shortcut-row">
      <div class="setting-copy">
        <h2 class="setting-title">粘贴板快捷键</h2>
        <p class="field-hint">{{ captureHint }}</p>
      </div>
      <el-input
        v-model="mainShortcut"
        class="shortcut-input"
        size="small"
        readonly
        :aria-label="captureHint"
        @keydown="captureShortcut"
      />
    </div>
  </section>
</template>

<style scoped>
.settings-section {
  min-width: 0;
}

.setting-copy {
  min-width: 0;
}

.shortcut-input {
  flex: 0 0 180px;
}

.field-hint {
  margin: 6px 0 0;
  color: #6b7280;
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
  color: #202124;
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
