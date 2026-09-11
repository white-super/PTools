import assert from "node:assert/strict";
import test from "node:test";
import { sourceUrl } from "./loadTypeScript.mjs";

const shortcuts = await import(await sourceUrl(
  new URL("../src/layouts/utils/toolWindowShortcuts.ts", import.meta.url),
));

const baseEvent = {
  code: "KeyB",
  ctrlKey: true,
  metaKey: false,
  altKey: false,
  shiftKey: false,
  isComposing: false,
  repeat: false,
};

test("Ctrl+B is the shared line-wrapping shortcut", () => {
  assert.equal(shortcuts.LINE_WRAPPING_SHORTCUT_LABEL, "Ctrl+B");
  assert.equal(shortcuts.isLineWrappingShortcut(baseEvent), true);
});

test("line-wrapping ignores modified, repeated, composing and unrelated keys", () => {
  for (const override of [
    { code: "KeyL" },
    { ctrlKey: false },
    { metaKey: true },
    { altKey: true },
    { shiftKey: true },
    { isComposing: true },
    { repeat: true },
  ]) {
    assert.equal(shortcuts.isLineWrappingShortcut({ ...baseEvent, ...override }), false);
  }
});
