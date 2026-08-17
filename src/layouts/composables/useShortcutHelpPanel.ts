import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { onMounted, onUnmounted, readonly, shallowRef } from "vue";

interface UseShortcutHelpPanelOptions {
  readonly reportError: (error: unknown) => void;
}

const VISIBILITY_EVENT = "shortcut-help-visibility-changed";

export function useShortcutHelpPanel(options: UseShortcutHelpPanelOptions) {
  const isHelpOpen = shallowRef(false);
  let disposed = false;
  let unlistenVisibility: UnlistenFn | undefined;

  function setHelpOpen(showHelp: boolean) {
    if (showHelp === isHelpOpen.value) {
      return;
    }
    const command = showHelp ? "show_shortcut_help" : "hide_shortcut_help";
    void invoke(command).catch(options.reportError);
  }

  onMounted(() => {
    void listen<boolean>(VISIBILITY_EVENT, (event) => {
      isHelpOpen.value = event.payload;
    }).then((unlisten) => {
      if (disposed) {
        unlisten();
        return;
      }
      unlistenVisibility = unlisten;
    }).catch(options.reportError);
  });

  onUnmounted(() => {
    disposed = true;
    unlistenVisibility?.();
  });

  return { isHelpOpen: readonly(isHelpOpen), setHelpOpen };
}
