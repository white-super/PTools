<script setup lang="ts">
import { getCurrentWindow } from "@tauri-apps/api/window";
import { ElMessageBox } from "element-plus";
import { computed, onMounted, onUnmounted } from "vue";
import SequentialPasteHeader from "./components/SequentialPasteHeader.vue";
import SequentialPasteQueue from "./components/SequentialPasteQueue.vue";
import { useSequentialPaste } from "./composables/useSequentialPaste";
import type { SequentialPasteMode } from "./types";

const {
  busy,
  clear,
  close,
  initialized,
  nextItemId,
  operationError,
  remove,
  reorder,
  setDirection,
  setMode,
  shortcut,
  state,
} = useSequentialPaste();
const editable = computed(() => state.value.mode === "capture" && !busy.value);
const globalError = computed(() => operationError.value || (
  state.value.error?.itemId === undefined ? state.value.error?.message : ""
));
let unlistenClose: (() => void) | undefined;
let closing = false;

function confirmationCancelled(error: unknown) {
  return error === "cancel" || error === "close";
}

async function confirmAction(message: string, title: string, confirmButtonText: string) {
  try {
    await ElMessageBox.confirm(message, title, {
      confirmButtonText,
      cancelButtonText: "取消",
      type: "warning",
    });
    return true;
  } catch (error) {
    if (!confirmationCancelled(error)) operationError.value = String(error);
    return false;
  }
}

async function requestClose() {
  if (closing || busy.value) return;
  closing = true;
  try {
    if (state.value.items.length > 0 && !await confirmAction(
      "关闭后将清空当前临时队列。",
      "关闭顺序粘贴",
      "关闭并清空",
    )) return;
    await close();
  } finally {
    closing = false;
  }
}

async function requestClear() {
  if (state.value.items.length === 0 || busy.value) return;
  if (await confirmAction("当前队列内容将被全部移除。", "清空队列", "清空")) {
    await clear();
  }
}

function changeMode(mode: SequentialPasteMode) {
  if (mode !== state.value.mode) void setMode(mode);
}

function handleKeydown(event: KeyboardEvent) {
  if (event.isComposing || event.defaultPrevented) return;
  if (event.target instanceof Element && event.target.closest(".el-overlay")) return;
  const key = event.key.toLowerCase();
  if (key === "escape" || (key === "w" && (event.metaKey || event.ctrlKey))) {
    event.preventDefault();
    void requestClose();
  }
}

onMounted(() => {
  window.addEventListener("keydown", handleKeydown, true);
  void getCurrentWindow().onCloseRequested((event) => {
    event.preventDefault();
    void requestClose();
  }).then((unlisten) => { unlistenClose = unlisten; });
});

onUnmounted(() => {
  window.removeEventListener("keydown", handleKeydown, true);
  unlistenClose?.();
});
</script>

<template>
  <main class="sequential-paste-page" @contextmenu.prevent>
    <SequentialPasteHeader
      :mode="state.mode"
      :count="state.items.length"
      :busy="busy"
      @close="requestClose"
      @mode-change="changeMode"
    />
    <div class="queue-controls">
      <div v-if="state.mode === 'paste'" class="direction-switch" role="group" aria-label="粘贴顺序">
        <button
          type="button"
          :class="{ active: state.direction === 'forward' }"
          :aria-pressed="state.direction === 'forward'"
          :disabled="busy"
          @click="setDirection('forward')"
        >正序</button>
        <button
          type="button"
          :class="{ active: state.direction === 'reverse' }"
          :aria-pressed="state.direction === 'reverse'"
          :disabled="busy"
          @click="setDirection('reverse')"
        >倒序</button>
      </div>
      <span v-else class="capture-status"><i></i>正在采集</span>
      <button
        v-if="state.mode === 'capture' && state.items.length"
        type="button"
        class="clear-button"
        :disabled="busy"
        @click="requestClear"
      >清空</button>
    </div>
    <div v-if="globalError" class="error-banner" role="alert">
      <span>{{ globalError }}</span>
    </div>
    <section class="queue-scroll" :aria-busy="busy">
      <div v-if="!initialized" class="loading-state">正在读取队列…</div>
      <SequentialPasteQueue
        v-else
        :items="state.items"
        :editable="editable"
        :next-item-id="nextItemId"
        :error="state.error"
        @reorder="reorder"
        @remove="remove"
      />
    </section>
    <footer class="queue-footer">
      <span>{{ state.items.length }} 项</span>
      <span v-if="state.mode === 'paste' && state.items.length" class="shortcut-label">
        <kbd>{{ shortcut }}</kbd>
      </span>
      <span v-else>{{ state.mode === "capture" ? "临时队列" : "已完成" }}</span>
    </footer>
  </main>
</template>

<style scoped>
.sequential-paste-page { display: flex; width: 100%; height: 100%; min-height: 0; flex-direction: column; color: var(--app-text); background: var(--app-background); }
.queue-controls { display: flex; min-height: 38px; flex: 0 0 38px; align-items: center; justify-content: space-between; padding: 0 10px; border-bottom: 1px solid var(--app-border); background: var(--app-surface-elevated); }
.direction-switch { display: inline-flex; gap: 2px; padding: 2px; border-radius: 7px; background: var(--app-active); }
.direction-switch button, .clear-button { border: 0; color: inherit; background: transparent; font: inherit; cursor: pointer; }
.direction-switch button { min-width: 48px; height: 24px; padding: 0 9px; border-radius: 5px; color: var(--app-muted); font-size: 11px; }
.direction-switch button.active { color: var(--app-heading); background: var(--app-surface-elevated); }
.direction-switch button:disabled, .clear-button:disabled { cursor: default; opacity: 0.5; }
.capture-status { display: inline-flex; align-items: center; gap: 7px; color: var(--app-secondary); font-size: 11px; }
.capture-status i { width: 7px; height: 7px; border-radius: 50%; background: var(--app-success); box-shadow: 0 0 0 3px color-mix(in srgb, var(--app-success) 16%, transparent); }
.clear-button { padding: 4px 6px; border-radius: 5px; color: var(--app-danger); font-size: 11px; }
.clear-button:hover { background: var(--app-danger-hover); }
.direction-switch button:focus-visible, .clear-button:focus-visible { outline: 2px solid var(--app-primary-ring); outline-offset: 1px; }
.error-banner { display: flex; flex: 0 0 auto; align-items: flex-start; gap: 6px; padding: 6px 9px; color: var(--app-danger); background: var(--app-danger-hover); font-size: 11px; line-height: 1.4; }
.error-banner span { flex: 1; overflow-wrap: anywhere; }
.queue-scroll { flex: 1; min-height: 0; overflow-x: hidden; overflow-y: auto; background: color-mix(in srgb, var(--app-background) 88%, var(--app-surface)); scrollbar-gutter: stable; }
.loading-state { display: grid; height: 100%; place-items: center; color: var(--app-muted); font-size: 12px; }
.queue-footer { display: flex; min-height: 30px; flex: 0 0 30px; align-items: center; justify-content: space-between; padding: 0 10px; border-top: 1px solid var(--app-border); color: var(--app-muted); background: var(--app-surface); font-size: 10px; }
.shortcut-label kbd { padding: 2px 5px; border: 1px solid var(--app-border); border-radius: 4px; color: var(--app-secondary); background: var(--app-surface-elevated); font: 10px ui-monospace, monospace; }
</style>
