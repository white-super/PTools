import { MAX_QUICK_TOOLS } from "./quickToolOrder";
import {
  matchesKeyboardShortcut,
  type KeyboardShortcutEvent,
} from "../../utils/keyboardShortcut";

interface QuickToolKeyboardEvent extends KeyboardShortcutEvent {
  readonly isComposing: boolean;
  readonly repeat: boolean;
}

export const DEFAULT_QUICK_TOOL_SHORTCUTS: readonly string[] = [
  "Command+1", "Command+2", "Command+3", "Command+4", "Command+5",
];

export function quickToolIndexFromKeyboard(
  event: QuickToolKeyboardEvent,
  shortcuts: readonly string[] = DEFAULT_QUICK_TOOL_SHORTCUTS,
) {
  if (event.isComposing || event.repeat) return undefined;
  const index = shortcuts
    .slice(0, MAX_QUICK_TOOLS)
    .findIndex((shortcut) => matchesKeyboardShortcut(event, shortcut));
  return index < 0 ? undefined : index;
}
