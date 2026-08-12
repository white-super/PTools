<script setup lang="ts">
import { computed } from "vue";
import { formatStorageSize } from "../../composables/useAppSettings";
import type { AppSettings, HistoryStats } from "../../types/settings";

interface Props {
  historyStats: HistoryStats;
  isClearing: boolean;
}

const props = defineProps<Props>();
const emit = defineEmits<{ clear: [] }>();
const settings = defineModel<AppSettings>({ required: true });

function updateSettings(partialSettings: Partial<AppSettings>) {
  settings.value = { ...settings.value, ...partialSettings };
}

const historyRetentionDays = computed({
  get: () => settings.value.historyRetentionDays,
  set: (historyRetentionDays: number) => updateSettings({ historyRetentionDays }),
});

const maxHistoryEntries = computed({
  get: () => settings.value.maxHistoryEntries,
  set: (maxHistoryEntries: number) => updateSettings({ maxHistoryEntries }),
});

const recordText = computed({
  get: () => settings.value.recordText,
  set: (recordText: boolean) => updateSettings({ recordText }),
});

const recordImages = computed({
  get: () => settings.value.recordImages,
  set: (recordImages: boolean) => updateSettings({ recordImages }),
});

const recordFiles = computed({
  get: () => settings.value.recordFiles,
  set: (recordFiles: boolean) => updateSettings({ recordFiles }),
});
</script>

<template>
  <div class="settings-stack">
    <section class="settings-section">
      <div class="setting-row">
        <div class="setting-copy">
          <h2 class="setting-title">保留时间</h2>
          <p class="setting-description">超过保留时间的历史记录会自动删除。</p>
        </div>
        <el-select v-model="historyRetentionDays" class="select-control" size="small" aria-label="保留时间">
          <el-option label="7 天" :value="7" />
          <el-option label="30 天" :value="30" />
          <el-option label="90 天" :value="90" />
          <el-option label="永久保留" :value="0" />
        </el-select>
      </div>
    </section>
    <section class="settings-section">
      <div class="setting-row">
        <div class="setting-copy">
          <h2 class="setting-title">最大记录数</h2>
          <p class="setting-description">达到上限后，最早的记录会被替换。</p>
        </div>
        <el-input-number
          v-model="maxHistoryEntries"
          class="number-control"
          size="small"
          :min="1"
          controls-position="right"
          aria-label="最大记录数"
        />
      </div>
    </section>
    <section class="settings-section">
      <div class="setting-row">
        <div class="setting-copy">
          <h2 class="setting-title">记录类型</h2>
          <p class="setting-description">选择需要保存的剪贴板内容类型。</p>
        </div>
        <div class="record-types">
          <label class="record-type">
            <span>文本</span>
            <el-switch v-model="recordText" size="small" aria-label="记录文本" />
          </label>
          <label class="record-type">
            <span>图片</span>
            <el-switch v-model="recordImages" size="small" aria-label="记录图片" />
          </label>
          <label class="record-type">
            <span>文件</span>
            <el-switch v-model="recordFiles" size="small" aria-label="记录文件" />
          </label>
        </div>
      </div>
    </section>
    <section class="settings-section">
      <div class="setting-row">
        <div class="setting-copy">
          <h2 class="setting-title">清空历史记录</h2>
          <p class="setting-description">
            当前共 {{ props.historyStats.count }} 条记录，占用 {{ formatStorageSize(props.historyStats.storageBytes) }}。
          </p>
        </div>
        <el-button size="small" :loading="props.isClearing" @click="emit('clear')">清空</el-button>
      </div>
    </section>
  </div>
</template>

<style scoped>
.settings-stack {
  display: grid;
  gap: 12px;
}

.select-control {
  flex: 0 0 150px;
}

.number-control {
  flex: 0 0 120px;
}

.setting-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 24px;
}

.record-types {
  display: flex;
  align-items: center;
  gap: 16px;
}

.record-type {
  display: flex;
  align-items: center;
  gap: 6px;
  color: #5f6368;
  font-size: 12px;
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

@media (max-width: 640px) {
  .setting-row {
    align-items: flex-start;
    flex-direction: column;
    gap: 12px;
  }

  .select-control,
  .number-control {
    width: 150px;
  }
}
</style>
