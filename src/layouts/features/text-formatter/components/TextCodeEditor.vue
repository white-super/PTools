<script setup lang="ts">
import { HighlightStyle, foldAll, syntaxHighlighting, unfoldAll } from "@codemirror/language";
import { closeSearchPanel, search } from "@codemirror/search";
import { tags } from "@lezer/highlight";
import { basicSetup, EditorView } from "codemirror";
import { onMounted, onUnmounted, shallowRef, useTemplateRef, watch } from "vue";
import { json } from "@codemirror/lang-json";
import { codeMirrorChineseLocalization } from "../../../utils/codeMirrorLocalization";
import type { TextFormat } from "../types";

const content = defineModel<string>({ required: true });
const props = defineProps<{
  format?: TextFormat;
  editorLanguage?: "json" | "plain";
  lineWrapping: boolean;
}>();
const editorHost = useTemplateRef<HTMLDivElement>("editorHost");
const editorView = shallowRef<EditorView>();

const editorTheme = EditorView.theme({
  "&": {
    height: "100%",
    color: "var(--app-text)",
    backgroundColor: "var(--formatter-editor-background)",
  },
  "&.cm-focused": { outline: "none" },
  ".cm-scroller": {
    overflow: "auto",
    fontFamily: "SFMono-Regular, Consolas, Liberation Mono, monospace",
    fontSize: "13px",
    lineHeight: "1.65",
  },
  ".cm-content": { padding: "16px 0", caretColor: "var(--app-primary)" },
  ".cm-line": { padding: "0 18px" },
  ".cm-gutters": {
    color: "var(--app-muted)",
    backgroundColor: "var(--formatter-gutter-background)",
    borderRight: "1px solid var(--app-border)",
  },
  ".cm-gutterElement": { padding: "0 10px 0 14px" },
  ".cm-activeLine, .cm-activeLineGutter": { backgroundColor: "var(--app-hover)" },
  ".cm-foldPlaceholder": {
    color: "var(--app-muted)",
    backgroundColor: "var(--app-active)",
    borderColor: "var(--app-border)",
  },
  ".cm-selectionBackground, &.cm-focused .cm-selectionBackground": {
    backgroundColor: "var(--formatter-selection) !important",
  },
});

const highlightTheme = HighlightStyle.define([
  { tag: tags.propertyName, color: "var(--formatter-code-property)" },
  { tag: tags.string, color: "var(--formatter-code-string)" },
  { tag: tags.number, color: "var(--formatter-code-number)" },
  { tag: [tags.bool, tags.null], color: "var(--formatter-code-atom)" },
  { tag: [tags.brace, tags.squareBracket, tags.separator], color: "var(--app-secondary)" },
  { tag: tags.comment, color: "var(--app-muted)", fontStyle: "italic" },
]);

function createEditor(parent: HTMLElement) {
  return new EditorView({
    parent,
    doc: content.value,
    extensions: [
      basicSetup,
      search({ top: true }),
      codeMirrorChineseLocalization,
      ...(props.lineWrapping ? [EditorView.lineWrapping] : []),
      ...(props.editorLanguage === "json" ? [json()] : []),
      editorTheme,
      syntaxHighlighting(highlightTheme),
      EditorView.updateListener.of((update) => {
        if (update.docChanged) {
          content.value = update.state.doc.toString();
        }
      }),
    ],
  });
}

function replaceEditorContent(value: string) {
  const view = editorView.value;
  if (!view || view.state.doc.toString() === value) {
    return;
  }
  view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: value } });
}

function recreateEditor() {
  const view = editorView.value;
  const parent = editorHost.value;
  if (!view || !parent) {
    return;
  }
  const selection = view.state.selection.main;
  const documentLength = content.value.length;
  view.destroy();
  editorView.value = createEditor(parent);
  editorView.value.dispatch({
    selection: {
      anchor: Math.min(selection.anchor, documentLength),
      head: Math.min(selection.head, documentLength),
    },
  });
}

function foldAllContent() {
  if (editorView.value) foldAll(editorView.value);
}

function unfoldAllContent() {
  if (editorView.value) unfoldAll(editorView.value);
}

function getSelectedText() {
  const view = editorView.value;
  if (!view) return undefined;
  const selection = view.state.selection.main;
  if (selection.empty) return undefined;
  return view.state.sliceDoc(selection.from, selection.to);
}

function replaceSelectedText(value: string) {
  const view = editorView.value;
  if (!view || view.state.selection.main.empty) return false;
  const selection = view.state.selection.main;
  view.dispatch({ changes: { from: selection.from, to: selection.to, insert: value } });
  return true;
}

function closeSearch() {
  const view = editorView.value;
  return view ? closeSearchPanel(view) : false;
}

watch(content, replaceEditorContent);
watch(() => props.editorLanguage, recreateEditor);
watch(() => props.lineWrapping, recreateEditor);

onMounted(() => {
  if (editorHost.value) {
    editorView.value = createEditor(editorHost.value);
  }
});

onUnmounted(() => {
  editorView.value?.destroy();
  editorView.value = undefined;
});

defineExpose({
  closeSearch,
  foldAll: foldAllContent,
  getSelectedText,
  replaceSelectedText,
  unfoldAll: unfoldAllContent,
});
</script>

<template>
  <div class="code-editor-shell">
    <div ref="editorHost" class="code-editor"></div>
    <span v-if="content.length === 0" class="code-editor-placeholder">粘贴或输入内容</span>
  </div>
</template>

<style scoped>
.code-editor-shell {
  position: relative;
  min-height: 0;
  overflow: hidden;
  background: var(--formatter-editor-background);
}

.code-editor {
  height: 100%;
}

.code-editor-placeholder {
  position: absolute;
  top: 18px;
  left: 70px;
  color: var(--app-muted);
  font-family: SFMono-Regular, Consolas, Liberation Mono, monospace;
  font-size: 13px;
  pointer-events: none;
}
</style>
