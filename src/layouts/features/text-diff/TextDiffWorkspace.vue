<script setup lang="ts">
import { onMounted, onUnmounted, shallowRef, useTemplateRef, watch } from "vue";
import DiffToolbar from "./components/DiffToolbar.vue";
import DiffSourceHeader from "./components/DiffSourceHeader.vue";
import DiffMergeEditor from "./components/DiffMergeEditor.vue";
import { useDiffWorkspace } from "./useDiffWorkspace";

const {
  result, busy, error, original, left, right, format, formatted, edit, hasLiveEdits,
  wrapping, collapse, pinned, operationError, status, loading, compare, close, togglePinned,
  importFile, restore, swap, copy, cancel,
} = useDiffWorkspace();
const editor = useTemplateRef<InstanceType<typeof DiffMergeEditor>>("editor");
const count = shallowRef(0);
const current = shallowRef(0);

function updateStats(total: number, position: number) { count.value = total; current.value = position; }
function navigate(direction: -1 | 1) { editor.value?.navigate(direction); }

function handleKeydown(event: KeyboardEvent) {
  if (event.isComposing || event.repeat || event.defaultPrevented) return;
  if (event.target instanceof Element && event.target.closest(".el-overlay")) return;
  const key = event.key.toLowerCase();
  if (key === "d" && !event.ctrlKey && !event.shiftKey && event.metaKey !== event.altKey) {
    event.preventDefault(); void togglePinned(); return;
  }
  if (key === "w" && !event.altKey && !event.shiftKey && event.metaKey !== event.ctrlKey) {
    event.preventDefault(); void close(); return;
  }
  if (!event.ctrlKey || event.altKey || event.metaKey || event.shiftKey) return;
  if (key === "r") { event.preventDefault(); compare(); }
  if (key === "q" || key === "e") { event.preventDefault(); navigate(key === "q" ? -1 : 1); }
}

function closeMore(event: MouseEvent) {
  const target = event.target;
  if (target instanceof Element && target.closest("details")) return;
  document.querySelectorAll("details[open]").forEach(element => element.removeAttribute("open"));
}

watch(result, () => { count.value = 0; current.value = 0; });
onMounted(() => window.addEventListener("keydown", handleKeydown, true));
onUnmounted(() => window.removeEventListener("keydown", handleKeydown, true));
</script>

<template>
  <main class="diff-page" @contextmenu.prevent @click="closeMore">
    <header class="diff-header" data-tauri-drag-region>
      <button type="button" class="window-button" title="关闭对比窗口 (Command+W / Ctrl+W)" aria-label="关闭对比窗口" @click="close">×</button>
      <h1 data-tauri-drag-region>文本对比</h1>
      <button type="button" class="window-button pin" :aria-pressed="pinned" :title="`${pinned ? '取消置顶' : '置顶窗口'} (Command+D / Alt+D)`" aria-label="切换窗口置顶" @click="togglePinned">
        <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.6" aria-hidden="true"><path d="M9 3h6l-1 5 4 4v2h-5v7l-2-2v-5H6v-2l4-4z" /></svg>
      </button>
    </header>
    <DiffToolbar
      v-if="original"
      v-model:format="format" v-model:formatted="formatted" v-model:wrapping="wrapping" v-model:collapse="collapse"
      :importing="loading.left || loading.right" :navigable="!busy && count > 0"
      @compare="compare" @navigate="navigate" @swap="swap"
    />
    <div v-if="operationError" class="error-banner" role="alert">
      <span>{{ operationError }}</span><button type="button" aria-label="关闭错误提示" @click="operationError = ''">×</button>
    </div>
    <div v-if="original" class="source-grid">
      <DiffSourceHeader side="左侧" :name="left.name" :loading="loading.left" :can-copy="!!result" @import="importFile('left', $event)" @copy="copy('left')" @restore="restore('left')" />
      <DiffSourceHeader side="右侧" :name="right.name" :loading="loading.right" :can-copy="!!result" @import="importFile('right', $event)" @copy="copy('right')" @restore="restore('right')" />
    </div>
    <DiffMergeEditor v-if="result" ref="editor" :result="result" :wrapping="wrapping" :collapse="collapse" @stats="updateStats" @edit="edit" />
    <div v-else class="diff-placeholder" :role="error ? 'alert' : 'status'">
      <p>{{ error || (busy ? '正在本地计算差异…' : original ? '请选择对比操作' : '正在读取对比内容…') }}</p>
      <button v-if="error" type="button" @click="formatted = false; compare()">返回原文对比并编辑</button>
    </div>
    <footer class="diff-footer" role="status">
      <span v-if="busy">正在计算… <button type="button" @click="cancel">取消</button></span>
      <span v-else-if="result">{{ count ? `${count} 处差异 · 当前 ${current}/${count}` : hasLiveEdits ? '当前内容相同' : formatted ? '格式化后相同' : '原文完全相同' }}</span>
      <span v-else>{{ error ? '对比未完成' : '' }}</span>
      <span class="footer-status">{{ status }}</span>
      <span>{{ hasLiveEdits ? '编辑后对比' : formatted ? '格式化对比' : '原文对比' }} · {{ result?.format.toUpperCase() || '—' }}</span>
    </footer>
  </main>
</template>

<style scoped>
.diff-page { display: flex; flex-direction: column; height: 100%; min-height: 0; color: var(--app-text); background: var(--app-menu-background); }
.diff-header { display: flex; height: 36px; flex: 0 0 auto; align-items: center; border-bottom: 1px solid var(--app-border); padding: 0 8px; background: var(--app-surface); }
h1 { flex: 1; text-align: center; margin: 0; font-size: 14px; color: var(--app-heading); }
button { color: var(--app-text); background: transparent; border: 0; border-radius: 5px; padding: 4px 8px; font: inherit; cursor: pointer; }
button:hover { background: var(--app-active); }
.window-button { display: inline-flex; align-items: center; justify-content: center; width: 28px; height: 28px; font-size: 22px; }
.pin[aria-pressed="true"] { background: var(--app-primary-ring); color: var(--app-primary); }
.source-grid { display: grid; grid-template-columns: minmax(0, 1fr) minmax(0, 1fr); }
.source-grid > :last-child { border-left: 1px solid var(--app-border); }
.diff-merge-host, .diff-placeholder { flex: 1; min-height: 0; }
.diff-placeholder { overflow: auto; padding: 20px; text-align: center; color: var(--app-muted); white-space: pre-wrap; }
.error-banner { display: flex; gap: 8px; align-items: start; padding: 6px 10px; color: var(--app-danger); background: var(--app-danger-hover); font-size: 12px; max-height: 100px; overflow: auto; }
.error-banner span { flex: 1; white-space: pre-wrap; }
.diff-footer { flex: 0 0 auto; display: flex; align-items: center; gap: 12px; min-height: 28px; padding: 0 10px; border-top: 1px solid var(--app-border); font-size: 11px; color: var(--app-muted); }
.footer-status { flex: 1; text-align: right; color: var(--app-primary); }
</style>
