<script setup lang="ts">
import { computed } from "vue";
import { MAX_QUICK_TOOLS } from "../../features/quick-tools/quickToolOrder";
import { formatQuickToolShortcut } from "../../features/quick-tools/quickToolShortcut";
import type { AppSettings } from "../../types/settings";
import ShortcutCaptureRow from "./ShortcutCaptureRow.vue";

const settings = defineModel<AppSettings>({ required: true });
type ShortcutField =
  | "mainShortcut"
  | "previousFilterShortcut"
  | "nextFilterShortcut"
  | "previousCardShortcut"
  | "nextCardShortcut"
  | "sequentialPasteShortcut";

function settingField(key: ShortcutField) {
  return computed({
    get: () => settings.value[key],
    set: (value: string) => {
      settings.value = { ...settings.value, [key]: value };
    },
  });
}

const mainShortcut = settingField("mainShortcut");
const previousFilterShortcut = settingField("previousFilterShortcut");
const nextFilterShortcut = settingField("nextFilterShortcut");
const previousCardShortcut = settingField("previousCardShortcut");
const nextCardShortcut = settingField("nextCardShortcut");
const sequentialPasteShortcut = settingField("sequentialPasteShortcut");

function updateQuickToolShortcut(index: number, value: string) {
  const quickToolShortcuts = [...settings.value.quickToolShortcuts];
  quickToolShortcuts[index] = value;
  settings.value = { ...settings.value, quickToolShortcuts };
}
</script>

<template>
  <div class="settings-stack">
    <ShortcutCaptureRow
      v-model="mainShortcut"
      title="粘贴板快捷键"
      description="点击输入框后按下新的组合键"
    />
    <ShortcutCaptureRow
      v-model="previousFilterShortcut"
      title="上一个标签快捷键"
      description="在粘贴面板中向左切换标签"
    />
    <ShortcutCaptureRow
      v-model="nextFilterShortcut"
      title="下一个标签快捷键"
      description="在粘贴面板中向右切换标签"
    />
    <ShortcutCaptureRow
      v-model="previousCardShortcut"
      title="上一张卡片快捷键"
      description="向左选择卡片，也可以使用左方向键"
    />
    <ShortcutCaptureRow
      v-model="nextCardShortcut"
      title="下一张卡片快捷键"
      description="向右选择卡片，也可以使用右方向键"
    />
    <ShortcutCaptureRow
      v-model="sequentialPasteShortcut"
      title="顺序粘贴快捷键"
      description="粘贴模式下依次粘贴并移除队列内容"
    />
    <ShortcutCaptureRow
      v-for="shortcutIndex in MAX_QUICK_TOOLS"
      :key="shortcutIndex"
      :model-value="settings.quickToolShortcuts[shortcutIndex - 1]"
      :display-value="formatQuickToolShortcut(settings.quickToolShortcuts[shortcutIndex - 1])"
      :title="`快捷工具 ${shortcutIndex}`"
      :description="`执行工具栏第 ${shortcutIndex} 个工具，排序后仍按位置生效`"
      @update:model-value="updateQuickToolShortcut(shortcutIndex - 1, $event)"
    />
  </div>
</template>

<style scoped>
.settings-stack {
  display: grid;
  gap: 12px;
}
</style>
