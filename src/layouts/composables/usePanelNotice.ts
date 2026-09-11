import { onUnmounted, readonly, shallowRef } from "vue";

const NOTICE_DURATION_MS = 2800;

export interface PanelNoticeState {
  readonly message: string;
  readonly cardId?: number;
}

export function usePanelNotice() {
  const notice = shallowRef<PanelNoticeState>();
  let dismissTimer: number | undefined;

  function clearNotice() {
    if (dismissTimer !== undefined) window.clearTimeout(dismissTimer);
    dismissTimer = undefined;
    notice.value = undefined;
  }

  function showNotice(nextNotice: PanelNoticeState) {
    clearNotice();
    notice.value = nextNotice;
    dismissTimer = window.setTimeout(clearNotice, NOTICE_DURATION_MS);
  }

  function reportError(action: string, error: unknown, cardId?: number) {
    const message = error instanceof Error ? error.message : String(error);
    console.error(`Failed to ${action}`, error);
    showNotice({ message, cardId });
  }

  onUnmounted(clearNotice);
  return { notice: readonly(notice), reportError, clearNotice };
}
