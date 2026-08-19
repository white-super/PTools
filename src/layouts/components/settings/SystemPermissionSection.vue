<script setup lang="ts">
import { computed, onMounted, onUnmounted } from "vue";
import { useSystemPermission } from "../../composables/useSystemPermission";

type PermissionTagType = "success" | "warning" | "info";

const {
  error,
  isLoading,
  isOpeningSettings,
  openPermissionSettings,
  permissionStatus,
  refreshPermissionStatus,
} = useSystemPermission();

const shouldShowPermissionSection = computed(() =>
  error.value !== undefined || permissionStatus.value?.accessibilityPermissionSupported === true,
);

const permissionStatusText = computed(() => {
  if (!permissionStatus.value?.accessibilityPermissionSupported) {
    return "当前系统不支持";
  }
  return permissionStatus.value.accessibilityPermissionGranted ? "已开启" : "未开启";
});

const permissionStatusType = computed<PermissionTagType>(() => {
  if (!permissionStatus.value?.accessibilityPermissionSupported) {
    return "info";
  }
  return permissionStatus.value.accessibilityPermissionGranted ? "success" : "warning";
});

function refreshWhenWindowFocused() {
  if (permissionStatus.value?.accessibilityPermissionSupported === false) return;
  void refreshPermissionStatus();
}

onMounted(() => {
  refreshWhenWindowFocused();
  window.addEventListener("focus", refreshWhenWindowFocused);
});

onUnmounted(() => {
  window.removeEventListener("focus", refreshWhenWindowFocused);
});
</script>

<template>
  <section v-if="shouldShowPermissionSection" class="settings-section">
    <div v-if="isLoading" class="permission-loading">正在检查系统权限…</div>
    <template v-else>
      <el-alert v-if="error" class="permission-error" :title="error" type="error" :closable="false" show-icon />
      <div v-if="permissionStatus" class="permission-row">
        <div class="permission-copy">
          <h2 class="permission-title">辅助功能权限</h2>
          <p class="permission-detail">{{ permissionStatus.systemName }}：用于响应全局快捷键和粘贴操作。</p>
          <p
            v-if="permissionStatus.accessibilityPermissionSupported && !permissionStatus.accessibilityPermissionGranted"
            class="permission-detail permission-warning"
          >
            请在系统设置中开启辅助功能权限。
          </p>
          <p class="permission-detail" v-else-if="!permissionStatus.accessibilityPermissionSupported">
            当前系统暂不支持此权限校验和设置跳转。
          </p>
        </div>
        <div class="permission-actions">
          <el-tag size="small" :type="permissionStatusType" effect="light">{{ permissionStatusText }}</el-tag>
          <el-button
            v-if="permissionStatus.accessibilityPermissionSupported && !permissionStatus.accessibilityPermissionGranted"
            type="primary"
            size="small"
            :loading="isOpeningSettings"
            @click="openPermissionSettings"
          >
            打开系统设置
          </el-button>
          <el-button size="small" :loading="isLoading" @click="refreshPermissionStatus">重新检查</el-button>
        </div>
      </div>
    </template>
  </section>
</template>

<style scoped>
.permission-detail,
.permission-loading {
  margin: 6px 0 0;
  color: var(--app-muted);
  font-size: 13px;
}

.permission-title {
  margin: 0;
  color: var(--app-heading);
  font-size: 15px;
  font-weight: 650;
}

.permission-error {
  margin-bottom: 12px;
}

.permission-row {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 24px;
}

.permission-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.permission-copy {
  min-width: 0;
}

.permission-warning {
  color: var(--app-warning);
}

@media (max-width: 720px) {
  .permission-row {
    align-items: flex-start;
    flex-direction: column;
    gap: 12px;
  }
}
</style>
