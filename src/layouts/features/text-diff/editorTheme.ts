import { HighlightStyle, syntaxHighlighting } from "@codemirror/language";
import { EditorView } from "@codemirror/view";
import { tags } from "@lezer/highlight";
import { json } from "@codemirror/lang-json";
import { xml } from "@codemirror/lang-xml";
import { html } from "@codemirror/lang-html";
import type { DiffFormat } from "./types";

export function diffLanguage(format: DiffFormat) {
  if (format === "json") return json();
  if (format === "xml") return xml();
  if (format === "html") return html();
  return [];
}

export const diffHighlight = syntaxHighlighting(
  HighlightStyle.define([
    { tag: [tags.propertyName, tags.tagName], color: "var(--formatter-code-property)" },
    { tag: [tags.string, tags.attributeValue], color: "var(--formatter-code-string)" },
    { tag: tags.number, color: "var(--formatter-code-number)" },
    { tag: [tags.bool, tags.null, tags.attributeName], color: "var(--formatter-code-atom)" },
    { tag: tags.comment, color: "var(--app-muted)", fontStyle: "italic" },
  ]),
);

export const diffEditorTheme = EditorView.theme({
  "&": { color: "var(--app-text)", background: "var(--formatter-editor-background)", fontSize: "13px" },
  "&.cm-focused": { outline: "none" },
  ".cm-scroller": { fontFamily: "SFMono-Regular, Consolas, monospace", lineHeight: "1.6" },
  ".cm-content": { padding: "8px 0", caretColor: "var(--app-primary)" },
  ".cm-line": { padding: "0 10px" },
  ".cm-gutters": {
    color: "var(--app-muted)",
    background: "var(--formatter-gutter-background)",
    borderColor: "var(--app-border)",
  },
  ".cm-activeLine, .cm-activeLineGutter": { background: "var(--app-hover)" },
  ".cm-selectionBackground, &.cm-focused .cm-selectionBackground": {
    background: "var(--formatter-selection) !important",
  },
  "&.cm-merge-a .cm-changedLine": { background: "color-mix(in srgb, var(--app-danger) 13%, transparent)" },
  "&.cm-merge-b .cm-changedLine": { background: "color-mix(in srgb, var(--app-success) 13%, transparent)" },
  "&.cm-merge-a .cm-changedText": {
    background: "color-mix(in srgb, var(--app-danger) 30%, transparent)",
    textDecoration: "none",
  },
  "&.cm-merge-b .cm-changedText": {
    background: "color-mix(in srgb, var(--app-success) 30%, transparent)",
    textDecoration: "none",
  },
  "&.cm-merge-a .cm-changedLineGutter": { background: "var(--app-danger)" },
  "&.cm-merge-b .cm-changedLineGutter": { background: "var(--app-success)" },
  ".cm-collapsedLines": {
    color: "var(--app-muted)",
    background: "var(--app-active)",
    borderColor: "var(--app-border)",
  },
});
