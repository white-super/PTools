import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { onMounted, onUnmounted } from "vue";

const ESCAPE_KEY = "Escape";
const MAIN_PANEL_BLUR_EVENT = "tauri://blur";

function reportDismissalError(error: unknown) {
  console.error("Failed to hide the main panel", error);
}

export function usePanelDismissal() {
  let disposed = false;
  let unlistenBlur: UnlistenFn | undefined;

  function dismissPanel() {
    return invoke<void>("hide_main_panel");
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key !== ESCAPE_KEY) {
      return;
    }

    event.preventDefault();
    void dismissPanel().catch(reportDismissalError);
  }

  onMounted(() => {
    window.addEventListener("keydown", handleKeydown);
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
  });
}
