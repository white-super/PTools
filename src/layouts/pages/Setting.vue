<script setup lang="ts">
import { ElMessageBox } from "element-plus";
import { onMounted, shallowRef } from "vue";
import BehaviorSettingsSection from "../components/settings/BehaviorSettingsSection.vue";
import HistorySettingsSection from "../components/settings/HistorySettingsSection.vue";
import SettingsNavigation from "../components/settings/SettingsNavigation.vue";
import ShortcutSettingsSection from "../components/settings/ShortcutSettingsSection.vue";
import SystemPermissionSection from "../components/settings/SystemPermissionSection.vue";
import ThemeSettingsSection from "../components/settings/ThemeSettingsSection.vue";
import VersionUpdateSection from "../components/settings/VersionUpdateSection.vue";
import { useAppSettings } from "../composables/useAppSettings";

const {
  clearHistory,
  error,
  historyStats,
  isClearingHistory,
  isLoading,
  isSaving,
  load,
  reset,
  settings,
  status,
} = useAppSettings();

type SettingsTab = "system" | "updates";

const activeTab = shallowRef<SettingsTab>("system");

function isConfirmationCancelled(error: unknown) {
  return error === "cancel" || error === "close";
}

async function confirmClearHistory() {
  try {
    await ElMessageBox.confirm("清空后无法恢复历史内容。", "确认清空全部历史", {
      confirmButtonText: "清空",
      cancelButtonText: "取消",
      type: "warning",
    });
  } catch (error) {
    if (isConfirmationCancelled(error)) {
      return;
    }
    throw error;
  }
  await clearHistory();
}

async function confirmResetSettings() {
  try {
    await ElMessageBox.confirm("恢复默认值会立即替换当前快捷键和界面设置。", "确认恢复默认设置", {
      confirmButtonText: "恢复默认",
      cancelButtonText: "取消",
      type: "warning",
    });
  } catch (error) {
    if (isConfirmationCancelled(error)) {
      return;
    }
    throw error;
  }
  await reset();
}

onMounted(() => {
  void load();
});
</script>

<template>
  <main class="settings-page">
    <div class="settings-shell">
      <aside class="settings-sidebar">
        <SettingsNavigation v-model="activeTab" />
      </aside>
      <section class="settings-main">
        <header class="page-header">
          <h1 class="page-title">设置</h1>
        </header>
        <template v-if="activeTab === 'system'">
          <el-alert v-if="error" class="settings-alert" :title="error" type="error" :closable="false" show-icon />
          <div v-if="isLoading" class="loading-state">正在读取本机设置…</div>
          <el-form v-else-if="settings && historyStats" class="settings-form" label-position="top">
            <ThemeSettingsSection v-model="settings" />
            <ShortcutSettingsSection v-model="settings" />
            <HistorySettingsSection
              v-model="settings"
              :history-stats="historyStats"
              :is-clearing="isClearingHistory"
              @clear="confirmClearHistory"
            />
            <BehaviorSettingsSection v-model="settings" />
            <SystemPermissionSection />
            <footer class="settings-footer">
              <span class="save-status">{{ status || "设置已生效" }}</span>
              <el-button :disabled="isSaving" @click="confirmResetSettings">恢复默认</el-button>
            </footer>
          </el-form>
        </template>
        <div v-else class="update-page">
          <VersionUpdateSection />
        </div>
      </section>
    </div>
  </main>
</template>

<style scoped>
.settings-page {
  height: 100%;
  color: var(--app-text);
  background: var(--app-background);
}

.settings-shell {
  display: grid;
  grid-template-columns: 220px minmax(0, 1fr);
  height: 100%;
}

.settings-sidebar {
  box-sizing: border-box;
  padding: 20px 12px;
  border-right: 1px solid var(--app-border);
  background: var(--settings-sidebar-background);
}

.settings-main {
  min-width: 0;
  overflow-y: auto;
  padding: 28px 36px 40px;
  background: var(--settings-main-background);
  scrollbar-gutter: stable;
}

.page-header {
  padding-bottom: 22px;
}

.page-title {
  margin: 0;
  color: var(--app-heading);
  font-size: 24px;
  font-weight: 700;
  letter-spacing: -0.02em;
}

.settings-alert,
.loading-state,
.settings-form,
.update-page {
  max-width: 860px;
}

.settings-alert {
  margin-bottom: 14px;
}

.loading-state {
  padding: 28px 0;
  color: var(--app-muted);
  font-size: 14px;
}

.settings-form {
  display: grid;
  gap: 12px;
}

.update-page {
  display: grid;
}

.settings-main :deep(.settings-section) {
  display: flex;
  align-items: center;
  box-sizing: border-box;
  min-height: 84px;
  padding: 18px 24px;
  border: 0;
  border-radius: 12px;
  background: var(--settings-section-background);
  box-shadow: none;
}

.settings-main :deep(.setting-row),
.settings-main :deep(.shortcut-row),
.settings-main :deep(.behavior-row),
.settings-main :deep(.permission-row),
.settings-main :deep(.version-row) {
  width: 100%;
}

.settings-main :deep(.el-input__wrapper),
.settings-main :deep(.el-select__wrapper) {
  border-radius: 10px;
  background: var(--settings-control-background);
  box-shadow: 0 0 0 1px var(--settings-control-border) inset;
}

.settings-main :deep(.el-input-number) {
  overflow: hidden;
  border-radius: 10px;
}

.settings-main :deep(.el-input-number .el-input__wrapper) {
  box-shadow: none;
}

.settings-main :deep(.el-switch) {
  --el-switch-on-color: var(--app-success);
}

.settings-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 2px 0;
}

.save-status {
  color: var(--app-muted);
  font-size: 13px;
}

@media (max-width: 720px) {
  .settings-shell {
    grid-template-columns: minmax(0, 1fr);
    grid-template-rows: auto minmax(0, 1fr);
  }

  .settings-sidebar {
    border-right: 0;
    border-bottom: 1px solid var(--app-border);
  }
}

@media (max-width: 640px) {
  .settings-main {
    padding: 22px 20px 32px;
  }
}
</style>
