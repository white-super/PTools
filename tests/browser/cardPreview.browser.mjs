import { createApp, h, nextTick, shallowRef } from "vue";
import PasteCard from "../../src/layouts/components/PasteCard/index.vue";
import "../../src/layouts/modes/PasteFlowPanel.css";

function mountPreview(theme, style, content) {
  const previousTheme = document.documentElement.className;
  document.documentElement.className = `theme-${theme}`;
  const host = document.createElement("div");
  host.className = `paste-flow-panel theme-${theme}`;
  host.style.cssText = style;
  document.body.append(host);
  const app = createApp({ setup: () => () => content() });
  app.mount(host);
  return () => {
    app.unmount();
    host.remove();
    document.documentElement.className = previousTheme;
  };
}

function assert(value, message) {
  if (!value) throw new Error(message);
}

function cardGeometry(host) {
  return [".card", ".card-header", ".card-text"].map((selector) => {
    const rect = host.querySelector(selector).getBoundingClientRect();
    return { width: rect.width, height: rect.height, y: rect.y };
  });
}

function assertSameGeometry(expected, host, message) {
  assert(JSON.stringify(expected) === JSON.stringify(cardGeometry(host)), message);
}

function createHeaderHarness() {
  const diffSide = shallowRef();
  const notice = shallowRef();
  const events = [];
  const host = document.createElement("div");
  host.className = "paste-flow-panel theme-soft-glow";
  host.style.cssText = "position:fixed;inset:0;width:230px;height:220px;z-index:999";
  document.body.append(host);
  const app = createApp({ setup: () => () => h(PasteCard, {
    content: "PTools diff header test", format: "text", filePaths: [],
    quickKey: 1, isSelected: true, diffSide: diffSide.value, notice: notice.value,
    onCancelDiff: () => { events.push("cancel"); diffSide.value = undefined; },
    onSelect: () => events.push("select"), onPaste: () => events.push("paste"),
  }) });
  app.mount(host);
  return { app, diffSide, events, host, notice };
}

async function checkMarkerGeometry(harness, initialGeometry) {
  harness.diffSide.value = "left";
  await nextTick();
  assertSameGeometry(initialGeometry, harness.host, "Diff-L marker changes card geometry");
  harness.notice.value = { message: "当前文本暂不支持格式化工具", cardId: 1 };
  await nextTick();
  const noticeBounds = harness.host.querySelector(".card-notice").getBoundingClientRect();
  const cardBounds = harness.host.querySelector(".card").getBoundingClientRect();
  assertSameGeometry(initialGeometry, harness.host, "notice changes card geometry");
  assert(noticeBounds.top > cardBounds.top + cardBounds.height / 2, "notice is not near the card bottom");
  assert(noticeBounds.bottom < cardBounds.bottom, "notice escapes the card bottom");
}

function checkSelectedVisuals(host, initialGeometry) {
  const card = host.querySelector(".card");
  const header = host.querySelector(".card-header");
  const quickKey = host.querySelector(".quick-key");
  [card, header, quickKey].forEach((element) => { element.style.transition = "none"; });
  const selected = [card, header, quickKey].map((element) => getComputedStyle(element).backgroundColor);
  card.classList.remove("card-selected");
  header.classList.remove("card-header-selected");
  const normal = [card, header, quickKey].map((element) => getComputedStyle(element).backgroundColor);
  assertSameGeometry(initialGeometry, host, "selected state changes card geometry");
  assert(selected[0] === normal[0], "selected state changes the content background");
  assert(selected[1] !== normal[1], "selected header is not visually distinct");
  assert(selected[2] !== normal[2], "selected shortcut is not visually distinct");
  card.classList.add("card-selected");
  header.classList.add("card-header-selected");
}

async function checkDiffMarkers(harness, initialGeometry) {
  const header = harness.host.querySelector(".card-header");
  let marker = harness.host.querySelector('[aria-label="清除左侧文本对比标记"]');
  marker.style.transition = "none";
  const leftBackground = getComputedStyle(marker).backgroundColor;
  assert(marker.textContent.trim() === "Diff-L", "left marker does not show Diff-L");
  assert(marker.getBoundingClientRect().left > header.getBoundingClientRect().left + header.clientWidth / 2,
    "Diff-L marker is not positioned toward the right");
  harness.diffSide.value = "right";
  await nextTick();
  marker = harness.host.querySelector('[aria-label="清除右侧文本对比标记"]');
  assert(marker.textContent.trim() === "DIFF-R", "right marker does not show DIFF-R");
  assert(getComputedStyle(marker).backgroundColor !== leftBackground, "Diff-L and DIFF-R look identical");
  assertSameGeometry(initialGeometry, harness.host, "DIFF-R marker changes card geometry");
  marker.dispatchEvent(new MouseEvent("dblclick", { bubbles: true }));
  marker.click();
  await nextTick();
  assert(harness.events.join() === "cancel" && !harness.diffSide.value,
    "marker cancellation also selected or pasted the card");
}

function checkThemeBackgrounds(host) {
  return ["classic", "soft-glow", "dark"].map((theme) => {
    document.documentElement.className = `theme-${theme}`;
    host.className = `paste-flow-panel theme-${theme}`;
    const header = getComputedStyle(host.querySelector(".card-header")).backgroundColor;
    const card = getComputedStyle(host.querySelector(".card")).backgroundColor;
    assert(header !== card, "header background does not distinguish the marker area");
    return header;
  });
}

export async function runCardHeaderChecks() {
  const previousTheme = document.documentElement.className;
  const harness = createHeaderHarness();
  try {
    const geometry = cardGeometry(harness.host);
    await checkMarkerGeometry(harness, geometry);
    checkSelectedVisuals(harness.host, geometry);
    await checkDiffMarkers(harness, geometry);
    assert(geometry[1].height === 34, "card header is not fixed-height");
    const backgrounds = checkThemeBackgrounds(harness.host);
    assert(new Set(backgrounds).size === 3, "card header does not follow all themes");
    return { passed: true, geometry, backgrounds };
  } finally {
    harness.app.unmount();
    harness.host.remove();
    document.documentElement.className = previousTheme;
  }
}

// Synthetic content only: visual previews never read clipboard history or local files.
export function mountCardPreview(theme = "soft-glow") {
  const samples = [
    { content: "界面优化备忘\n\n让内容成为主角，\n让操作保持轻巧。\n\nPTools · 剪贴板与文本工具", format: "text" },
    { content: '{\n  "name": "PTools",\n  "version": "1.4.0",\n  "features": [\n    "clipboard",\n    "text-diff"\n  ]\n}', format: "text", diffSide: "left" },
    { content: "https://example.com/docs/clipboard\n\n支持文本格式化、差异对比，以及快捷粘贴。", format: "text" },
    { content: "", format: "file", filePaths: ["/example/release-notes.md"] },
  ];
  return mountPreview(
    theme,
    "position:fixed;inset:0;z-index:999;padding:24px;justify-content:center",
    () => h("div", {
      style: "display:grid;grid-template-columns:repeat(4,minmax(0,1fr));gap:14px;height:250px",
    }, samples.map((sample, index) => h(PasteCard, {
      filePaths: [], isSelected: index === 1, quickKey: index + 1, ...sample,
    }))),
  );
}

export function mountCardStatePreview(theme = "soft-glow") {
  const samples = [
    { label: "默认", content: "保持内容清晰，减少不必要的视觉干扰。", quickKey: 1 },
    { label: "悬停", content: "https://ptools.example/tools\n\n鼠标悬停时轻微浮起。", quickKey: 2, diffSide: "left" },
    { label: "选中", content: '{\n  "name": "PTools",\n  "state": "selected"\n}', quickKey: 3, isSelected: true, diffSide: "right" },
  ];
  return mountPreview(
    theme,
    "position:fixed;inset:0;z-index:999;padding:40px;justify-content:center",
    () => h("div", {
      style: "display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:24px;width:min(1120px,100%)",
    }, samples.map(({ label, ...sample }) => h("section", {
      style: "display:grid;grid-template-rows:24px 250px;gap:10px;min-width:0",
    }, [
      h("strong", { style: "color:var(--panel-muted);font-size:13px;font-weight:600" }, label),
      h(PasteCard, { format: "text", filePaths: [], isSelected: false, ...sample }),
    ]))),
  );
}
