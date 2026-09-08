<script setup lang="ts">
import { nextTick, onMounted, onUnmounted, useTemplateRef } from "vue";

const open = defineModel<boolean>({ required: true });
const emit = defineEmits<{ settings: [] }>();
const container = useTemplateRef<HTMLDivElement>("container");
const trigger = useTemplateRef<HTMLButtonElement>("trigger");
const settingsItem = useTemplateRef<HTMLButtonElement>("settingsItem");

function close() {
  open.value = false;
}

function handleOutsidePointer(event: PointerEvent) {
  if (event.target instanceof Node && !container.value?.contains(event.target)) close();
}

function handleKeydown(event: KeyboardEvent) {
  if (event.key === "Escape") {
    event.preventDefault();
    close();
    trigger.value?.focus();
  } else if (event.key === "ArrowDown" || event.key === "ArrowUp") {
    event.preventDefault();
    open.value = true;
    void nextTick(() => settingsItem.value?.focus());
  } else if (event.key === "Tab") {
    close();
  }
}

function openSettings() {
  close();
  emit("settings");
}

onMounted(() => document.addEventListener("pointerdown", handleOutsidePointer, true));
onUnmounted(() => document.removeEventListener("pointerdown", handleOutsidePointer, true));
</script>

<template>
  <div ref="container" class="more-menu" @click.stop @keydown.stop="handleKeydown">
    <button
      ref="trigger"
      type="button"
      class="more-trigger"
      tabindex="-1"
      aria-label="更多操作"
      title="更多操作"
      aria-haspopup="menu"
      aria-controls="clipboard-more-menu"
      :aria-expanded="open"
      @click="open = !open"
    >
      <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
        <circle cx="5" cy="12" r="1.8" />
        <circle cx="12" cy="12" r="1.8" />
        <circle cx="19" cy="12" r="1.8" />
      </svg>
    </button>
    <div v-if="open" id="clipboard-more-menu" class="more-dropdown" role="menu" aria-label="更多操作">
      <button ref="settingsItem" type="button" class="more-item" role="menuitem" @click="openSettings">
        设置
      </button>
    </div>
  </div>
</template>

<style scoped>
.more-menu {
  position: relative;
  justify-self: end;
}

.more-trigger {
  display: flex;
  width: 28px;
  height: 28px;
  align-items: center;
  justify-content: center;
  padding: 0;
  border: 0;
  border-radius: 6px;
  color: var(--panel-muted);
  background: transparent;
  cursor: pointer;
}

.more-trigger:hover,
.more-trigger[aria-expanded="true"] {
  color: var(--panel-text);
  background: var(--panel-control-active);
}

.more-dropdown {
  position: absolute;
  top: calc(100% + 6px);
  right: 0;
  z-index: 30;
  width: 144px;
  padding: 4px;
  border: 1px solid var(--app-border);
  border-radius: 8px;
  background: var(--app-menu-background);
  box-shadow: var(--app-shadow);
}

.more-item {
  width: 100%;
  padding: 8px 12px;
  border: 0;
  border-radius: 5px;
  color: var(--app-text);
  background: transparent;
  font: inherit;
  font-size: 13px;
  text-align: left;
  cursor: pointer;
}

.more-item:hover {
  color: var(--app-heading);
  background: var(--app-active);
}

.more-trigger:focus-visible,
.more-item:focus-visible {
  outline: 2px solid var(--app-primary-ring);
  outline-offset: -2px;
}
</style>
