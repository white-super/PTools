import assert from "node:assert/strict";
import test from "node:test";
import { sourceUrl } from "./loadTypeScript.mjs";

const order = await import(await sourceUrl(
  new URL("../src/layouts/features/quick-tools/quickToolOrder.ts", import.meta.url),
));
const shortcuts = await import(await sourceUrl(
  new URL("../src/layouts/features/quick-tools/quickToolShortcut.ts", import.meta.url),
));

test("quick tools can be added, moved and removed without mutating the source", () => {
  const original = ["text-diff", "json", "url"];
  assert.deepEqual(order.placeQuickTool(original, "base64", 3), ["text-diff", "json", "url", "base64"]);
  assert.deepEqual(order.placeQuickTool(original, "text-diff", 3), ["json", "url", "text-diff"]);
  assert.deepEqual(order.removeQuickTool(original, "json"), ["text-diff", "url"]);
  assert.deepEqual(original, ["text-diff", "json", "url"]);
});

test("quick tool order rejects overflow, duplicates and unknown tools", () => {
  assert.throws(
    () => order.placeQuickTool(["text-diff", "json", "xml", "html", "url"], "base64", 5),
    /最多只能添加 5 个/,
  );
  assert.throws(() => order.validateQuickToolOrder(["json", "json"]), /不能重复添加/);
  assert.throws(() => order.validateQuickToolOrder(["unknown"]), /未知快捷工具/);
});

test("Command+1 through Command+5 map to current positions only", () => {
  const event = {
    key: "3",
    code: "Digit3",
    ctrlKey: false,
    metaKey: true,
    altKey: false,
    shiftKey: false,
    isComposing: false,
    repeat: false,
  };
  assert.equal(shortcuts.quickToolIndexFromKeyboard(event), 2);
  assert.equal(
    shortcuts.quickToolIndexFromKeyboard({ ...event, metaKey: false, ctrlKey: true }),
    undefined,
  );
  for (const override of [
    { metaKey: false },
    { ctrlKey: true },
    { altKey: true },
    { shiftKey: true },
    { isComposing: true },
    { repeat: true },
    { key: "6", code: "Digit6" },
  ]) {
    assert.equal(shortcuts.quickToolIndexFromKeyboard({ ...event, ...override }), undefined);
  }
});

test("quick tool shortcuts support custom keys and Alt/Option aliases", () => {
  const event = {
    key: "q",
    code: "KeyQ",
    ctrlKey: false,
    metaKey: false,
    altKey: true,
    shiftKey: true,
    isComposing: false,
    repeat: false,
  };
  assert.equal(
    shortcuts.quickToolIndexFromKeyboard(event, ["Ctrl+1", "Option+Shift+Q"]),
    1,
  );
  assert.equal(
    shortcuts.quickToolIndexFromKeyboard(event, ["Ctrl+1", "Alt+Q"]),
    undefined,
  );
});
