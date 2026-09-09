import { createApp, h, nextTick, shallowRef } from "vue";
import { EditorView } from "@codemirror/view";
import PasteCard from "../../src/layouts/components/PasteCard/index.vue";
import "../../src/layouts/modes/PasteFlowPanel.css";

function assert(value, message) {
  if (!value) throw new Error(message);
}
const editorFor = (host, side) =>
  EditorView.findFromDOM(
    host.querySelector(`[aria-label="${side}侧对比内容"]`),
  );
const replace = (editor, content) =>
  editor.dispatch({
    changes: { from: 0, to: editor.state.doc.length, insert: content },
    selection: { anchor: content.length },
  });
const shortcut = (target, options) =>
  target.dispatchEvent(
    new KeyboardEvent("keydown", {
      bubbles: true,
      cancelable: true,
      ...options,
    }),
  );

// Synthetic content only: visual checks never read clipboard history or local files.
export function mountCardPreview(theme = "soft-glow") {
  const previousTheme = document.documentElement.className;
  document.documentElement.className = `theme-${theme}`;
  const host = document.createElement("div");
  host.className = `paste-flow-panel theme-${theme}`;
  host.style.cssText = "position:fixed;inset:0;z-index:999;padding:24px;justify-content:center";
  document.body.append(host);
  const samples = [
    { content: "界面优化备忘\n\n让内容成为主角，\n让操作保持轻巧。\n\nPTools · 剪贴板与文本工具", format: "text" },
    { content: '{\n  "name": "PTools",\n  "version": "1.4.0",\n  "features": [\n    "clipboard",\n    "text-diff"\n  ]\n}', format: "text", isDiffSource: true },
    { content: "https://example.com/docs/clipboard\n\n支持文本格式化、差异对比，以及快捷粘贴。", format: "text" },
    { content: "", format: "file", filePaths: ["/example/release-notes.md"] },
  ];
  const app = createApp({ setup: () => () => h("div", {
    style: "display:grid;grid-template-columns:repeat(4,minmax(0,1fr));gap:14px;height:250px",
  }, samples.map((sample, index) => h(PasteCard, {
    filePaths: [], isSelected: index === 1, quickKey: index + 1, ...sample,
  }))) });
  app.mount(host);
  return () => { app.unmount(); host.remove(); document.documentElement.className = previousTheme; };
}

export async function checkLiveEditing(host, calls) {
  const right = editorFor(host, "右");
  const initial = right.state.doc.toString();
  const changed = '{"name":"updated","items":[1,2,3]}';
  assert(right.contentDOM.isContentEditable, "right editor is read-only");
  right.focus();
  replace(right, changed);
  await nextTick();
  assert(
    editorFor(host, "右") === right &&
      document.activeElement === right.contentDOM,
    "typing recreated editor or lost focus",
  );
  assert(
    right.state.selection.main.head === changed.length,
    "typing moved the caret",
  );
  assert(
    right.state.doc.toString() === changed,
    "typing unexpectedly formatted content",
  );
  assert(host.querySelector(".cm-changedText"), "live diff highlights missing");
  const mod = /Mac/.test(navigator.platform)
    ? { metaKey: true }
    : { ctrlKey: true };
  shortcut(right.contentDOM, { key: "z", code: "KeyZ", ...mod });
  await nextTick();
  assert(right.state.doc.toString() === initial, "undo failed");
  shortcut(right.contentDOM, {
    key: "z",
    code: "KeyZ",
    shiftKey: true,
    ...mod,
  });
  await nextTick();
  assert(right.state.doc.toString() === changed, "redo failed");
  const left = editorFor(host, "左");
  assert(left.contentDOM.isContentEditable, "left editor is read-only");
  replace(left, changed);
  await nextTick();
  assert(
    host.querySelector("footer").textContent.includes("当前内容相同"),
    "live equality status is stale",
  );
  host.querySelector('[aria-label="右侧复制内容"]').click();
  await nextTick();
  assert(
    calls.findLast((call) => call.command === "plugin:clipboard|write_text")
      .args.text === changed,
    "copy used stale worker output after direct editing",
  );
  // Restore the left source through its UI before later import tests.
  host.querySelector('[aria-label="左侧恢复原文"]').click();
  await nextTick();
  document.querySelector(".el-message-box__btns .el-button--primary").click();
  return changed;
}

export async function checkInvalidEditing(host, waitFor) {
  let right = editorFor(host, "右");
  replace(right, "{invalid JSON");
  await nextTick();
  assert(
    editorFor(host, "右") === right,
    "invalid JSON stopped direct editing",
  );
  shortcut(right.contentDOM, { key: "r", ctrlKey: true });
  await waitFor(
    () => host.querySelector('.diff-placeholder[role="alert"]'),
    "invalid JSON formatting did not report an error",
  );
  host.querySelector(".diff-placeholder button").click();
  await waitFor(
    () => host.querySelectorAll(".cm-editor").length === 2,
    "cannot return to raw editing",
  );
  right = editorFor(host, "右");
  assert(
    right.state.doc.toString() === "{invalid JSON",
    "returning to raw lost edits",
  );
  replace(right, "raw editable content");
  await nextTick();
  assert(
    editorFor(host, "右") === right &&
      right.state.doc.toString() === "raw editable content",
    "raw editing failed",
  );
}

export async function runCardHeaderChecks() {
  const marked = shallowRef(false);
  const events = [];
  const previousTheme = document.documentElement.className;
  const host = document.createElement("div");
  host.className = "paste-flow-panel theme-soft-glow";
  host.style.cssText =
    "position:fixed;inset:0;width:230px;height:220px;z-index:999";
  document.body.append(host);
  const app = createApp({
    setup: () => () =>
      h(PasteCard, {
        content: "PTools diff header test",
        format: "text",
        filePaths: [],
        quickKey: 1,
        isSelected: true,
        isDiffSource: marked.value,
        onCancelDiff: () => {
          events.push("cancel");
          marked.value = false;
        },
        onSelect: () => events.push("select"),
        onPaste: () => events.push("paste"),
      }),
  });
  try {
    app.mount(host);
    const geometry = () =>
      [".card", ".card-header", ".card-text"].map((selector) => {
        const rect = host.querySelector(selector).getBoundingClientRect();
        return { width: rect.width, height: rect.height, y: rect.y };
      });
    const before = geometry();
    marked.value = true;
    await nextTick();
    assert(
      JSON.stringify(before) === JSON.stringify(geometry()),
      "Diff badge resizes the card or shifts text",
    );
    const selectedCard = host.querySelector(".card");
    const selectedHeader = host.querySelector(".card-header");
    const selectedHeaderBackground = getComputedStyle(selectedHeader).backgroundColor;
    selectedCard.classList.remove("card-selected");
    selectedHeader.classList.remove("card-header-selected");
    const normalHeaderBackground = getComputedStyle(selectedHeader).backgroundColor;
    assert(
      selectedHeaderBackground !== normalHeaderBackground,
      "selected card header does not use a distinct state background",
    );
    selectedCard.classList.add("card-selected");
    selectedHeader.classList.add("card-header-selected");
    const badge = host.querySelector('[aria-label="取消 Diff 对比标记"]');
    badge.dispatchEvent(new MouseEvent("dblclick", { bubbles: true }));
    badge.click();
    await nextTick();
    assert(
      events.join() === "cancel" && !marked.value,
      "cancel badge also selected/pasted the card",
    );
    assert(before[1].height === 32, "card header is not fixed-height");
    const backgrounds = ["classic", "soft-glow", "dark"].map((theme) => {
      document.documentElement.className = `theme-${theme}`;
      host.className = `paste-flow-panel theme-${theme}`;
      const header = getComputedStyle(
        host.querySelector(".card-header"),
      ).backgroundColor;
      assert(
        header !==
          getComputedStyle(host.querySelector(".card")).backgroundColor,
        "header background does not distinguish the badge area",
      );
      return header;
    });
    assert(
      new Set(backgrounds).size === 3,
      "card header does not follow all themes",
    );
    return { passed: true, geometry: before, backgrounds };
  } finally {
    app.unmount();
    host.remove();
    document.documentElement.className = previousTheme;
  }
}
