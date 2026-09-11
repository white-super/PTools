import { createApp, h, nextTick } from "vue";
import ClipboardToolbarActions from "../../src/layouts/components/ClipboardToolbarActions.vue";
import { useTextDiffLauncher } from "../../src/layouts/composables/useTextDiffLauncher";

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

export async function runTextDiffEntryChecks() {
  const calls = [];
  const errors = [];
  const previous = window.__TAURI_INTERNALS__;
  window.__TAURI_INTERNALS__ = {
    transformCallback: () => 1,
    invoke: async (command, args) => {
      calls.push({ command, args });
      if (command === "show_text_diff") return "text-diff-browser-test";
      throw new Error(`Unexpected native call: ${command}`);
    },
  };
  const host = document.createElement("div");
  host.className = "paste-flow-panel theme-classic";
  document.body.append(host);
  const app = createApp({
    setup() {
      const { openEmptyTextDiff } = useTextDiffLauncher((error) => errors.push(error));
      return () => h(ClipboardToolbarActions, {
        modelValue: false,
        "onUpdate:modelValue": () => {},
        onTextDiff: openEmptyTextDiff,
      });
    },
  });
  try {
    app.mount(host);
    const trigger = host.querySelector('[aria-label="打开空白文本对比窗口"]');
    assert(trigger?.textContent.includes("文本对比"), "text comparison entry is missing");
    trigger.click();
    await waitFor(() => calls.length === 1, "text comparison entry did not launch a window");
    const request = calls[0];
    assert(request.command === "show_text_diff", "entry invoked the wrong native command");
    assert(request.args.pinned === true, "empty comparison window is not pinned by default");
    assert(request.args.input.left.content === "", "left comparison content is not empty");
    assert(request.args.input.right.content === "", "right comparison content is not empty");
    assert(errors.length === 0, "entry reported an unexpected error");
    return { passed: true, pinned: request.args.pinned };
  } finally {
    app.unmount();
    host.remove();
    await nextTick();
    window.__TAURI_INTERNALS__ = previous;
  }
}
