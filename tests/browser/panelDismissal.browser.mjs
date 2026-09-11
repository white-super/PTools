import { createApp, h, nextTick } from "vue";
import { usePanelDismissal } from "../../src/layouts/composables/usePanelDismissal";

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

export async function runPanelDismissalChecks() {
  const calls = [];
  const callbacks = new Map();
  const previous = window.__TAURI_INTERNALS__;
  let nextCallbackId = 1;
  let cleanupCount = 0;
  window.__TAURI_INTERNALS__ = {
    transformCallback: (callback) => {
      const callbackId = nextCallbackId++;
      callbacks.set(callbackId, callback);
      return callbackId;
    },
    invoke: async (command, args) => {
      calls.push({ command, args });
      if (command === "plugin:event|listen") return args.handler;
      if (command === "plugin:event|unlisten" || command === "hide_main_panel") return;
      throw new Error(`Unexpected native call: ${command}`);
    },
  };
  const host = document.createElement("div");
  document.body.append(host);
  const app = createApp({
    setup() {
      usePanelDismissal({ beforeDismiss: () => cleanupCount++ });
      return () => h("main", "panel dismissal test");
    },
  });
  try {
    app.mount(host);
    await waitFor(() => calls.filter((call) => call.command === "plugin:event|listen").length === 2, "panel listeners did not register");
    const dismissListener = calls.find((call) => call.args?.event === "ptools://main-panel-dismiss");
    callbacks.get(dismissListener.args.handler)({ payload: true });
    await nextTick();
    assert(cleanupCount === 1, "native panel dismissal did not clear transient state");
    return { passed: true, cleanupCount };
  } finally {
    app.unmount();
    host.remove();
    await nextTick();
    window.__TAURI_INTERNALS__ = previous;
  }
}
