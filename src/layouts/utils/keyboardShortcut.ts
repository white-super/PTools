const MODIFIER_KEYS = ["Alt", "Control", "Meta", "Shift"];

export interface KeyboardShortcutEvent {
  readonly altKey: boolean;
  readonly code: string;
  readonly ctrlKey: boolean;
  readonly key: string;
  readonly metaKey: boolean;
  readonly shiftKey: boolean;
}

export function isKeyboardModifierKey(event: KeyboardShortcutEvent) {
  return MODIFIER_KEYS.includes(event.key);
}

export function keyboardShortcutFromEvent(event: KeyboardShortcutEvent) {
  if (isKeyboardModifierKey(event)) {
    return undefined;
  }
  const modifiers = [
    event.ctrlKey && "Ctrl",
    event.altKey && "Option",
    event.shiftKey && "Shift",
    event.metaKey && "Command",
  ].filter(Boolean);
  if (modifiers.length === 0) {
    return undefined;
  }
  return [...modifiers, keyboardShortcutKey(event)].join("+");
}

export function matchesKeyboardShortcut(event: KeyboardShortcutEvent, shortcut: string) {
  const eventShortcut = keyboardShortcutFromEvent(event);
  return eventShortcut !== undefined
    && normalizeKeyboardShortcut(eventShortcut) === normalizeKeyboardShortcut(shortcut);
}

function keyboardShortcutKey(event: KeyboardShortcutEvent) {
  if (event.code.startsWith("Key")) {
    return event.code.slice(3);
  }
  if (event.code.startsWith("Digit")) {
    return event.code.slice(5);
  }
  if (event.code === "Space") {
    return "Space";
  }
  if (/^[0-9]$/.test(event.key)) {
    return event.key;
  }
  return event.code;
}

function normalizeKeyboardShortcut(shortcut: string) {
  return shortcut
    .split("+")
    .map((part) => part.trim().toLowerCase() === "option" ? "alt" : part.trim().toLowerCase())
    .join("+");
}
