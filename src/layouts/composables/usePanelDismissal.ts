import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { onMounted, onUnmounted } from "vue";

const ESCAPE_KEY = "Escape";
const MAIN_PANEL_BLUR_EVENT = "ptools://main-panel-blur";
const MAIN_PANEL_DISMISS_EVENT = "ptools://main-panel-dismiss";

function reportDismissalError(error: unknown) {
  console.error("Failed to hide the main panel", error);
}

interface UsePanelDismissalOptions {
  readonly beforeDismiss?: () => void;
}

export function usePanelDismissal(options: UsePanelDismissalOptions = {}) {
  let disposed = false;
  let unlistenBlur: UnlistenFn | undefined;
  let unlistenDismiss: UnlistenFn | undefined;

  function dismissPanel() {
    options.beforeDismiss?.();
    return invoke<void>("hide_main_panel");
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key !== ESCAPE_KEY || event.defaultPrevented
      || (event.target instanceof Element && event.target.closest(".el-overlay"))) {
      return;
    }

    event.preventDefault();
    void dismissPanel().catch(reportDismissalError);
  }

  onMounted(() => {
    window.addEventListener("keydown", handleKeydown);
    void listen(MAIN_PANEL_DISMISS_EVENT, () => options.beforeDismiss?.())
      .then((unlisten) => {
        if (disposed) {
          unlisten();
          return;
        }
        unlistenDismiss = unlisten;
      })
      .catch(reportDismissalError);
    void listen(MAIN_PANEL_BLUR_EVENT, () => dismissPanel())
      .then((unlisten) => {
        if (disposed) {
          unlisten();
          return;
        }

        unlistenBlur = unlisten;
      })
      .catch(reportDismissalError);
  });

  onUnmounted(() => {
    disposed = true;
    window.removeEventListener("keydown", handleKeydown);
    unlistenBlur?.();
    unlistenDismiss?.();
  });
}
