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

export async function runTextFormatterKeyboardChecks() {
  const previous = window.__TAURI_INTERNALS__;
  window.__TAURI_INTERNALS__ = {
    metadata: { currentWindow: { label: "text-formatter-browser-test" } },
    transformCallback: () => 1,
    invoke: async (command) => {
      if (command === "get_text_formatter_input") {
        return { format: "json", content: '{"name":"PTools"}', updatedAt: 1 };
      }
      if (command === "get_text_formatter_pinned") return false;
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
    const toolbar = host.querySelector(".formatter-toolbar");
    assert(toolbar.scrollWidth <= toolbar.clientWidth, "formatter toolbar overflows at the minimum window width");
    assert(toolbar.querySelector(".toolbar-switch").textContent.includes("Ctrl+B"), "wrapping shortcut is not shown");
    const sendWrappingShortcut = () => host.dispatchEvent(new KeyboardEvent("keydown", {
      key: "b", code: "KeyB", ctrlKey: true, bubbles: true, cancelable: true,
    }));
    assert(!sendWrappingShortcut(), "formatter did not prevent the Ctrl+B browser action");
    await nextTick();
    assert(!host.querySelector(".cm-lineWrapping"), "Ctrl+B did not switch formatter wrapping off");
    sendWrappingShortcut();
    await nextTick();
    assert(host.querySelector(".cm-lineWrapping"), "Ctrl+B did not switch formatter wrapping on");
    const editor = host.querySelector(".cm-content");
    const modifier = navigator.platform.includes("Mac") ? { metaKey: true } : { ctrlKey: true };
    editor.dispatchEvent(new KeyboardEvent("keydown", {
      key: "f", code: "KeyF", ...modifier, bubbles: true, cancelable: true,
    }));
    await waitFor(() => host.querySelector(".cm-panels-top .cm-search"), "search panel did not open at the top");
    assert(!host.querySelector(".cm-panels-bottom .cm-search"), "search panel still opens at the bottom");
    return { passed: true, shortcut: "Ctrl+B", searchPanel: "top" };
  } finally {
    app.unmount();
    host.remove();
    await nextTick();
    window.__TAURI_INTERNALS__ = previous;
  }
}
