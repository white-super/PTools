import { createApp, h, nextTick, shallowRef } from "vue";
import QuickToolsBar from "../../src/layouts/features/quick-tools/components/QuickToolsBar.vue";

const DEFAULT_TOOLS = ["text-diff", "json", "url", "base64", "date"];
const DEFAULT_SHORTCUTS = ["Command+1", "Command+2", "Command+3", "Command+4", "Command+5"];

function assert(value, message) {
  if (!value) throw new Error(message);
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

export async function runQuickToolsBrowserChecks() {
  const previousTheme = document.documentElement.className;
  const harness = mountHarness();
  try {
    await nextTick();
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
      assert(
        slot.querySelector(".quick-tool-tooltip")?.textContent.includes(DEFAULT_SHORTCUTS[index]),
        "custom tooltip does not display the configured shortcut",
      );
    });

    const jsonSlot = bar.querySelector('[data-tool-id="json"]');
    assert(jsonSlot.classList.contains("is-active"), "open tool state is not visible");
    const firstButton = slots[0].querySelector("button");
    assert(firstButton.getAttribute("aria-label").includes("Command+1"), "configured shortcut is missing from the tool label");
    firstButton.click();
    assert(harness.executed[0] === "text-diff", "tool click executes the wrong item");

    bar.querySelector('[aria-label="管理快捷工具"]').click();
    await nextTick();
    assert(bar.querySelector(".quick-tool-manager"), "tool manager did not open");
    assert(
      bar.querySelector('.manager-tool[data-tool-id="text-diff"] .manager-shortcut')?.textContent === "Command+1",
      "manager does not display the configured shortcut",
    );
    bar.querySelector('.manager-tool[data-tool-id="date"]').click();
    await nextTick();
    assert(harness.toolIds.value.length === 4, "selected tool was not removed");
    const addTransfer = new DataTransfer();
    const xmlManagerItem = bar.querySelector('.manager-tool[data-tool-id="xml"]');
    const base64Slot = bar.querySelector('[data-tool-id="base64"]');
    const base64Rect = base64Slot.getBoundingClientRect();
    xmlManagerItem.dispatchEvent(new DragEvent("dragstart", { bubbles: true, dataTransfer: addTransfer }));
    base64Slot.dispatchEvent(new DragEvent("dragover", {
      bubbles: true,
      cancelable: true,
      clientX: base64Rect.right - 1,
      dataTransfer: addTransfer,
    }));
    base64Slot.dispatchEvent(new DragEvent("drop", { bubbles: true, cancelable: true, dataTransfer: addTransfer }));
    xmlManagerItem.dispatchEvent(new DragEvent("dragend", { bubbles: true, dataTransfer: addTransfer }));
    await nextTick();
    assert(harness.toolIds.value.at(-1) === "xml", "manager drag did not add the tool");
    assert(bar.querySelectorAll(".quick-tool-shortcut").length === 0, "shortcut numbers should stay hidden");

    const transfer = new DataTransfer();
    const draggedButton = bar.querySelector('[data-tool-id="text-diff"] .quick-tool-button');
    const targetSlot = bar.querySelector('[data-tool-id="xml"]');
    const targetRect = targetSlot.getBoundingClientRect();
    draggedButton.dispatchEvent(new DragEvent("dragstart", { bubbles: true, dataTransfer: transfer }));
    targetSlot.dispatchEvent(new DragEvent("dragover", {
      bubbles: true,
      cancelable: true,
      clientX: targetRect.right - 1,
      dataTransfer: transfer,
    }));
    targetSlot.dispatchEvent(new DragEvent("drop", { bubbles: true, cancelable: true, dataTransfer: transfer }));
    draggedButton.dispatchEvent(new DragEvent("dragend", { bubbles: true, dataTransfer: transfer }));
    await nextTick();
    assert(harness.toolIds.value.at(-1) === "text-diff", "dragging did not reorder the tools");
    assert(
      bar.querySelector('.manager-tool[data-tool-id="text-diff"] .manager-shortcut')?.textContent === "Command+5",
      "shortcut did not follow the reordered position",
    );

    setTheme("dark");
    await nextTick();
    const darkButton = bar.querySelector(".quick-tool-button");
    assert(getComputedStyle(darkButton).boxShadow === "none", "dark theme adds a tool shadow");
    return { passed: true, tools: [...harness.toolIds.value] };
  } finally {
    unmountHarness(harness, previousTheme);
    await nextTick();
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
