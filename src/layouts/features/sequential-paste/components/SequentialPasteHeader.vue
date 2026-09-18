<script setup lang="ts">
import type { SequentialPasteMode } from "../types";

interface Props {
  readonly mode: SequentialPasteMode;
  readonly count: number;
  readonly busy: boolean;
}

defineProps<Props>();
const emit = defineEmits<{
  close: [];
  modeChange: [mode: SequentialPasteMode];
}>();
</script>

<template>
  <header class="queue-header" data-tauri-drag-region>
    <button
      type="button"
      class="icon-button close-button"
      aria-label="关闭顺序粘贴"
      title="关闭"
      @mousedown.stop
      @click="emit('close')"
    >
      <svg viewBox="0 0 20 20" aria-hidden="true"><path d="m6 6 8 8M14 6l-8 8" /></svg>
    </button>
    <div class="queue-title" data-tauri-drag-region>
      <svg viewBox="0 0 24 24" aria-hidden="true">
        <path d="M5 6.5h11M5 12h8M5 17.5h5M16 14l3 3-3 3M19 17h-6" />
      </svg>
      <strong data-tauri-drag-region>顺序粘贴</strong>
      <span>{{ count }}</span>
    </div>
    <div class="mode-switch" role="group" aria-label="队列模式">
      <button
        type="button"
        :class="{ active: mode === 'capture' }"
        :aria-pressed="mode === 'capture'"
        :disabled="busy"
        @click="emit('modeChange', 'capture')"
      >采集</button>
      <button
        type="button"
        :class="{ active: mode === 'paste' }"
        :aria-pressed="mode === 'paste'"
        :disabled="busy || count === 0"
        @click="emit('modeChange', 'paste')"
      >粘贴</button>
    </div>
  </header>
</template>

<style scoped>
.queue-header {
  display: grid;
  box-sizing: border-box;
  height: 42px;
  grid-template-columns: 30px minmax(0, 1fr) auto;
  align-items: center;
  gap: 5px;
  padding: 0 8px;
  border-bottom: 1px solid var(--app-border);
  background: var(--app-surface);
  user-select: none;
}

button { border: 0; color: inherit; font: inherit; cursor: pointer; }
.icon-button { display: inline-flex; width: 28px; height: 28px; align-items: center; justify-content: center; padding: 0; border-radius: 6px; background: transparent; }
.icon-button:hover { background: var(--app-active); }
.icon-button:focus-visible, .mode-switch button:focus-visible { outline: 2px solid var(--app-primary-ring); outline-offset: 1px; }
.icon-button svg { width: 17px; height: 17px; fill: none; stroke: currentColor; stroke-linecap: round; stroke-width: 1.7; }
.queue-title { display: flex; min-width: 0; align-items: center; gap: 7px; color: var(--app-heading); }
.queue-title > svg { width: 16px; height: 16px; flex: 0 0 auto; fill: none; stroke: var(--app-primary); stroke-linecap: round; stroke-linejoin: round; stroke-width: 1.6; }
.queue-title strong { overflow: hidden; font-size: 13px; font-weight: 650; text-overflow: ellipsis; white-space: nowrap; }
.queue-title span { min-width: 17px; padding: 1px 4px; border-radius: 8px; color: var(--app-muted); background: var(--app-active); font-size: 10px; text-align: center; }
.mode-switch { display: inline-flex; padding: 2px; border: 1px solid var(--app-border); border-radius: 7px; background: var(--app-surface-elevated); }
.mode-switch button { min-width: 42px; height: 23px; padding: 0 7px; border-radius: 5px; color: var(--app-muted); background: transparent; font-size: 11px; }
.mode-switch button.active { color: var(--app-on-primary); background: var(--app-primary); }
.mode-switch button:disabled { cursor: default; opacity: 0.45; }
</style>
