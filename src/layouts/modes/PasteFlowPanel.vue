<script setup lang="ts">
import { computed, shallowRef, useTemplateRef } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { ElMessageBox } from "element-plus";
import CardContextMenu from "../components/CardContextMenu.vue";
import CardNotice from "../components/PasteCard/CardNotice.vue";
import ClipboardFilterBar from "../components/ClipboardFilterBar.vue";
import ClipboardSearchInput from "../components/ClipboardSearchInput.vue";
import ClipboardToolbarActions from "../components/ClipboardToolbarActions.vue";
import ClipboardCardList from "../components/ClipboardCardList.vue";
import { useTextDiffLauncher } from "../composables/useTextDiffLauncher";
import { DEFAULT_APP_THEME } from "../constants/appThemes";
import { useClipboardHistory } from "../composables/useClipboardHistory";
import { useClipboardHistoryQuery } from "../composables/useClipboardHistoryQuery";
import { useCardContextMenu } from "../composables/useCardContextMenu";
import { usePanelNotice } from "../composables/usePanelNotice";
import { usePanelDismissal } from "../composables/usePanelDismissal";
import { usePanelEmptyState } from "../composables/usePanelEmptyState";
import { usePasteFlowKeyboard } from "../composables/usePasteFlowKeyboard";
import { useShortcutHelpPanel } from "../composables/useShortcutHelpPanel";
import { useTextFormatterLauncher } from "../composables/useTextFormatterLauncher";
import type { TextFormat } from "../features/text-formatter/types";
import type { ClipboardFilter, ClipboardHistoryEntry, ClipboardTagInput } from "../types/settings";
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
const { notice: panelNotice, reportError: reportPanelError, clearNotice } = usePanelNotice();
const { openTextFormatter, openTextFormatterWithFormat } = useTextFormatterLauncher(
  (error, card) => reportPanelError("open text formatter", error, card.id),
);
const { pending: diffPending, busy: diffBusy, select: selectDiff, cancel: cancelDiff, openEmptyTextDiff } = useTextDiffLauncher(
  (error, card) => reportPanelError(card ? "select diff source" : "open empty text diff", error, card?.id),
);
const isPasting = shallowRef(false);
const isMoreMenuOpen = shallowRef(false);
const cardList = useTemplateRef<InstanceType<typeof ClipboardCardList>>("cardList");
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
  visibleCards,
} = useClipboardHistoryQuery({
  cards,
  hasMore,
  isLoading,
  loadMore,
  reload,
  reportError: (error) => reportPanelError("load clipboard history", error),
  resetScroll: () => cardList.value?.element?.scrollTo({ left: 0, behavior: "auto" }),
  getCardContainer: () => cardList.value?.element ?? null,
});
const availableFilters = computed<readonly ClipboardFilter[]>(() => [
  "all",
  ...(settings.value?.showFormatFilters ? FORMAT_FILTERS : []),
  ...tags.value.map((tag) => tag.id),
]);
const { description: emptyStateDescription, title: emptyStateTitle } = usePanelEmptyState({
  activeFilter, isLoading, searchQuery, tags,
});
const { closeCardContextMenu, contextMenu, openCardContextMenu } = useCardContextMenu({ tags });
function closeMenus() {
  isMoreMenuOpen.value = false;
  closeCardContextMenu();
  filterBar.value?.closeMenus();
}

async function openSettings() {
  closeMenus();
  try {
    await invoke("show_settings_window");
    searchQuery.value = "";
  } catch (error) {
    reportPanelError("open settings", error);
  }
}

function handleCardContextMenu(card: ClipboardHistoryEntry, event: MouseEvent) {
  selectedCardId.value = card.id;
  openCardContextMenu(card, event);
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
    reportPanelError("update clipboard tags", error, menu.cardId);
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
    reportPanelError("delete clipboard history", error, menu.cardId);
  }
}

function handleUseCardTool(format: TextFormat) {
  const menu = contextMenu.value;
  closeCardContextMenu();
  if (menu) {
    openTextFormatterWithFormat(menu.card, format);
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
    reportPanelError("paste selected card", error, card.id);
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
  getCardContainer: () => cardList.value?.element ?? null,
  scrollToCard: (id) => cardList.value?.scrollToCard(id),
  closeMenus,
  filterNavigation: {
    activeFilter,
    filters: availableFilters,
    shortcutSettings: settings,
  },
  focusSearch: () => searchInput.value?.focus(),
  formatCard: openTextFormatter,
  diffCard: (card) => { void selectDiff(card); },
  pasteCard: (card) => void handlePaste(card),
  reportError: (error) => reportPanelError("listen for main panel focus", error),
});

usePanelDismissal({ beforeDismiss: () => { closeMenus(); cancelDiff(); clearNotice(); searchQuery.value = ""; } });
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
      <ClipboardToolbarActions v-model="isMoreMenuOpen" @text-diff="cancelDiff(); openEmptyTextDiff()" @settings="openSettings" />
    </div>
    <ClipboardCardList
      v-if="visibleCards.length"
      ref="cardList"
      :cards="visibleCards"
      :selected-card-id="selectedCardId"
      :diff-left-card-id="diffPending?.cardId"
      :notice="panelNotice"
      :aria-busy="diffBusy"
      @cancel-diff="cancelDiff"
      @select="selectCard"
      @paste="handlePaste"
      @context-menu="handleCardContextMenu"
      @scroll="handleCardContainerScroll"
    />
    <div v-else class="empty-state">
      <p class="empty-state-title">{{ emptyStateTitle }}</p>
      <p class="empty-state-description">{{ emptyStateDescription }}</p>
    </div>
    <div v-if="panelNotice && panelNotice.cardId === undefined" class="panel-notice-host">
      <CardNotice :notice="panelNotice" />
    </div>
    <CardContextMenu
      v-if="contextMenu"
      :x="contextMenu.x"
      :y="contextMenu.y"
      :tags="tags"
      :assigned-tag-ids="contextMenu.assignedTagIds"
      :tools="contextMenu.tools"
      :diff-label="diffPending?.cardId === contextMenu.cardId ? '取消文本对比' : diffPending ? '与已选内容对比' : '文本对比'"
      :diff-disabled="diffBusy || contextMenu.card.format === 'image'"
      @diff="selectDiff(contextMenu.card); closeCardContextMenu()"
      @toggle-tag="handleToggleCardTag"
      @use-tool="handleUseCardTool"
      @delete="handleDeleteCard"
    />
  </div>
</template>

<style scoped src="./PasteFlowPanel.css"></style>
