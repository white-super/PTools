import { invoke } from "@tauri-apps/api/core";

interface UsePanelWindowActionsOptions {
  readonly closeMenus: () => void;
  readonly clearSearch: () => void;
  readonly reportError: (action: string, error: unknown) => void;
}

export function usePanelWindowActions(options: UsePanelWindowActionsOptions) {
  async function openWindow(command: string, action: string, afterOpen?: () => void) {
    options.closeMenus();
    try {
      await invoke(command);
      afterOpen?.();
    } catch (error) {
      options.reportError(action, error);
    }
  }

  return {
    openSequentialPaste: () => openWindow("show_sequential_paste", "open sequential paste"),
    openSettings: () => openWindow("show_settings_window", "open settings", options.clearSearch),
  };
}
