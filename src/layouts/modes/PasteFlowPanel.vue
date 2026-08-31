<script setup lang="ts">
import { computed, shallowRef, useTemplateRef } from "vue";
import { ElMessage, ElMessageBox } from "element-plus";
import CardContextMenu from "../components/CardContextMenu.vue";
import ClipboardFilterBar from "../components/ClipboardFilterBar.vue";
import ClipboardSearchInput from "../components/ClipboardSearchInput.vue";
import PasteCard from "../components/PasteCard/index.vue";
import { DEFAULT_APP_THEME } from "../constants/appThemes";
import { useClipboardHistory } from "../composables/useClipboardHistory";
import { useClipboardHistoryQuery } from "../composables/useClipboardHistoryQuery";
import { usePanelDismissal } from "../composables/usePanelDismissal";
import { usePanelEmptyState } from "../composables/usePanelEmptyState";
import { MAX_QUICK_SELECT_CARDS, usePasteFlowKeyboard } from "../composables/usePasteFlowKeyboard";
import { useShortcutHelpPanel } from "../composables/useShortcutHelpPanel";
import { useTextFormatterLauncher } from "../composables/useTextFormatterLauncher";
import type { ClipboardFilter, ClipboardHistoryEntry, ClipboardTagInput } from "../types/settings";
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
const FORMAT_FILTERS: readonly ClipboardFilter[] = ["text", "image", "file"];

const {
  cards,
  createTag,
  deleteCard,
  deleteTag,
  hasMore,
  isLoading,
  loadMore,
  pasteCard,
  reload,
  setCardTags,
  settings,
  tags,
  updateTag,
} = useClipboardHistory();
const { openTextFormatter } = useTextFormatterLauncher();
const isPasting = shallowRef(false);
const contextMenu = shallowRef<CardContextMenuState>();
const cardContainer = useTemplateRef<HTMLDivElement>("cardContainer");
const filterBar = useTemplateRef<InstanceType<typeof ClipboardFilterBar>>("filterBar");
const searchInput = useTemplateRef<InstanceType<typeof ClipboardSearchInput>>("searchInput");
const panelThemeClass = computed(() => `theme-${settings.value?.theme ?? DEFAULT_APP_THEME}`);
const { isHelpOpen, setHelpOpen } = useShortcutHelpPanel({
  reportError: (error) => reportPanelError("toggle shortcut help panel", error),
});
const {
  activeFilter,
  flushSearch,
  handleCardContainerScroll,
  searchQuery,
  scrollHorizontally,
  visibleCards,
} = useClipboardHistoryQuery({
  cards,
  hasMore,
  isLoading,
  loadMore,
  reload,
  reportError: (error) => reportPanelError("load clipboard history", error),
  resetScroll: () => cardContainer.value?.scrollTo({ left: 0, behavior: "auto" }),
});
const availableFilters = computed<readonly ClipboardFilter[]>(() => [
  "all",
  ...(settings.value?.showFormatFilters ? FORMAT_FILTERS : []),
  ...tags.value.map((tag) => tag.id),
]);
const { description: emptyStateDescription, title: emptyStateTitle } = usePanelEmptyState({
  activeFilter, isLoading, searchQuery, tags,
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
  filterBar.value?.closeMenus();
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
  closeCardContextMenu();
  try {
    await setCardTags(menu.cardId, assignedTagIds);
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

async function handleSearchSubmit() {
  try {
    await flushSearch();
  } catch (error) {
    reportPanelError("search clipboard history", error);
    return;
  }
  const card = visibleCards.value.find((entry) => entry.id === selectedCardId.value) ?? visibleCards.value[0];
  if (card) {
    await handlePaste(card);
  }
}

const { selectedCardId, selectCard } = usePasteFlowKeyboard({
  cards: visibleCards,
  cardNavigation: { shortcutSettings: settings },
  getCardContainer: () => cardContainer.value,
  closeMenus,
  filterNavigation: {
    activeFilter,
    filters: availableFilters,
    shortcutSettings: settings,
  },
  focusSearch: () => searchInput.value?.focus(),
  formatCard: openTextFormatter,
  pasteCard: (card) => void handlePaste(card),
  reportError: (error) => reportPanelError("listen for main panel focus", error),
});

usePanelDismissal({ beforeDismiss: () => { closeMenus(); searchQuery.value = ""; } });
</script>

<template>
  <div class="paste-flow-panel" :class="panelThemeClass" @click="closeMenus" @contextmenu.prevent>
    <div class="panel-toolbar">
      <ClipboardFilterBar
        ref="filterBar"
        class="panel-filters"
        :tags="tags"
        :active-filter="activeFilter"
        :help-open="isHelpOpen"
        :show-format-filters="settings?.showFormatFilters ?? false"
        @filter-change="activeFilter = $event"
        @create-tag="handleCreateTag"
        @update-tag="handleUpdateTag"
        @delete-tag="handleDeleteTag"
        @help-open-change="setHelpOpen"
      />
      <ClipboardSearchInput
        ref="searchInput"
        v-model="searchQuery"
        class="panel-search"
        @submit="handleSearchSubmit"
      />
    </div>
    <div
      v-if="visibleCards.length"
      ref="cardContainer"
      class="card-container"
      @scroll="handleCardContainerScroll"
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
