import { createApp, h, nextTick, shallowRef } from "vue";
import QuickToolsBar from "../../src/layouts/features/quick-tools/components/QuickToolsBar.vue";
import { useTextFormatterLauncher } from "../../src/layouts/composables/useTextFormatterLauncher";
import { useQuickTools } from "../../src/layouts/features/quick-tools/useQuickTools";

const DEFAULT_TOOLS = ["text-diff", "sequential-paste", "json", "url", "base64"];
const DEFAULT_SHORTCUTS = ["Command+1", "Command+2", "Command+3", "Command+4", "Command+5"];

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

function setTheme(theme) {
  document.documentElement.classList.remove("theme-soft-glow", "theme-classic", "theme-dark");
  document.documentElement.classList.add(`theme-${theme}`);
}

function mountHarness(theme = "classic") {
  const host = document.createElement("div");
  const toolIds = shallowRef([...DEFAULT_TOOLS]);
  const activeToolIds = shallowRef(["json"]);
  const managerOpen = shallowRef(false);
  const executed = [];
  host.className = "quick-tools-browser-host";
  host.style.width = "300px";
  document.body.append(host);
  setTheme(theme);
  const app = createApp({
    setup() {
      return () => h(QuickToolsBar, {
        modelValue: managerOpen.value,
        toolIds: toolIds.value,
        toolShortcuts: DEFAULT_SHORTCUTS,
        activeToolIds: activeToolIds.value,
        saving: false,
        "onUpdate:modelValue": (open) => { managerOpen.value = open; },
        onExecute: (toolId) => executed.push(toolId),
        onOrderChange: (nextToolIds) => { toolIds.value = [...nextToolIds]; },
        onError: (error) => { throw error; },
      });
    },
  });
  app.mount(host);
  return { activeToolIds, app, executed, host, managerOpen, toolIds };
}

function unmountHarness(harness, previousTheme) {
  harness.app.unmount();
  harness.host.remove();
  document.documentElement.className = previousTheme;
}

function verifyToolbarAppearance(harness) {
  const bar = harness.host.querySelector(".quick-tools-bar");
  const slots = [...bar.querySelectorAll(".quick-tool-slot")];
  assert(slots.length === 5, "toolbar does not render exactly five configured tools");
  assert(bar.scrollWidth <= harness.host.clientWidth, "toolbar overflows a narrow host");
  assert(getComputedStyle(bar).boxShadow === "none", "toolbar has a forbidden shadow");
  assert(getComputedStyle(bar).backgroundColor === "rgba(0, 0, 0, 0)", "toolbar is not transparent");
  slots.forEach((slot, index) => {
    const button = slot.querySelector("button");
    const buttonRect = button.getBoundingClientRect();
    assert(Math.round(buttonRect.width) === 30 && Math.round(buttonRect.height) === 28, "tool button size changed");
    assert(slot.querySelector("kbd") === null, "shortcut number should not be displayed");
    assert(getComputedStyle(button).boxShadow === "none", "tool button has a forbidden shadow");
    assert(button.getAttribute("title") === null, "tool button still uses the delayed native tooltip");
    assert(slot.querySelector(".quick-tool-tooltip")?.textContent.includes(`cmd+${index + 1}`), "custom tooltip is wrong");
  });
  assert(bar.querySelector('[data-tool-id="json"]').classList.contains("is-active"), "open tool state is not visible");
  assert(slots[0].querySelector("button").getAttribute("aria-label").includes("cmd+1"), "compact shortcut is missing");
  slots[0].querySelector("button").click();
  assert(harness.executed[0] === "text-diff", "tool click executes the wrong item");
  return bar;
}

function dragAfter(source, target) {
  const transfer = new DataTransfer();
  const targetRect = target.getBoundingClientRect();
  source.dispatchEvent(new DragEvent("dragstart", { bubbles: true, dataTransfer: transfer }));
  target.dispatchEvent(new DragEvent("dragover", {
    bubbles: true,
    cancelable: true,
    clientX: targetRect.right - 1,
    dataTransfer: transfer,
  }));
  target.dispatchEvent(new DragEvent("drop", { bubbles: true, cancelable: true, dataTransfer: transfer }));
  source.dispatchEvent(new DragEvent("dragend", { bubbles: true, dataTransfer: transfer }));
}

async function verifyManagerInteractions(harness, bar) {
  bar.querySelector('[aria-label="管理快捷工具"]').click();
  await nextTick();
  assert(bar.querySelector(".quick-tool-manager"), "tool manager did not open");
  assert(
    bar.querySelector('.manager-tool[data-tool-id="text-diff"] .manager-shortcut')?.textContent === "cmd+1",
    "manager does not display the configured shortcut",
  );
  bar.querySelector('.manager-tool[data-tool-id="sequential-paste"]').click();
  await nextTick();
  assert(harness.toolIds.value.length === 4, "selected tool was not removed");
  dragAfter(bar.querySelector('.manager-tool[data-tool-id="xml"]'), bar.querySelector('[data-tool-id="base64"]'));
  await nextTick();
  assert(harness.toolIds.value.at(-1) === "xml", "manager drag did not add the tool");
  assert(bar.querySelectorAll(".quick-tool-shortcut").length === 0, "shortcut numbers should stay hidden");
  dragAfter(bar.querySelector('[data-tool-id="text-diff"] .quick-tool-button'), bar.querySelector('[data-tool-id="xml"]'));
  await nextTick();
  assert(harness.toolIds.value.at(-1) === "text-diff", "dragging did not reorder the tools");
  assert(
    bar.querySelector('.manager-tool[data-tool-id="text-diff"] .manager-shortcut')?.textContent === "cmd+5",
    "shortcut did not follow the reordered position",
  );
  assert(bar.querySelector(".manager-remove-zone") === null, "obsolete drag-to-remove zone is still rendered");
}

export async function runQuickToolsBrowserChecks() {
  const previousTheme = document.documentElement.className;
  const harness = mountHarness();
  try {
    await nextTick();
    const bar = verifyToolbarAppearance(harness);
    await verifyManagerInteractions(harness, bar);
    setTheme("dark");
    await nextTick();
    assert(getComputedStyle(bar.querySelector(".quick-tool-button")).boxShadow === "none", "dark theme adds a tool shadow");
    return { passed: true, tools: [...harness.toolIds.value] };
  } finally {
    unmountHarness(harness, previousTheme);
    await nextTick();
  }
}

function installLaunchMock() {
  const previous = window.__TAURI_INTERNALS__;
  const state = { formatterInputs: [], sequentialPasteOpens: 0 };
  window.__TAURI_INTERNALS__ = {
    transformCallback: () => 1,
    invoke: async (command, args) => {
      if (command === "get_active_quick_tools") return [];
      if (command === "show_text_formatter") {
        state.formatterInputs.push(structuredClone(args.input));
        return "text-formatter-browser-test";
      }
      if (command === "show_sequential_paste") {
        state.sequentialPasteOpens += 1;
        return { enabled: true, mode: "capture", direction: "forward", items: [] };
      }
      if (["plugin:event|listen", "plugin:event|unlisten"].includes(command)) return 1;
      throw new Error(`Unexpected native call: ${command}`);
    },
  };
  return { previous, state };
}

function mountLaunchHarness(selectedCard) {
  const host = document.createElement("div");
  document.body.append(host);
  let quickTools;
  const app = createApp({
    setup() {
      const launcher = useTextFormatterLauncher((error) => { throw error; });
      quickTools = useQuickTools({
        cards: shallowRef([selectedCard]),
        selectedCardId: shallowRef(selectedCard.id),
        settings: shallowRef(),
        saveToolIds: async () => {},
        closeMenus: () => {},
        openTextDiff: () => {},
        openSequentialPaste: () => { void window.__TAURI_INTERNALS__.invoke("show_sequential_paste"); },
        openEmptyTextFormatter: launcher.openEmptyTextFormatter,
        openTextFormatter: launcher.openTextFormatterWithFormat,
        reportError: (error) => { throw error; },
      });
      return () => null;
    },
  });
  app.mount(host);
  return { app, host, quickTools };
}

async function verifyLaunchInputs(quickTools, selectedCard, state) {
  quickTools.executeEmptyTool("json");
  await waitFor(() => state.formatterInputs.length === 1, "toolbar tool did not open a formatter");
  assert(
    JSON.stringify(state.formatterInputs[0]) === '{"format":"json","content":"","updatedAt":0}',
    "toolbar tool imported clipboard content",
  );
  quickTools.executeSelectedTool("json");
  await waitFor(() => state.formatterInputs.length === 2, "card shortcut did not open a formatter");
  assert(
    state.formatterInputs[1].content === selectedCard.content
      && state.formatterInputs[1].updatedAt === selectedCard.updatedAt,
    "card shortcut did not import the selected card",
  );
  quickTools.executeEmptyTool("sequential-paste");
  quickTools.executeSelectedTool("sequential-paste");
  await waitFor(() => state.sequentialPasteOpens === 2, "sequential paste tool did not open from both entries");
  assert(state.formatterInputs.length === 2, "sequential paste imported the selected card into a formatter");
}

export async function runQuickToolLaunchChecks() {
  const mock = installLaunchMock();
  const selectedCard = { id: 7, format: "text", content: '{"source":"selected-card"}', updatedAt: 42 };
  const harness = mountLaunchHarness(selectedCard);
  try {
    await verifyLaunchInputs(harness.quickTools, selectedCard, mock.state);
    return { passed: true, formatterInputs: mock.state.formatterInputs };
  } finally {
    harness.app.unmount();
    harness.host.remove();
    await nextTick();
    window.__TAURI_INTERNALS__ = mock.previous;
  }
}

export function mountQuickToolsPreview(theme = "classic") {
  const previousTheme = document.documentElement.className;
  const harness = mountHarness(theme);
  harness.host.style.margin = "36px";
  harness.host.style.padding = "18px";
  harness.host.style.background = "var(--app-background)";
  return () => unmountHarness(harness, previousTheme);
}
