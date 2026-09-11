<script setup lang="ts">
import { onMounted, onUnmounted, useTemplateRef, watch } from "vue";
import { Change, MergeView } from "@codemirror/merge";
import { search } from "@codemirror/search";
import { Compartment, EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { basicSetup } from "codemirror";
import { diffEditorTheme, diffHighlight, diffLanguage } from "../editorTheme";
import type { DiffResult, DiffSide } from "../types";

const props = defineProps<{ result: DiffResult; wrapping: boolean; collapse: boolean }>();
const emit = defineEmits<{
  stats: [count: number, current: number];
  edit: [side: DiffSide, content: string];
}>();
const host = useTemplateRef<HTMLDivElement>("host");
const wrapA = new Compartment();
const wrapB = new Compartment();
let view: MergeView | undefined;
let current = -1;

function extensions(wrap: Compartment, side: DiffSide) {
  return [basicSetup, search({ top: true }), diffEditorTheme, diffHighlight, diffLanguage(props.result.format),
    EditorState.lineSeparator.of("\n"),
    EditorView.contentAttributes.of({ "aria-label": side === "left" ? "左侧对比内容" : "右侧对比内容" }),
    EditorView.updateListener.of(update => {
      if (!update.docChanged) return;
      current = -1;
      emit("edit", side, update.state.doc.toString());
      emit("stats", view!.chunks.length, 0);
    }),
    wrap.of(props.wrapping ? EditorView.lineWrapping : [])];
}

function create() {
  view?.destroy();
  const changes = props.result.changes.map(c => new Change(c.fromA, c.toA, c.fromB, c.toB));
  view = new MergeView({
    parent: host.value!,
    a: { doc: props.result.left, extensions: extensions(wrapA, "left") },
    b: { doc: props.result.right, extensions: extensions(wrapB, "right") },
    diffConfig: { override: () => changes },
    collapseUnchanged: props.collapse ? {} : undefined,
  });
  // Initial full diff comes from the worker. Later edits use CodeMirror's precise default diffing.
  view.reconfigure({ diffConfig: {} });
  current = -1;
  emit("stats", view.chunks.length, 0);
}

function navigate(direction: -1 | 1) {
  if (!view?.chunks.length) return;
  current = current < 0 ? (direction === 1 ? 0 : view.chunks.length - 1)
    : (current + direction + view.chunks.length) % view.chunks.length;
  const chunk = view.chunks[current];
  const a = Math.min(chunk.fromA, view.a.state.doc.length);
  const b = Math.min(chunk.fromB, view.b.state.doc.length);
  view.a.dispatch({ selection: { anchor: a }, effects: EditorView.scrollIntoView(a, { y: "center" }) });
  view.b.dispatch({ selection: { anchor: b } });
  emit("stats", view.chunks.length, current + 1);
}

watch(() => props.result, create);
watch(() => props.wrapping, wrapping => {
  view?.a.dispatch({ effects: wrapA.reconfigure(wrapping ? EditorView.lineWrapping : []) });
  view?.b.dispatch({ effects: wrapB.reconfigure(wrapping ? EditorView.lineWrapping : []) });
});
watch(() => props.collapse, collapse => view?.reconfigure({ collapseUnchanged: collapse ? {} : undefined }));
onMounted(create);
onUnmounted(() => view?.destroy());
defineExpose({ navigate });
</script>

<template><div ref="host" class="diff-merge-host" /></template>

<style scoped>
.diff-merge-host { min-height: 0; overflow: hidden; }
.diff-merge-host :deep(.cm-mergeView) { height: 100%; overflow: auto; }
.diff-merge-host :deep(.cm-mergeViewEditors) { min-height: 100%; }
.diff-merge-host :deep(.cm-mergeViewEditor) { min-width: 0; }
.diff-merge-host :deep(.cm-mergeViewEditor + .cm-mergeViewEditor) { border-left: 1px solid var(--app-border); }
</style>
