<script setup lang="ts">
import { computed, shallowRef, useTemplateRef } from "vue";
import { ElMessage, ElMessageBox } from "element-plus";
import CardContextMenu from "../components/CardContextMenu.vue";
import ClipboardFilterBar from "../components/ClipboardFilterBar.vue";
import PasteCard from "../components/PasteCard/index.vue";
import { useClipboardHistory } from "../composables/useClipboardHistory";
import { usePanelDismissal } from "../composables/usePanelDismissal";
import {
  MAX_QUICK_SELECT_CARDS,
  usePasteFlowKeyboard,
} from "../composables/usePasteFlowKeyboard";
import type {
  ClipboardFilter,
  ClipboardHistoryEntry,
  ClipboardTagInput,
} from "../types/settings";

interface CardContextMenuState {
  readonly cardId: number;
  readonly x: number;
  readonly y: number;
  readonly assignedTagIds: readonly number[];
}

const CONTEXT_MENU_WIDTH = 152;
const CONTEXT_MENU_MAX_HEIGHT = 264;
const CONTEXT_MENU_BASE_HEIGHT = 42;
const CONTEXT_MENU_MARGIN = 8;

const {
  cards,
  createTag,
  deleteCard,
  deleteTag,
  pasteCard,
  setCardTags,
  tags,
  updateTag,
} = useClipboardHistory();
const activeFilter = shallowRef<ClipboardFilter>("all");
const isPasting = shallowRef(false);
const contextMenu = shallowRef<CardContextMenuState>();
const cardContainer = useTemplateRef<HTMLDivElement>("cardContainer");
const filterBar = useTemplateRef<InstanceType<typeof ClipboardFilterBar>>("filterBar");

const visibleCards = computed(() => {
  const filter = activeFilter.value;
  if (filter === "all") {
    return cards.value;
  }
  if (typeof filter === "number") {
    return cards.value.filter((card) => card.tagIds.includes(filter));
  }
  return cards.value.filter((card) => card.format === filter);
});

const emptyStateTitle = computed(() => {
  if (cards.value.length === 0) {
    return "暂无历史内容";
  }
  if (typeof activeFilter.value === "number") {
    const tag = tags.value.find((entry) => entry.id === activeFilter.value);
    return `暂无“${tag?.name ?? "该标签"}”内容`;
  }
  return "当前格式暂无内容";
});

const emptyStateDescription = computed(() => {
  if (cards.value.length === 0) {
    return "复制文本、图片或文件后会自动显示在这里";
  }
  if (typeof activeFilter.value === "number") {
    return "右键卡片即可为它添加标签";
  }
  return "请选择其他格式查看历史内容";
});

function reportPanelError(action: string, error: unknown) {
  console.error(`Failed to ${action}`, error);
  const message = error instanceof Error ? error.message : String(error);
  ElMessage.error(message);
}

function closeCardContextMenu() {
  contextMenu.value = undefined;
}

function closeMenus() {
  closeCardContextMenu();
  filterBar.value?.closeTagContextMenu();
}

function openCardContextMenu(card: ClipboardHistoryEntry, event: MouseEvent) {
  selectedCardId.value = card.id;
  contextMenu.value = {
    cardId: card.id,
    x: Math.min(
      event.clientX,
      Math.max(CONTEXT_MENU_MARGIN, window.innerWidth - CONTEXT_MENU_WIDTH - CONTEXT_MENU_MARGIN),
    ),
    y: Math.min(
      event.clientY,
      Math.max(
        CONTEXT_MENU_MARGIN,
        window.innerHeight -
          (tags.value.length > 0 ? CONTEXT_MENU_MAX_HEIGHT : CONTEXT_MENU_BASE_HEIGHT) -
          CONTEXT_MENU_MARGIN,
      ),
    ),
    assignedTagIds: card.tagIds,
  };
}

function toggleTagId(tagIds: readonly number[], tagId: number) {
  return tagIds.includes(tagId)
    ? tagIds.filter((currentTagId) => currentTagId !== tagId)
    : [...tagIds, tagId];
}

async function handleToggleCardTag(tagId: number) {
  const menu = contextMenu.value;
  if (!menu) {
    return;
  }
  const assignedTagIds = toggleTagId(menu.assignedTagIds, tagId);
  try {
    await setCardTags(menu.cardId, assignedTagIds);
    contextMenu.value = { ...menu, assignedTagIds };
  } catch (error) {
    reportPanelError("update clipboard tags", error);
  }
}

async function handleDeleteCard() {
  const menu = contextMenu.value;
  closeCardContextMenu();
  if (!menu) {
    return;
  }
  try {
    await deleteCard(menu.cardId);
    if (selectedCardId.value === menu.cardId) {
      selectedCardId.value = undefined;
    }
  } catch (error) {
    reportPanelError("delete clipboard history", error);
  }
}

async function handleCreateTag(input: ClipboardTagInput) {
  try {
    await createTag(input);
  } catch (error) {
    reportPanelError("create clipboard tag", error);
  }
}

async function handleUpdateTag(id: number, input: ClipboardTagInput) {
  try {
    await updateTag(id, input);
  } catch (error) {
    reportPanelError("update clipboard tag", error);
  }
}

async function handleDeleteTag(id: number) {
  const tag = tags.value.find((entry) => entry.id === id);
  if (!tag) {
    return;
  }
  try {
    await ElMessageBox.confirm(
      `删除“${tag.name}”后，所有卡片上的该标签关联都会被移除，且无法恢复。`,
      "确认删除标签",
      {
        type: "warning",
        confirmButtonText: "删除",
        cancelButtonText: "取消",
      },
    );
  } catch {
    return;
  }
  try {
    await deleteTag(id);
    if (activeFilter.value === id) {
      activeFilter.value = "all";
    }
  } catch (error) {
    reportPanelError("delete clipboard tag", error);
  }
}

async function handlePaste(card: ClipboardHistoryEntry) {
  if (isPasting.value) {
    return;
  }
  isPasting.value = true;
  try {
    await pasteCard(card);
  } catch (error) {
    reportPanelError("paste selected card", error);
  } finally {
    isPasting.value = false;
  }
}

function scrollHorizontally(event: WheelEvent) {
  const container = event.currentTarget;
  if (!(container instanceof HTMLElement)) {
    return;
  }
  const scrollOffset = event.deltaX || event.deltaY;
  if (scrollOffset !== 0) {
    container.scrollLeft += scrollOffset;
  }
}

const { selectedCardId, selectCard } = usePasteFlowKeyboard({
  cards: visibleCards,
  getCardContainer: () => cardContainer.value,
  closeMenus,
  pasteCard: (card) => void handlePaste(card),
  reportError: (error) => reportPanelError("listen for main panel focus", error),
});

usePanelDismissal();
</script>

<template>
  <div class="paste-flow-panel" @click="closeMenus">
    <ClipboardFilterBar
      ref="filterBar"
      :tags="tags"
      :active-filter="activeFilter"
      @filter-change="activeFilter = $event"
      @create-tag="handleCreateTag"
      @update-tag="handleUpdateTag"
      @delete-tag="handleDeleteTag"
    />
    <div
      v-if="visibleCards.length"
      ref="cardContainer"
      class="card-container"
      @wheel.prevent="scrollHorizontally"
    >
      <PasteCard
        v-for="(card, index) in visibleCards"
        :key="card.id"
        :data-card-id="card.id"
        :content="card.content"
        :format="card.format"
        :file-paths="card.filePaths"
        :is-selected="selectedCardId === card.id"
        :quick-key="index < MAX_QUICK_SELECT_CARDS ? index + 1 : undefined"
        @select="selectCard(card.id)"
        @paste="handlePaste(card)"
        @context-menu="openCardContextMenu(card, $event)"
      />
    </div>
    <div v-else class="empty-state">
      <p class="empty-state-title">{{ emptyStateTitle }}</p>
      <p class="empty-state-description">{{ emptyStateDescription }}</p>
    </div>
    <CardContextMenu
      v-if="contextMenu"
      :x="contextMenu.x"
      :y="contextMenu.y"
      :tags="tags"
      :assigned-tag-ids="contextMenu.assignedTagIds"
      @toggle-tag="handleToggleCardTag"
      @delete="handleDeleteCard"
    />
  </div>
</template>

<style scoped src="./PasteFlowPanel.css"></style>
