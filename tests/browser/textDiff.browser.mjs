import { createApp, h, nextTick, shallowRef } from "vue";
import { checkLiveEditing, checkInvalidEditing } from "./diffEditing.browser.mjs";
import TextDiffWorkspace from "../../src/layouts/features/text-diff/TextDiffWorkspace.vue";
import { usePasteFlowKeyboard } from "../../src/layouts/composables/usePasteFlowKeyboard";
import { useDiffComputation } from "../../src/layouts/features/text-diff/useDiffComputation";

// Browser-only component checks. Only native IPC is mocked; editors and workers are real.
function assert(value, message) {
  if (!value) throw new Error(message);
}
async function waitFor(predicate, message) {
  const deadline = performance.now() + 10000;
  while (!predicate()) {
    if (performance.now() > deadline) throw new Error(message);
    await new Promise((resolve) => setTimeout(resolve, 25));
  }
}

export async function runTextDiffBrowserChecks() {
  const calls = [];
  const previous = window.__TAURI_INTERNALS__;
  const originalInput = {
    left: { name: "left.json", content: '{"name":"PTools","items":[1,2]}' },
    right: {
      name: "right.json",
      content: '{\n  "name": "PTools",\n  "items": [1, 2]\n}',
    },
  };
  window.__TAURI_INTERNALS__ = {
    metadata: { currentWindow: { label: "text-diff-browser-test" } },
    transformCallback: () => 1,
    invoke: async (command, args) => {
      calls.push({ command, args });
      if (command === "get_text_diff_input") return structuredClone(originalInput);
      if (command === "get_text_diff_pinned") return false;
      if (command === "plugin:event|listen") return 1;
      if (["plugin:event|unlisten", "set_text_diff_pinned", "close_text_diff", "plugin:clipboard|write_text"].includes(command)) return;
      throw new Error(`Unexpected native call: ${command}`);
    },
  };
  const host = document.createElement("div");
  host.style.cssText = "position:fixed;inset:0;width:860px;height:600px;z-index:999;background:var(--app-menu-background)";
  document.body.append(host);
  const app = createApp(TextDiffWorkspace);
  const previousTheme = document.documentElement.className;
  try {
    app.mount(host);
    await waitFor(() => host.querySelectorAll(".cm-editor").length === 2, "diff editors did not mount");
    assert(host.querySelector(".cm-changedText"), "raw JSON whitespace differences missing");
    const toolbar = host.querySelector(".diff-toolbar");
    assert(toolbar.scrollWidth <= toolbar.clientWidth, "toolbar overflows at minimum width");
    const row = [...toolbar.children].map((element) => element.getBoundingClientRect());
    assert(
      row.every((rect) => Math.abs(rect.y + rect.height / 2 - row[0].y - row[0].height / 2) < 2),
      "toolbar wraps",
    );
    host.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", code: "Escape", bubbles: true }));
    await waitFor(() => calls.some((call) => call.command === "close_text_diff"), "Escape did not close an unpinned diff window");

    toolbar.querySelector('input[type="checkbox"]').click();
    await waitFor(() => host.querySelector("footer").textContent.includes("格式化后相同"), "formatted JSON should be equal");
    toolbar.querySelector("details summary").click();
    const more = toolbar.querySelector(".more-menu");
    more.querySelector("input").click();
    await nextTick();
    assert(!host.querySelector(".cm-lineWrapping"), "wrapping did not switch off on first click");
    more.querySelector("input").click();
    await nextTick();
    assert(host.querySelectorAll(".cm-lineWrapping").length === 2, "wrapping did not switch on");
    assert(more.textContent.includes("Ctrl+B"), "wrapping shortcut is not shown");
    host.dispatchEvent(new KeyboardEvent("keydown", { key: "b", code: "KeyB", ctrlKey: true, bubbles: true }));
    await nextTick();
    assert(!host.querySelector(".cm-lineWrapping"), "Ctrl+B did not switch wrapping off");
    host.dispatchEvent(new KeyboardEvent("keydown", { key: "b", code: "KeyB", ctrlKey: true, bubbles: true }));
    await nextTick();
    assert(host.querySelectorAll(".cm-lineWrapping").length === 2, "Ctrl+B did not switch wrapping on");

    const changed = await checkLiveEditing(host, calls);
    await waitFor(() => host.querySelector(".cm-changedText"), "edited draft changes missing");
    host.dispatchEvent(new KeyboardEvent("keydown", { key: "e", ctrlKey: true, bubbles: true }));
    await nextTick();
    assert(host.querySelector("footer").textContent.includes("当前 1/"), "next difference shortcut failed");
    host.dispatchEvent(new KeyboardEvent("keydown", { key: "d", metaKey: true, bubbles: true }));
    await nextTick();
    assert(
      calls.some((call) => call.command === "set_text_diff_pinned" && call.args.pinned),
      "pin shortcut failed",
    );
    const closeCallsWhilePinned = calls.filter((call) => call.command === "close_text_diff").length;
    host.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", code: "Escape", bubbles: true }));
    await nextTick();
    assert(
      calls.filter((call) => call.command === "close_text_diff").length === closeCallsWhilePinned,
      "Escape closed a pinned diff window",
    );
    host.querySelector('[aria-label="右侧复制内容"]').click();
    await waitFor(() => calls.some((call) => call.command === "plugin:clipboard|write_text"), "copy failed");
    assert(
      calls.findLast((call) => call.command === "plugin:clipboard|write_text").args.text === JSON.stringify(JSON.parse(changed), null, 2),
      "copy did not use edited content",
    );

    const leftFile = host.querySelectorAll('input[type="file"]')[0];
    const data = new DataTransfer();
    data.items.add(
      new File(['\ufeff{"name":"imported"}'], "imported.json", {
        type: "text/plain",
      }),
    );
    leftFile.files = data.files;
    leftFile.dispatchEvent(new Event("change", { bubbles: true }));
    await waitFor(() => host.querySelector(".source-name").textContent === "imported.json", "file import failed");
    await waitFor(() => host.querySelector(".cm-changedText"), "imported file diff missing");
    const themes = [];
    for (const theme of ["classic", "soft-glow", "dark"]) {
      document.documentElement.className = `theme-${theme}`;
      const line = host.querySelector(".cm-changedLine");
      themes.push({
        theme,
        background: getComputedStyle(line).backgroundColor,
      });
    }
    assert(new Set(themes.map((theme) => theme.background)).size > 1, "diff colors do not follow theme");
    await checkInvalidEditing(host, waitFor);
    const summary = {
      toolbarWidth: toolbar.clientWidth,
      themes,
      calls: calls.map((call) => call.command),
      passed: true,
    };
    return summary;
  } finally {
    app.unmount();
    host.remove();
    await nextTick();
    window.__TAURI_INTERNALS__ = previous;
    document.documentElement.className = previousTheme;
  }
}

export async function runDiffKeyboardChecks() {
  const previous = window.__TAURI_INTERNALS__;
  window.__TAURI_INTERNALS__ = {
    transformCallback: () => 1,
    invoke: async () => 1,
  };
  const seen = [];
  const cards = shallowRef([
    { id: 1, content: "one" },
    { id: 2, content: "two" },
  ]);
  let keyboard;
  const host = document.createElement("div");
  document.body.append(host);
  const app = createApp({
    setup() {
      keyboard = usePasteFlowKeyboard({
        cards,
        getCardContainer: () => null,
        scrollToCard: () => {},
        cardNavigation: {
          shortcutSettings: shallowRef({ nextCardShortcut: "Ctrl+D" }),
        },
        filterNavigation: {
          activeFilter: shallowRef("all"),
          filters: shallowRef(["all"]),
          shortcutSettings: shallowRef(),
        },
        closeMenus: () => {},
        focusSearch: () => {},
        formatCard: () => {},
        pasteCard: () => {},
        diffCard: (card) => seen.push(card.id),
        reportError: (error) => {
          throw error;
        },
      });
      return () => h("input", { "aria-label": "keyboard-test-input" });
    },
  });
  app.mount(host);
  await nextTick();
  keyboard.selectCard(1);
  const send = (target, options = {}) =>
    target.dispatchEvent(
      new KeyboardEvent("keydown", {
        key: "d",
        code: "KeyD",
        bubbles: true,
        ...options,
      }),
    );
  send(host);
  send(host.querySelector("input"));
  send(host, { isComposing: true });
  send(host, { repeat: true });
  send(host, { ctrlKey: true });
  assert(keyboard.selectedCardId.value === 2, "Ctrl+D card navigation regressed");
  send(host);
  assert(JSON.stringify(seen) === "[1,2]", "D intercepts input, modifiers, repeat, or IME");
  app.unmount();
  host.remove();
  await nextTick();
  window.__TAURI_INTERNALS__ = previous;
  return { seen, passed: true };
}

export async function runDiffWorkerChecks() {
  let state;
  const host = document.createElement("div");
  const app = createApp({
    setup() {
      state = useDiffComputation();
      return () => null;
    },
  });
  app.mount(host);
  const cases = [
    {
      left: '<root xml:space="preserve">before <b>bold</b> after</root>',
      right: '<root xml:space="preserve">before <b>bold</b> after</root>',
      format: "xml",
      formatted: true,
    },
    {
      left: "<div><pre> a\n  b </pre></div>",
      right: "<div><pre> a\n  b </pre></div>",
      format: "html",
      formatted: true,
    },
  ];
  for (const input of cases) {
    state.compute(input);
    await waitFor(() => !state.busy.value, `${input.format} worker did not finish`);
    assert(!state.error.value, state.error.value);
    assert(state.result.value.changes.length === 0, `${input.format} worker output differs`);
  }
  const lines = Array.from({ length: 12000 }, (_, index) => `${index} ${Math.imul(index + 1, 2654435761) >>> 0}`);
  let ticks = 0;
  const timer = setInterval(() => ticks++, 16);
  state.compute({
    left: lines.join("\n"),
    right: [...lines].reverse().join("\n"),
    format: "plain",
    formatted: false,
  });
  await new Promise((resolve) => setTimeout(resolve, 260));
  state.cancel();
  clearInterval(timer);
  assert(ticks > 4, "background comparison blocked the UI thread");
  assert(!state.busy.value && state.error.value.includes("已取消"), "worker cancellation failed");
  state.compute({
    left: "old",
    right: "discarded",
    format: "plain",
    formatted: false,
  });
  state.compute({
    left: "latest",
    right: "latest",
    format: "plain",
    formatted: false,
  });
  await waitFor(() => !state.busy.value, "latest request did not finish");
  assert(state.result.value.left === "latest" && state.result.value.changes.length === 0, "stale result replaced current request");
  app.unmount();
  return {
    passed: true,
    responsiveTicks: ticks,
    formats: ["xml", "html"],
    cancellation: true,
    latestRequestWins: true,
  };
}
