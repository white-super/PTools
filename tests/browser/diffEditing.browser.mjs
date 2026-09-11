import { createApp, h, nextTick } from "vue";
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

export function mountCardNoticePreview(theme = "soft-glow") {
  const previousTheme = document.documentElement.className;
  document.documentElement.className = `theme-${theme}`;
  const host = document.createElement("div");
  host.className = `paste-flow-panel theme-${theme}`;
  host.style.cssText = "position:fixed;inset:0;z-index:999;padding:40px;justify-content:center";
  document.body.append(host);
  const samples = [
    { label: "文件编码错误", message: "文件不是 UTF-8 文本，请转换编码后重试" },
    { label: "不支持的操作", message: "当前卡片不是文本内容", isSelected: true },
  ];
  const app = createApp({ setup: () => () => h("div", {
    style: "display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:24px;width:min(760px,100%)",
  }, samples.map(({ label, message, ...sample }, index) => h("section", {
    style: "display:grid;grid-template-rows:24px 240px;gap:10px;min-width:0",
  }, [
    h("strong", { style: "color:var(--panel-muted);font-size:13px;font-weight:600" }, label),
    h(PasteCard, {
      content: "对应卡片的内容保持在原位，错误提示短暂悬浮在底部。",
      format: "text",
      filePaths: [],
      isSelected: false,
      quickKey: index + 1,
      notice: { message, cardId: index + 1 },
      ...sample,
    }),
  ]))) });
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
