import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { onMounted, onUnmounted, shallowRef, watch } from "vue";
import { APP_THEME_OPTIONS, DEFAULT_APP_THEME } from "../constants/appThemes";
import type { AppSettings, AppTheme } from "../types/settings";

const SETTINGS_UPDATED_EVENT = "app-settings-updated";
const THEME_CLASSES = APP_THEME_OPTIONS.map((option) => `theme-${option.value}`);

function reportThemeError(error: unknown) {
  console.error("Failed to synchronize application theme", error);
}

export function useAppTheme() {
  const theme = shallowRef<AppTheme>(DEFAULT_APP_THEME);
  let unlistenSettingsUpdate: UnlistenFn | undefined;
  let disposed = false;

  const stopThemeWatch = watch(theme, (nextTheme) => {
    document.documentElement.classList.remove(...THEME_CLASSES);
    document.documentElement.classList.add(`theme-${nextTheme}`);
  }, { immediate: true });

  async function initialize() {
    unlistenSettingsUpdate = await listen<AppSettings>(SETTINGS_UPDATED_EVENT, (event) => {
      theme.value = event.payload.theme;
    });
    if (disposed) {
      unlistenSettingsUpdate();
      return;
    }
    theme.value = (await invoke<AppSettings>("get_app_settings")).theme;
  }

  onMounted(() => {
    void initialize().catch(reportThemeError);
  });

  onUnmounted(() => {
    disposed = true;
    stopThemeWatch();
    unlistenSettingsUpdate?.();
  });

  return { theme };
}
