import type { TextFormat, TextTransformAction } from "./types";

const ENCODE_KEY = "E";
const DECODE_KEY = "D";

const ACTION_KEYS: Partial<Record<TextTransformAction, string>> = {
  format: "R",
  "remove-comments": "Q",
  "minify-copy": "W",
  "escape-copy": "A",
  "to-xml-copy": "S",
  "to-typescript-copy": "T",
  "url-encode": ENCODE_KEY,
  "url-decode": DECODE_KEY,
  "base64-encode": ENCODE_KEY,
  "base64-decode": DECODE_KEY,
  "date-to-timestamp": ENCODE_KEY,
  "timestamp-to-date": DECODE_KEY,
};

const FORMAT_ACTIONS: Partial<Record<TextFormat, Readonly<Record<string, TextTransformAction>>>> = {
  json: {
    KeyR: "format",
    KeyQ: "remove-comments",
    KeyW: "minify-copy",
    KeyA: "escape-copy",
    KeyS: "to-xml-copy",
    KeyT: "to-typescript-copy",
  },
  xml: { KeyR: "format" },
  html: { KeyR: "format" },
  url: { KeyE: "url-encode", KeyD: "url-decode" },
  base64: { KeyE: "base64-encode", KeyD: "base64-decode" },
  date: {
    KeyR: "format",
    KeyE: "date-to-timestamp",
    KeyD: "timestamp-to-date",
  },
};

export function formatterActionFromKeyboard(event: KeyboardEvent, format?: TextFormat) {
  if (!format || !isControlShortcut(event)) {
    return undefined;
  }
  return FORMAT_ACTIONS[format]?.[event.code];
}

export function formatterShortcutLabel(action: TextTransformAction) {
  const key = ACTION_KEYS[action];
  if (!key) {
    return undefined;
  }
  return `Ctrl+${key}`;
}

function isControlShortcut(event: KeyboardEvent) {
  return event.ctrlKey && !event.metaKey && !event.altKey && !event.shiftKey;
}
