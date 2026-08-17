import { invoke } from "@tauri-apps/api/core";
import { shallowRef, watch } from "vue";
import type { AppSettings, HistoryStats } from "../types/settings";

const BYTES_PER_KILOBYTE = 1024;
const BYTES_PER_MEGABYTE = BYTES_PER_KILOBYTE * BYTES_PER_KILOBYTE;

function errorMessage(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}

function settingsSignature(settings: AppSettings) {
  return [
    settings.theme,
    settings.mainShortcut,
    settings.previousFilterShortcut,
    settings.nextFilterShortcut,
    settings.previousCardShortcut,
    settings.nextCardShortcut,
    settings.showFormatFilters,
    settings.historyRetentionDays,
    settings.maxHistoryEntries,
    settings.recordText,
    settings.recordImages,
    settings.recordFiles,
    settings.autoPaste,
  ].join("|");
}

export function formatStorageSize(storageBytes: number) {
  if (storageBytes < BYTES_PER_KILOBYTE) {
    return `${storageBytes} B`;
  }
  if (storageBytes < BYTES_PER_MEGABYTE) {
    return `${(storageBytes / BYTES_PER_KILOBYTE).toFixed(1)} KB`;
  }
  return `${(storageBytes / BYTES_PER_MEGABYTE).toFixed(1)} MB`;
}

export function useAppSettings() {
  const settings = shallowRef<AppSettings>();
  const historyStats = shallowRef<HistoryStats>();
  const isLoading = shallowRef(true);
  const isSaving = shallowRef(false);
  const isClearingHistory = shallowRef(false);
  const error = shallowRef<string>();
  const status = shallowRef<string>();
  const hasLoaded = shallowRef(false);
  let persistedSettingsSignature = "";
  let pendingOperations = 0;
  let operationQueue: Promise<void> = Promise.resolve();

  function applyPersistedSettings(nextSettings: AppSettings) {
    persistedSettingsSignature = settingsSignature(nextSettings);
    settings.value = nextSettings;
  }

  function enqueueSettingsOperation(operation: () => Promise<void>) {
    pendingOperations += 1;
    isSaving.value = true;
    const queuedOperation = operationQueue.then(async () => {
      error.value = undefined;
      try {
        await operation();
      } catch (operationError) {
        error.value = errorMessage(operationError);
        status.value = undefined;
      }
    });
    operationQueue = queuedOperation.finally(() => {
      pendingOperations -= 1;
      isSaving.value = pendingOperations > 0;
    });
    return queuedOperation;
  }

  async function load() {
    isLoading.value = true;
    error.value = undefined;
    try {
      const [nextSettings, nextHistoryStats] = await Promise.all([
        invoke<AppSettings>("get_app_settings"),
        invoke<HistoryStats>("get_history_stats"),
      ]);
      applyPersistedSettings(nextSettings);
      historyStats.value = nextHistoryStats;
      hasLoaded.value = true;
    } catch (loadError) {
      error.value = errorMessage(loadError);
    } finally {
      isLoading.value = false;
    }
  }

  async function save(nextSettings: AppSettings) {
    status.value = "正在自动保存…";
    return enqueueSettingsOperation(async () => {
      const savedSettings = await invoke<AppSettings>("update_app_settings", { settings: nextSettings });
      applyPersistedSettings(savedSettings);
      historyStats.value = await invoke<HistoryStats>("get_history_stats");
      if (pendingOperations === 1) {
        status.value = "已自动保存";
      }
    });
  }

  async function reset() {
    status.value = "正在恢复默认设置…";
    return enqueueSettingsOperation(async () => {
      const defaultSettings = await invoke<AppSettings>("reset_app_settings");
      applyPersistedSettings(defaultSettings);
      historyStats.value = await invoke<HistoryStats>("get_history_stats");
      if (pendingOperations === 1) {
        status.value = "已恢复默认设置";
      }
    });
  }

  async function clearHistory() {
    isClearingHistory.value = true;
    error.value = undefined;
    status.value = undefined;
    try {
      historyStats.value = await invoke<HistoryStats>("clear_clipboard_history");
      status.value = "历史记录已清空";
    } catch (clearError) {
      error.value = errorMessage(clearError);
    } finally {
      isClearingHistory.value = false;
    }
  }

  watch(settings, (nextSettings) => {
    if (!hasLoaded.value || !nextSettings || settingsSignature(nextSettings) === persistedSettingsSignature) {
      return;
    }
    void save(nextSettings);
  });

  return {
    clearHistory,
    error,
    historyStats,
    isClearingHistory,
    isLoading,
    isSaving,
    load,
    reset,
    save,
    settings,
    status,
  };
}
