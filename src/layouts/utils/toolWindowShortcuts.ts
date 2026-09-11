export const LINE_WRAPPING_SHORTCUT_LABEL = "Ctrl+B";

export function isLineWrappingShortcut(event: KeyboardEvent) {
  return event.code === "KeyB"
    && event.ctrlKey
    && !event.metaKey
    && !event.altKey
    && !event.shiftKey
    && !event.isComposing
    && !event.repeat;
}
