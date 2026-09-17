<script setup lang="ts">
import { computed } from "vue";
import CardMenuIcon from "./CardMenuIcon.vue";
import type { TextFormat, TextFormatterOption } from "../features/text-formatter/types";
import type { ClipboardTag } from "../types/settings";
import {
  CARD_CONTEXT_MENU_MAX_HEIGHT_PX,
  CARD_CONTEXT_MENU_SUBMENU_OFFSET_PX,
  CARD_CONTEXT_MENU_VIEWPORT_MARGIN_PX,
  CARD_CONTEXT_MENU_WIDTH_PX,
} from "../utils/cardContextMenuLayout";

interface Props {
  x: number;
  y: number;
  tags: readonly ClipboardTag[];
  assignedTagIds: readonly number[];
  tools: readonly TextFormatterOption[];
  diffLabel: string;
  diffDisabled: boolean;
}

const props = defineProps<Props>();
const emit = defineEmits<{
  toggleTag: [tagId: number];
  useTool: [format: TextFormat];
  delete: [];
  diff: [];
}>();

const opensSubmenuLeft = computed(
  () => props.x
    + CARD_CONTEXT_MENU_WIDTH_PX * 2
    + CARD_CONTEXT_MENU_SUBMENU_OFFSET_PX
    + CARD_CONTEXT_MENU_VIEWPORT_MARGIN_PX > window.innerWidth,
);
const menuStyle = computed(() => ({
  "--context-menu-max-height": `${CARD_CONTEXT_MENU_MAX_HEIGHT_PX}px`,
  "--context-menu-submenu-offset": `${CARD_CONTEXT_MENU_SUBMENU_OFFSET_PX}px`,
  "--context-menu-width": `${CARD_CONTEXT_MENU_WIDTH_PX}px`,
  left: `${props.x}px`,
  top: `${props.y}px`,
}));
</script>

<template>
  <div class="context-menu" :style="menuStyle" role="menu" aria-label="卡片操作" @click.stop>
    <div class="context-menu-group" role="group" aria-label="卡片扩展操作">
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
          <CardMenuIcon name="tools" />
          <span class="context-menu-label">使用工具</span>
          <span class="context-menu-trailing" aria-hidden="true">
            <kbd v-if="props.tools.length > 0" class="context-menu-shortcut">F</kbd>
            <span class="context-menu-submenu-arrow">{{ opensSubmenuLeft ? "‹" : "›" }}</span>
          </span>
        </button>
        <div
          v-if="props.tools.length > 0"
          class="context-menu context-menu-submenu"
          :class="{ 'context-menu-submenu-left': opensSubmenuLeft }"
          role="menu"
          aria-label="选择工具"
        >
          <div class="context-menu-group" role="group">
            <button
              v-for="tool in props.tools"
              :key="tool.format"
              type="button"
              class="context-menu-item"
              role="menuitem"
              @click="emit('useTool', tool.format)"
            >
              <CardMenuIcon name="tools" />
              <span class="tool-name">{{ tool.title }}</span>
            </button>
          </div>
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
          <CardMenuIcon name="tag" />
          <span class="context-menu-label">标签</span>
          <span class="context-menu-trailing" aria-hidden="true">
            <span class="context-menu-submenu-arrow">{{ opensSubmenuLeft ? "‹" : "›" }}</span>
          </span>
        </button>
        <div
          v-if="props.tags.length > 0"
          class="context-menu context-menu-submenu"
          :class="{ 'context-menu-submenu-left': opensSubmenuLeft }"
          role="menu"
          aria-label="选择标签"
        >
          <div class="context-menu-group" role="group">
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
      </div>
    </div>

    <div class="context-menu-divider" role="separator"></div>
    <button
      type="button"
      class="context-menu-item"
      role="menuitem"
      :disabled="props.diffDisabled"
      @click="emit('diff')"
    >
      <CardMenuIcon name="diff" />
      <span class="context-menu-label">{{ props.diffLabel }}</span>
      <span class="context-menu-trailing" aria-hidden="true">
        <kbd class="context-menu-shortcut">D</kbd>
      </span>
    </button>

    <div class="context-menu-divider" role="separator"></div>
    <button
      type="button"
      class="context-menu-item context-menu-item-danger"
      role="menuitem"
      @click="emit('delete')"
    >
      <CardMenuIcon name="trash" />
      <span class="context-menu-label">删除</span>
    </button>
  </div>
</template>

<style scoped src="./CardContextMenu.css"></style>
