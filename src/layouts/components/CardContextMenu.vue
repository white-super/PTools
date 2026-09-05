<script setup lang="ts">
import { computed } from "vue";
import type { TextFormat, TextFormatterOption } from "../features/text-formatter/types";
import type { ClipboardTag } from "../types/settings";

const CONTEXT_MENU_WIDTH_PX = 152;
const VIEWPORT_MARGIN_PX = 8;

interface Props {
  x: number;
  y: number;
  tags: readonly ClipboardTag[];
  assignedTagIds: readonly number[];
  tools: readonly TextFormatterOption[];
}

const props = defineProps<Props>();
const emit = defineEmits<{
  toggleTag: [tagId: number];
  useTool: [format: TextFormat];
  delete: [];
}>();

const opensSubmenuLeft = computed(
  () => props.x + CONTEXT_MENU_WIDTH_PX * 2 + VIEWPORT_MARGIN_PX > window.innerWidth,
);
</script>

<template>
  <div
    class="context-menu"
    :style="{ left: `${props.x}px`, top: `${props.y}px` }"
    role="menu"
    @click.stop
  >
    <div
      class="context-menu-submenu-container"
      :class="{ 'context-menu-submenu-container-left': opensSubmenuLeft }"
    >
      <button
        type="button"
        class="context-menu-item context-menu-submenu-trigger"
        role="menuitem"
        aria-haspopup="menu"
        :disabled="props.tools.length === 0"
      >
        <span>使用工具</span>
        <span class="context-menu-submenu-arrow">{{ opensSubmenuLeft ? "‹" : "›" }}</span>
      </button>
      <div
        v-if="props.tools.length > 0"
        class="context-menu context-menu-submenu"
        :class="{ 'context-menu-submenu-left': opensSubmenuLeft }"
        role="menu"
        aria-label="选择工具"
      >
        <button
          v-for="tool in props.tools"
          :key="tool.format"
          type="button"
          class="context-menu-item"
          role="menuitem"
          @click="emit('useTool', tool.format)"
        >
          <span class="tool-name">{{ tool.title }}</span>
        </button>
      </div>
    </div>
    <div
      class="context-menu-submenu-container"
      :class="{ 'context-menu-submenu-container-left': opensSubmenuLeft }"
    >
      <button
        type="button"
        class="context-menu-item context-menu-submenu-trigger"
        role="menuitem"
        aria-haspopup="menu"
        :disabled="props.tags.length === 0"
      >
        <span>标签</span>
        <span class="context-menu-submenu-arrow">{{ opensSubmenuLeft ? "‹" : "›" }}</span>
      </button>
      <div
        v-if="props.tags.length > 0"
        class="context-menu context-menu-submenu"
        :class="{ 'context-menu-submenu-left': opensSubmenuLeft }"
        role="menu"
        aria-label="选择标签"
      >
        <button
          v-for="tag in props.tags"
          :key="tag.id"
          type="button"
          class="context-menu-item"
          :class="{ 'context-menu-item-selected': props.assignedTagIds.includes(tag.id) }"
          role="menuitemcheckbox"
          :aria-checked="props.assignedTagIds.includes(tag.id)"
          @click="emit('toggleTag', tag.id)"
        >
          <span class="tag-color" :style="{ backgroundColor: tag.color }"></span>
          <span class="tag-name">{{ tag.name }}</span>
          <span class="tag-selection">{{ props.assignedTagIds.includes(tag.id) ? "✓" : "" }}</span>
        </button>
      </div>
    </div>
    <div class="context-menu-divider"></div>
    <button
      type="button"
      class="context-menu-item context-menu-item-danger"
      role="menuitem"
      @click="emit('delete')"
    >
      删除
    </button>
  </div>
</template>

<style scoped>
.context-menu {
  --context-menu-edge-offset: 5px;
  --context-menu-submenu-gap: 1px;
  position: fixed;
  z-index: 20;
  box-sizing: border-box;
  width: 152px;
  padding: 4px;
  border: 1px solid var(--app-border);
  border-radius: 8px;
  background: var(--app-menu-background);
  box-shadow: var(--app-shadow);
}

.context-menu-item {
  display: flex;
  width: 100%;
  align-items: center;
  gap: 7px;
  border: 0;
  border-radius: 5px;
  padding: 7px 9px;
  color: var(--app-text);
  background: transparent;
  font: inherit;
  font-size: 12px;
  text-align: left;
  cursor: pointer;
  transition: background-color 0.12s ease, color 0.12s ease, transform 0.12s ease;
}

.context-menu-item:hover:not(:disabled) {
  color: var(--app-heading);
  background: var(--app-active);
  transform: translateX(2px);
}

.context-menu-item:active:not(:disabled) {
  background: var(--app-primary-ring);
  transform: translateX(2px) scale(0.98);
}

.context-menu-item:focus-visible {
  outline: 2px solid var(--app-primary-ring);
  outline-offset: -2px;
}

.context-menu-item:disabled {
  color: var(--app-muted);
  cursor: default;
  opacity: 0.65;
}

.context-menu-item-selected {
  background: var(--app-active);
}

.context-menu-submenu-container {
  position: relative;
}

.context-menu-submenu-container::after {
  position: absolute;
  top: 0;
  bottom: 0;
  left: 100%;
  width: calc(var(--context-menu-edge-offset) + var(--context-menu-submenu-gap));
  content: "";
}

.context-menu-submenu-container-left::after {
  right: 100%;
  left: auto;
}

.context-menu-submenu-trigger {
  justify-content: space-between;
}

.context-menu-submenu-arrow {
  margin-left: auto;
  color: var(--app-muted);
  font-size: 16px;
  line-height: 12px;
  transition: color 0.12s ease, transform 0.12s ease;
}

.context-menu-submenu {
  position: absolute;
  top: -5px;
  left: calc(100% + var(--context-menu-edge-offset) + var(--context-menu-submenu-gap));
  max-height: min(264px, calc(100vh - 16px));
  overflow-y: auto;
  visibility: hidden;
  opacity: 0;
  pointer-events: none;
  transform: translateX(-4px);
  transition: opacity 0.12s ease, transform 0.12s ease, visibility 0.12s;
}

.context-menu-submenu-left {
  right: calc(100% + var(--context-menu-edge-offset) + var(--context-menu-submenu-gap));
  left: auto;
  transform: translateX(4px);
}

.context-menu-submenu-container:hover > .context-menu-submenu-trigger,
.context-menu-submenu-container:focus-within > .context-menu-submenu-trigger {
  color: var(--app-heading);
  background: var(--app-active);
  transform: translateX(2px);
}

.context-menu-submenu-container:hover > .context-menu-submenu-trigger .context-menu-submenu-arrow,
.context-menu-submenu-container:focus-within > .context-menu-submenu-trigger .context-menu-submenu-arrow {
  color: var(--app-primary);
  transform: translateX(2px);
}

.context-menu-submenu-container-left:hover > .context-menu-submenu-trigger .context-menu-submenu-arrow,
.context-menu-submenu-container-left:focus-within > .context-menu-submenu-trigger .context-menu-submenu-arrow {
  transform: translateX(-2px);
}

.context-menu-submenu-container:hover > .context-menu-submenu,
.context-menu-submenu-container:focus-within > .context-menu-submenu {
  visibility: visible;
  opacity: 1;
  pointer-events: auto;
  transform: translateX(0);
}

.context-menu-divider {
  height: 1px;
  margin: 4px;
  background: var(--app-border);
}

.tag-color {
  width: 8px;
  height: 8px;
  flex: 0 0 auto;
  border-radius: 999px;
}

.tool-name,
.tag-name {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.tag-selection {
  min-width: 10px;
  margin-left: auto;
  color: var(--app-primary);
  text-align: right;
}

.context-menu-item-danger {
  color: var(--app-danger);
}

.context-menu-item-danger:hover:not(:disabled) {
  color: var(--app-danger);
  background: var(--app-danger-hover);
}
</style>
