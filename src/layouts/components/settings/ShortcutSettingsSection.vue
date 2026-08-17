<script setup lang="ts">
import { computed } from "vue";
import type { AppSettings } from "../../types/settings";
import ShortcutCaptureRow from "./ShortcutCaptureRow.vue";

const settings = defineModel<AppSettings>({ required: true });
type ShortcutField =
  | "mainShortcut"
  | "previousFilterShortcut"
  | "nextFilterShortcut"
  | "previousCardShortcut"
  | "nextCardShortcut";

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
  </div>
</template>

<style scoped>
.settings-stack {
  display: grid;
  gap: 12px;
}
</style>
