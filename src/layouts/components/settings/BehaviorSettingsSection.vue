<script setup lang="ts">
import { computed } from "vue";
import type { AppSettings } from "../../types/settings";

const settings = defineModel<AppSettings>({ required: true });

const autoPaste = computed({
  get: () => settings.value.autoPaste,
  set: (autoPaste: boolean) => {
    settings.value = { ...settings.value, autoPaste };
  },
});
</script>

<template>
  <section class="settings-section">
    <div class="behavior-row">
      <div class="setting-copy">
        <h2 class="setting-title">双击后自动粘贴</h2>
        <p class="setting-description">关闭后，双击只会将内容写入系统剪贴板。</p>
      </div>
      <el-switch v-model="autoPaste" size="small" aria-label="双击后自动粘贴" />
    </div>
  </section>
</template>

<style scoped>
.behavior-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 20px;
}

.setting-copy {
  min-width: 0;
}

.setting-title,
.setting-description {
  margin: 0;
}

.setting-title {
  color: #202124;
  font-size: 15px;
  font-weight: 650;
}

.setting-description {
  margin-top: 6px;
  color: #6b7280;
  font-size: 13px;
}
</style>
