const MODIFIER_KEYS = ["Alt", "Control", "Meta", "Shift"];

export function isKeyboardModifierKey(event: KeyboardEvent) {
  return MODIFIER_KEYS.includes(event.key);
}

export function keyboardShortcutFromEvent(event: KeyboardEvent) {
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

export function matchesKeyboardShortcut(event: KeyboardEvent, shortcut: string) {
  const eventShortcut = keyboardShortcutFromEvent(event);
  return eventShortcut?.toLowerCase() === shortcut.trim().toLowerCase();
}

function keyboardShortcutKey(event: KeyboardEvent) {
  if (event.code.startsWith("Key")) {
    return event.code.slice(3);
  }
  if (event.code.startsWith("Digit")) {
    return event.code.slice(5);
  }
  return event.code === "Space" ? "Space" : event.code;
}
