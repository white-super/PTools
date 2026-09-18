import { createApp, nextTick } from "vue";
import TextFormatterWorkspace from "../../src/layouts/features/text-formatter/TextFormatterWorkspace.vue";

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

function dispatchKey(target, init) {
  return target.dispatchEvent(new KeyboardEvent("keydown", {
    bubbles: true,
    cancelable: true,
    ...init,
  }));
}

function assertLocalizedSearchPanel(searchPanel) {
  assert(searchPanel.querySelector('input[name="search"]').placeholder === "查找", "search input is not localized");
  assert(searchPanel.querySelector('input[name="replace"]').placeholder === "替换", "replace input is not localized");
  assert(searchPanel.querySelector('button[name="next"]').textContent === "下一个", "next action is not localized");
  assert(searchPanel.querySelector('button[name="prev"]').textContent === "上一个", "previous action is not localized");
  assert(searchPanel.querySelector('button[name="replaceAll"]').textContent === "全部替换", "replace-all action is not localized");
  assert(searchPanel.textContent.includes("区分大小写"), "search options are not localized");
  assert(searchPanel.querySelector('button[name="close"]').ariaLabel === "关闭", "close action is not localized");
}

async function checkLineWrappingShortcut(host) {
  const toolbar = host.querySelector(".formatter-toolbar");
  assert(toolbar.scrollWidth <= toolbar.clientWidth, "formatter toolbar overflows at the minimum window width");
  assert(toolbar.querySelector(".toolbar-switch").textContent.includes("Ctrl+B"), "wrapping shortcut is not shown");
  const sendShortcut = () => dispatchKey(host, { key: "b", code: "KeyB", ctrlKey: true });
  assert(!sendShortcut(), "formatter did not prevent the Ctrl+B browser action");
  await nextTick();
  assert(!host.querySelector(".cm-lineWrapping"), "Ctrl+B did not switch formatter wrapping off");
  sendShortcut();
  await nextTick();
  assert(host.querySelector(".cm-lineWrapping"), "Ctrl+B did not switch formatter wrapping on");
}

async function checkSearchPanel(host, getCloseCalls) {
  const editor = host.querySelector(".cm-content");
  const modifier = navigator.platform.includes("Mac") ? { metaKey: true } : { ctrlKey: true };
  dispatchKey(editor, { key: "f", code: "KeyF", ...modifier });
  await waitFor(() => host.querySelector(".cm-panels-top .cm-search"), "search panel did not open at the top");
  assert(!host.querySelector(".cm-panels-bottom .cm-search"), "search panel still opens at the bottom");
  const searchPanel = host.querySelector(".cm-search");
  assertLocalizedSearchPanel(searchPanel);
  const searchInput = searchPanel.querySelector('input[name="search"]');
  await waitFor(() => document.activeElement === searchInput, "search input did not receive focus");
  dispatchKey(searchInput, { key: "Escape", code: "Escape" });
  await waitFor(() => !host.querySelector(".cm-search"), "first Escape did not close the search panel");
  assert(getCloseCalls() === 0, "first Escape also closed the formatter window");
  assert(document.activeElement === editor, "focus did not return to the editor after closing search");
  dispatchKey(editor, { key: "Escape", code: "Escape" });
  await waitFor(() => getCloseCalls() === 1, "second Escape did not close the formatter window");
}

export async function runTextFormatterKeyboardChecks() {
  const previous = window.__TAURI_INTERNALS__;
  let closeCalls = 0;
  window.__TAURI_INTERNALS__ = {
    metadata: { currentWindow: { label: "text-formatter-browser-test" } },
    transformCallback: () => 1,
    invoke: async (command) => {
      if (command === "get_text_formatter_input") {
        return { format: "json", content: '{"name":"PTools"}', updatedAt: 1 };
      }
      if (command === "get_text_formatter_pinned") return false;
      if (command === "close_text_formatter") {
        closeCalls += 1;
        return undefined;
      }
      throw new Error(`Unexpected native call: ${command}`);
    },
  };
  const host = document.createElement("div");
  host.style.cssText = "position:fixed;inset:0;width:760px;height:600px;z-index:999";
  document.body.append(host);
  const app = createApp(TextFormatterWorkspace, { windowId: "text-formatter-browser-test" });
  try {
    app.mount(host);
    await waitFor(() => host.querySelector(".cm-lineWrapping"), "formatter wrapping did not initialize");
    await checkLineWrappingShortcut(host);
    await checkSearchPanel(host, () => closeCalls);
    return { passed: true, shortcut: "Ctrl+B", searchPanel: "top", layeredEscape: true };
  } finally {
    app.unmount();
    host.remove();
    await nextTick();
    window.__TAURI_INTERNALS__ = previous;
  }
}

export async function runEmptyTextFormatterChecks() {
  const previous = window.__TAURI_INTERNALS__;
  const initialErrorCount = document.querySelectorAll(".el-message--error").length;
  window.__TAURI_INTERNALS__ = {
    metadata: { currentWindow: { label: "empty-text-formatter-browser-test" } },
    transformCallback: () => 1,
    invoke: async (command) => {
      if (command === "get_text_formatter_input") {
        return { format: "json", content: "", updatedAt: 0 };
      }
      if (command === "get_text_formatter_pinned") return false;
      throw new Error(`Unexpected native call: ${command}`);
    },
  };
  const host = document.createElement("div");
  host.style.cssText = "position:fixed;inset:0;width:760px;height:600px;z-index:999";
  document.body.append(host);
  const app = createApp(TextFormatterWorkspace, { windowId: "empty-text-formatter-browser-test" });
  try {
    app.mount(host);
    await waitFor(() => host.querySelector(".cm-content"), "empty formatter editor did not initialize");
    assert(host.querySelector(".cm-content").textContent === "", "empty formatter did not stay empty");
    assert(
      document.querySelectorAll(".el-message--error").length === initialErrorCount,
      "empty formatter input raised an initialization error",
    );
    return { passed: true, content: "" };
  } finally {
    app.unmount();
    host.remove();
    await nextTick();
    window.__TAURI_INTERNALS__ = previous;
  }
}
