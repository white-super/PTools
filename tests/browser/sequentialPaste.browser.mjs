import { createApp, nextTick } from "vue";
import SequentialPasteWindow from "../../src/layouts/features/sequential-paste/SequentialPasteWindow.vue";

function assert(value, message) {
  if (!value) throw new Error(message);
}

function required(root, selector, message) {
  const element = root.querySelector(selector);
  if (!element) throw new Error(message);
  return element;
}

async function waitFor(predicate, message) {
  const deadline = performance.now() + 10000;
  while (!predicate()) {
    if (performance.now() > deadline) throw new Error(message);
    await new Promise((resolve) => setTimeout(resolve, 25));
  }
}

function sampleItems() {
  return [
    { id: 1, format: "text", content: "客户名称：星河科技", filePaths: [] },
    { id: 2, format: "file", content: "/tmp/报价单.pdf", filePaths: ["/tmp/报价单.pdf"] },
    { id: 3, format: "text", content: "上海市浦东新区世纪大道 100 号", filePaths: [] },
  ];
}

function createSnapshot(mode = "capture") {
  return { enabled: true, mode, direction: "forward", items: sampleItems() };
}

async function handleMockCommand(context, command, args) {
  context.calls.push({ command, args });
  if (command === "plugin:event|listen") return args.handler;
  if (command === "plugin:event|unlisten") return;
  if (command === "get_sequential_paste_state") return structuredClone(context.snapshot);
  if (command === "get_app_settings") {
    return { theme: context.theme, sequentialPasteShortcut: "Ctrl+Shift+V" };
  }
  if (command === "set_sequential_paste_mode") {
    context.snapshot = { ...context.snapshot, mode: args.mode };
  } else if (command === "set_sequential_paste_direction") {
    context.snapshot = { ...context.snapshot, direction: args.direction };
  } else if (command === "reorder_sequential_paste") {
    const byId = new Map(context.snapshot.items.map((item) => [item.id, item]));
    context.snapshot = { ...context.snapshot, items: args.itemIds.map((id) => byId.get(id)) };
  } else if (command === "remove_sequential_paste_item") {
    context.snapshot = {
      ...context.snapshot,
      items: context.snapshot.items.filter((item) => item.id !== args.itemId),
    };
  } else if (command === "clear_sequential_paste") {
    context.snapshot = { ...context.snapshot, items: [] };
  } else if (command === "close_sequential_paste") {
    context.snapshot = { ...context.snapshot, enabled: false, mode: "capture", items: [] };
    return;
  } else {
    throw new Error(`Unexpected native call: ${command}`);
  }
  return structuredClone(context.snapshot);
}

function installMock(theme = "classic", mode = "capture") {
  const previous = window.__TAURI_INTERNALS__;
  const previousTheme = document.documentElement.className;
  const callbacks = new Map();
  const calls = [];
  let nextCallbackId = 1;
  const context = { calls, snapshot: createSnapshot(mode), theme };
  window.__TAURI_INTERNALS__ = {
    metadata: { currentWindow: { label: "sequential-paste" } },
    transformCallback: (callback) => {
      const id = nextCallbackId++;
      callbacks.set(id, callback);
      return id;
    },
    invoke: (command, args) => handleMockCommand(context, command, args),
  };
  document.documentElement.className = `theme-${theme}`;
  return {
    callbacks,
    calls,
    context,
    restore() {
      window.__TAURI_INTERNALS__ = previous;
      document.documentElement.className = previousTheme;
    },
  };
}

function mountWindow(style) {
  const host = document.createElement("div");
  host.style.cssText = style;
  document.body.append(host);
  const app = createApp(SequentialPasteWindow);
  app.mount(host);
  return { app, host };
}

function dragItemAfter(host, sourceId, targetId) {
  const source = required(host, `[data-item-id="${sourceId}"]`, "drag source is missing");
  const target = required(host, `[data-item-id="${targetId}"]`, "drop target is missing");
  const transfer = new DataTransfer();
  source.dispatchEvent(new DragEvent("dragstart", { bubbles: true, dataTransfer: transfer }));
  target.dispatchEvent(new DragEvent("dragover", {
    bubbles: true,
    cancelable: true,
    clientY: target.getBoundingClientRect().bottom,
    dataTransfer: transfer,
  }));
  target.dispatchEvent(new DragEvent("drop", {
    bubbles: true,
    cancelable: true,
    clientY: target.getBoundingClientRect().bottom,
    dataTransfer: transfer,
  }));
  source.dispatchEvent(new DragEvent("dragend", { bubbles: true, dataTransfer: transfer }));
}

async function dragItemWithEarlyDragEnd(host, sourceId, targetId) {
  const source = required(host, `[data-item-id="${sourceId}"]`, "drag source is missing");
  const target = required(host, `[data-item-id="${targetId}"]`, "drop target is missing");
  const transfer = new DataTransfer();
  source.dispatchEvent(new DragEvent("dragstart", { bubbles: true, dataTransfer: transfer }));
  target.dispatchEvent(new DragEvent("dragover", {
    bubbles: true,
    cancelable: true,
    clientY: target.getBoundingClientRect().bottom,
    dataTransfer: transfer,
  }));
  source.dispatchEvent(new DragEvent("dragend", { bubbles: true, dataTransfer: transfer }));
  await new Promise((resolve) => setTimeout(resolve, 0));
  target.dispatchEvent(new DragEvent("drop", {
    bubbles: true,
    cancelable: true,
    clientY: target.getBoundingClientRect().bottom,
    dataTransfer: transfer,
  }));
}

async function verifyCaptureInteractions(host, context) {
  await waitFor(() => host.querySelectorAll(".queue-item").length === 3, "queue items did not mount");
  assert(host.textContent.includes("正在采集"), "capture state is not visible");
  assert(host.querySelectorAll('[draggable="true"]').length === 3, "capture items are not draggable");
  dragItemAfter(host, 1, 3);
  await waitFor(() => context.snapshot.items.map((item) => item.id).join() === "2,3,1", "drag reorder failed");
  await waitFor(
    () => [...host.querySelectorAll(".queue-item")].map((item) => item.dataset.itemId).join() === "2,3,1",
    "reordered queue did not render",
  );
  required(host, '[data-item-id="2"] .remove-button', "remove button is missing").click();
  await waitFor(() => host.querySelectorAll(".queue-item").length === 2, "queue removal failed");
  await dragItemWithEarlyDragEnd(host, 3, 1);
  await waitFor(() => context.snapshot.items.map((item) => item.id).join() === "1,3", "early drag-end reorder failed");
  await waitFor(
    () => [...host.querySelectorAll(".queue-item")].map((item) => item.dataset.itemId).join() === "1,3",
    "early drag-end reorder did not render",
  );
}

async function verifyPasteInteractions(host, context) {
  required(host, '.mode-switch button[aria-pressed="false"]', "paste mode button is missing").click();
  await waitFor(() => host.querySelector(".direction-switch"), "paste mode did not activate");
  const firstId = context.snapshot.items[0].id;
  const lastId = context.snapshot.items.at(-1).id;
  assert(required(host, `[data-item-id="${firstId}"]`, "forward item is missing").classList.contains("is-next"), "forward next item is wrong");
  const directionButtons = host.querySelectorAll(".direction-switch button");
  assert(directionButtons.length === 2, "direction controls are missing");
  directionButtons[1].click();
  await waitFor(() => required(host, `[data-item-id="${lastId}"]`, "reverse item is missing").classList.contains("is-next"), "reverse mode did not activate");
  assert(host.textContent.includes("Ctrl+Shift+V"), "configured shortcut is not shown");
}

async function verifyNativeCompletion(host, mock) {
  mock.context.snapshot = { ...mock.context.snapshot, items: [] };
  const stateListener = mock.calls.find(
    (call) => call.command === "plugin:event|listen" && call.args.event === "sequential-paste-state-changed",
  );
  mock.callbacks.get(stateListener.args.handler)({ payload: structuredClone(mock.context.snapshot) });
  await waitFor(() => host.textContent.includes("队列已完成"), "completion state did not update from native event");
  required(host, '[aria-label="关闭顺序粘贴"]', "close button is missing").click();
  await waitFor(() => mock.calls.some((call) => call.command === "close_sequential_paste"), "empty queue did not close");
}

export async function runSequentialPasteBrowserChecks() {
  const mock = installMock();
  const view = mountWindow("position:fixed;inset:0;width:320px;height:560px;z-index:999;background:var(--app-background)");
  try {
    await verifyCaptureInteractions(view.host, mock.context);
    await verifyPasteInteractions(view.host, mock.context);
    await verifyNativeCompletion(view.host, mock);
    document.documentElement.className = "theme-dark";
    await nextTick();
    assert(getComputedStyle(view.host.querySelector(".sequential-paste-page")).color !== "", "dark theme did not apply");
    return { passed: true, commands: mock.calls.map((call) => call.command) };
  } finally {
    view.app.unmount();
    view.host.remove();
    await nextTick();
    mock.restore();
  }
}

export function mountSequentialPastePreview(theme = "classic", mode = "capture") {
  const mock = installMock(theme, mode);
  const view = mountWindow("position:fixed;top:20px;right:20px;width:320px;height:560px;z-index:9999");
  return () => {
    view.app.unmount();
    view.host.remove();
    mock.restore();
  };
}
