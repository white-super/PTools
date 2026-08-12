import { getVersion } from "@tauri-apps/api/app";
import { relaunch } from "@tauri-apps/plugin-process";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { shallowRef } from "vue";

function errorMessage(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}

export function useAppUpdater() {
  const currentVersion = shallowRef("");
  const availableUpdate = shallowRef<Update | null>(null);
  const isLoadingVersion = shallowRef(true);
  const isChecking = shallowRef(false);
  const isInstalling = shallowRef(false);
  const hasChecked = shallowRef(false);
  const isUpdateCheckFailed = shallowRef(false);
  const error = shallowRef<string>();
  const versionError = shallowRef<string>();

  async function loadVersion() {
    isLoadingVersion.value = true;
    versionError.value = undefined;
    try {
      currentVersion.value = await getVersion();
    } catch (loadError) {
      versionError.value = errorMessage(loadError);
    } finally {
      isLoadingVersion.value = false;
    }
  }

  async function checkForUpdates() {
    isChecking.value = true;
    availableUpdate.value = null;
    hasChecked.value = false;
    isUpdateCheckFailed.value = false;
    error.value = undefined;
    try {
      availableUpdate.value = await check();
      hasChecked.value = true;
    } catch {
      isUpdateCheckFailed.value = true;
    } finally {
      isChecking.value = false;
    }
  }

  async function installUpdate() {
    if (!availableUpdate.value) {
      return;
    }
    isInstalling.value = true;
    error.value = undefined;
    try {
      await availableUpdate.value.downloadAndInstall();
      await relaunch();
    } catch (installError) {
      error.value = errorMessage(installError);
      isInstalling.value = false;
    }
  }

  return {
    availableUpdate,
    checkForUpdates,
    currentVersion,
    error,
    hasChecked,
    installUpdate,
    isChecking,
    isInstalling,
    isLoadingVersion,
    isUpdateCheckFailed,
    loadVersion,
    versionError,
  };
}
