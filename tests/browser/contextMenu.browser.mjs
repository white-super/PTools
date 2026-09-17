import { createApp, h, nextTick } from "vue";
import CardContextMenu from "../../src/layouts/components/CardContextMenu.vue";
import PasteCard from "../../src/layouts/components/PasteCard/index.vue";
import "../../src/layouts/modes/PasteFlowPanel.css";

const TOOLS = [
  { format: "json", title: "JSON 格式化" },
  { format: "xml", title: "XML 格式化" },
  { format: "html", title: "HTML 格式化" },
  { format: "url", title: "URL 编解码" },
  { format: "date", title: "日期转换" },
  { format: "base64", title: "Base64 编解码" },
];
const TAGS = [
  { id: 1, name: "工作", color: "#4f8edc" },
  { id: 2, name: "稍后处理", color: "#e39a46" },
  { id: 3, name: "重要", color: "#d65b65" },
];
const MENU_BACKGROUNDS = {
  classic: "rgb(255, 255, 255)",
  "soft-glow": "rgb(251, 251, 252)",
  dark: "rgb(41, 44, 49)",
};
const SUBMENU_TRANSITION_SETTLE_MS = 120;

function assert(value, message) {
  if (!value) throw new Error(message);
}

function setTheme(host, theme) {
  document.documentElement.className = `theme-${theme}`;
  host.className = `paste-flow-panel theme-${theme}`;
}

function createPreview(theme) {
  const previousTheme = document.documentElement.className;
  const host = document.createElement("div");
  host.style.cssText = "position:fixed;inset:0;z-index:999;min-width:820px;min-height:460px";
  setTheme(host, theme);
  document.body.append(host);
  const app = createApp({
    setup: () => () => h("div", [
      h("div", { style: "position:absolute;left:80px;top:100px;width:250px;height:210px" }, [
        h(PasteCard, {
          content: "https://ptools.example/tools\n\n右键菜单视觉预览",
          format: "text",
          filePaths: [],
          isSelected: true,
          quickKey: 1,
        }),
      ]),
      h(CardContextMenu, {
        x: 380,
        y: 74,
        tags: TAGS,
        assignedTagIds: [1],
        tools: TOOLS,
        diffLabel: "文本对比",
        diffDisabled: false,
      }),
    ]),
  });
  app.mount(host);
  return {
    host,
    cleanup: () => {
      app.unmount();
      host.remove();
      document.documentElement.className = previousTheme;
    },
  };
}

async function waitForSubmenuTransition() {
  await new Promise((resolve) => setTimeout(resolve, SUBMENU_TRANSITION_SETTLE_MS));
}

export async function runCardContextMenuChecks() {
  const preview = createPreview("classic");
  try {
    await nextTick();
    const menu = preview.host.querySelector(".context-menu:not(.context-menu-submenu)");
    const toolsTrigger = menu.querySelector('[aria-haspopup="menu"]');
    assert(menu.getBoundingClientRect().width === 212, "context menu width is incorrect");
    assert(menu.querySelectorAll(".card-menu-icon").length >= 4, "context menu icons are missing");
    assert(menu.textContent.includes("F") && menu.textContent.includes("D"), "shortcut hints are missing");
    const backgrounds = Object.entries(MENU_BACKGROUNDS).map(([theme, expected]) => {
      setTheme(preview.host, theme);
      const background = getComputedStyle(menu).backgroundColor;
      assert(background === expected, `${theme} menu background is not opaque`);
      return background;
    });
    const submenu = menu.querySelector(".context-menu-submenu");
    toolsTrigger.focus();
    assert(getComputedStyle(submenu).visibility === "visible", "tool submenu did not open on focus");
    await waitForSubmenuTransition();
    const gap = submenu.getBoundingClientRect().left - menu.getBoundingClientRect().right;
    assert(gap >= 1 && gap <= 3, "submenu spacing is incorrect");
    return { passed: true, width: 212, gap, backgrounds };
  } finally {
    preview.cleanup();
  }
}

export function mountCardContextMenuPreview(theme = "classic") {
  return createPreview(theme).cleanup;
}
