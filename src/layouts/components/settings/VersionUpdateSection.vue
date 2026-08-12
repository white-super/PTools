<script setup lang="ts">
import { computed, onMounted, shallowRef } from "vue";
import { openUrl } from "@tauri-apps/plugin-opener";
import { useAppUpdater } from "../../composables/useAppUpdater";

const GITHUB_RELEASES_URL = "https://github.com/white-super/PTools/releases";

function errorMessage(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}

const {
  availableUpdate,
  checkForUpdates,
  currentVersion,
  error,
  hasChecked,
  installUpdate,
  isChecking,
  isInstalling,
  isUpdateCheckFailed,
  loadVersion,
  versionError,
} = useAppUpdater();

const versionStatus = computed(() => {
  if (availableUpdate.value) {
    return "发现新版本";
  }
  if (hasChecked.value) {
    return "当前已是最新版本";
  }
  return "当前版本";
});

const displayedVersion = computed(() => {
  if (availableUpdate.value) {
    return `v${availableUpdate.value.version}`;
  }
  return currentVersion.value ? `v${currentVersion.value}` : "读取中…";
});

const downloadError = shallowRef<string>();

async function openDownloadPage() {
  downloadError.value = undefined;
  try {
    await openUrl(GITHUB_RELEASES_URL);
  } catch (openError) {
    downloadError.value = errorMessage(openError);
  }
}

onMounted(() => {
  void loadVersion();
  void checkForUpdates();
});
</script>

<template>
  <section class="settings-section">
    <div class="version-row">
      <p class="version-text">{{ versionStatus }}：{{ displayedVersion }}</p>
      <el-button :loading="isChecking" :disabled="isInstalling" @click="checkForUpdates">检查更新</el-button>
      <el-button
        v-if="availableUpdate"
        type="primary"
        :loading="isInstalling"
        :disabled="isChecking"
        @click="installUpdate"
      >
        更新到 v{{ availableUpdate.version }}
      </el-button>
    </div>
    <div v-if="isUpdateCheckFailed" class="download-hint">
      <span>无法检查更新，可前往 GitHub 下载。</span>
      <el-button link type="primary" @click="openDownloadPage">打开 GitHub 下载页</el-button>
    </div>
    <el-alert
      v-if="error || versionError || downloadError"
      class="update-error"
      :title="error || versionError || downloadError"
      type="error"
      :closable="false"
      show-icon
    />
  </section>
</template>

<style scoped>
.settings-section {
  padding: 20px 0;
  border-bottom: 1px solid #e5e7eb;
}

.version-text {
  margin: 0;
}

.version-row {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 12px;
}

.version-text {
  color: #273449;
  font-size: 18px;
  font-weight: 600;
}

.download-hint {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 4px;
  margin-top: 14px;
  color: #718096;
  font-size: 14px;
}

.update-error {
  margin-top: 12px;
}
</style>
