import { invoke } from "@tauri-apps/api/core";
import { shallowRef } from "vue";
import type { SystemPermissionStatus } from "../types/settings";

function errorMessage(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}

export function useSystemPermission() {
  const permissionStatus = shallowRef<SystemPermissionStatus>();
  const error = shallowRef<string>();
  const isLoading = shallowRef(true);
  const isOpeningSettings = shallowRef(false);

  async function refreshPermissionStatus() {
    isLoading.value = true;
    error.value = undefined;
    try {
      permissionStatus.value = await invoke<SystemPermissionStatus>("get_system_permission_status");
    } catch (statusError) {
      error.value = errorMessage(statusError);
    } finally {
      isLoading.value = false;
    }
  }

  async function openPermissionSettings() {
    isOpeningSettings.value = true;
    error.value = undefined;
    try {
      await invoke("open_system_permission_settings");
      await refreshPermissionStatus();
    } catch (openError) {
      error.value = errorMessage(openError);
    } finally {
      isOpeningSettings.value = false;
    }
  }

  return {
    error,
    isLoading,
    isOpeningSettings,
    openPermissionSettings,
    permissionStatus,
    refreshPermissionStatus,
  };
}
