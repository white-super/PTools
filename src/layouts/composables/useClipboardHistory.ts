import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { onMounted, onUnmounted, shallowRef } from "vue";
import {
  onFilesUpdate,
  onImageUpdate,
  onTextUpdate,
  startListening,
  writeFiles,
  writeImageBase64,
  writeText,
} from "tauri-plugin-clipboard-api";
import type {
  AppSettings,
  ClipboardFormat,
  ClipboardHistoryEntry,
  ClipboardHistoryPage,
  ClipboardHistoryQuery,
  ClipboardTag,
  ClipboardTagInput,
  SystemPermissionStatus,
} from "../types/settings";
import type { QuickToolId } from "../features/quick-tools/types";

const HISTORY_UPDATED_EVENT = "clipboard-history-updated";
const MAIN_PANEL_FOCUS_EVENT = "ptools://main-panel-focus";
const SETTINGS_UPDATED_EVENT = "app-settings-updated";
const TAGS_UPDATED_EVENT = "clipboard-tags-updated";
const HISTORY_PAGE_SIZE = 20;

interface ClipboardHistoryInput {
  readonly format: ClipboardFormat;
  readonly content: string;
  readonly filePaths?: readonly string[];
}

function reportHistoryError(error: unknown) {
  console.error("Failed to synchronize clipboard history", error);
}

export function useClipboardHistory() {
  const cards = shallowRef<ReadonlyArray<ClipboardHistoryEntry>>([]);
  const tags = shallowRef<ReadonlyArray<ClipboardTag>>([]);
  const settings = shallowRef<AppSettings>();
  let unlistenTextUpdate: UnlistenFn | undefined;
  let unlistenImageUpdate: UnlistenFn | undefined;
  let unlistenFilesUpdate: UnlistenFn | undefined;
  let unlistenHistoryUpdate: UnlistenFn | undefined;
  let unlistenPanelFocus: UnlistenFn | undefined;
  let unlistenSettingsUpdate: UnlistenFn | undefined;
  let unlistenTagsUpdate: UnlistenFn | undefined;
  let stopListening: (() => Promise<void>) | undefined;
  let historyQueue: Promise<void> = Promise.resolve();
  const hasMore = shallowRef(false);
  const isLoading = shallowRef(false);
  const currentQuery = shallowRef<ClipboardHistoryQuery>({ search: "" });
  let requestSequence = 0;

  function updateCards(nextCards: ClipboardHistoryEntry[]) {
    cards.value = nextCards;
  }

  function updateTags(nextTags: ClipboardTag[]) {
    tags.value = nextTags;
  }

  function updateCardTags(id: number, tagIds: readonly number[]) {
    const nextTagIds = [...tagIds];
    const activeTagId = currentQuery.value.tagId;
    if (activeTagId !== undefined && !nextTagIds.includes(activeTagId)) {
      updateCards(cards.value.filter((card) => card.id !== id));
      return;
    }
    updateCards(
      cards.value.map((card) => (card.id === id ? { ...card, tagIds: nextTagIds } : card)),
    );
  }

  async function load() {
    const [nextSettings, nextTags] = await Promise.all([
      invoke<AppSettings>("get_app_settings"),
      invoke<ClipboardTag[]>("get_clipboard_tags"),
    ]);
    settings.value = nextSettings;
    updateTags(nextTags);
  }

  async function refreshSettings() {
    settings.value = await invoke<AppSettings>("get_app_settings");
  }

  async function fetchPage(query: ClipboardHistoryQuery, offset: number, replace: boolean) {
    const sequence = ++requestSequence;
    isLoading.value = true;
    try {
      const page = await invoke<ClipboardHistoryPage>("get_clipboard_history", {
        query: { ...query, offset, limit: HISTORY_PAGE_SIZE },
      });
      if (sequence !== requestSequence) {
        return;
      }
      updateCards(replace ? page.entries : [...cards.value, ...page.entries]);
      hasMore.value = page.hasMore;
    } finally {
      if (sequence === requestSequence) {
        isLoading.value = false;
      }
    }
  }

  async function reload(query: ClipboardHistoryQuery) {
    currentQuery.value = query;
    updateCards([]);
    hasMore.value = false;
    await fetchPage(query, 0, true);
  }

  async function loadMore() {
    if (isLoading.value || !hasMore.value) {
      return;
    }
    await fetchPage(currentQuery.value, cards.value.length, false);
  }

  async function record(entry: ClipboardHistoryInput) {
    await invoke("record_clipboard_history", { entry });
  }

  function enqueueHistoryRecord(entry: ClipboardHistoryInput) {
    historyQueue = historyQueue.then(() => record(entry)).catch(reportHistoryError);
  }

  function recordText(content: string) {
    enqueueHistoryRecord({ format: "text", content });
  }

  function recordImage(content: string) {
    enqueueHistoryRecord({ format: "image", content });
  }

  function recordFiles(filePaths: string[]) {
    enqueueHistoryRecord({ format: "file", content: filePaths.join("\n"), filePaths });
  }

  async function writeToClipboard(card: ClipboardHistoryEntry) {
    if (card.format === "text") {
      return writeText(card.content);
    }
    if (card.format === "image") {
      return writeImageBase64(card.content);
    }
    return writeFiles([...card.filePaths]);
  }

  async function hidePanel() {
    await invoke("hide_main_panel");
  }

  async function ensureAutomaticPastePermission() {
    let status = await invoke<SystemPermissionStatus>("get_system_permission_status");
    if (!status.automaticPasteSupported) {
      throw new Error(`${status.systemName} 暂不支持自动粘贴`);
    }
    if (!status.accessibilityPermissionSupported) return;
    if (!status.accessibilityPermissionGranted) {
      status = await invoke<SystemPermissionStatus>("request_system_permission");
    }
    if (!status.accessibilityPermissionGranted) {
      throw new Error("自动粘贴需要“辅助功能”权限。请先在设置中打开系统权限后重试。");
    }
  }

  async function pasteCard(card: ClipboardHistoryEntry) {
    if (settings.value?.autoPaste) {
      await ensureAutomaticPastePermission();
    }
    await hidePanel();
    try {
      await writeToClipboard(card);
      if (settings.value?.autoPaste) {
        await invoke("paste_into_active_app");
      }
    } finally {
      await hidePanel();
    }
  }

  async function deleteCard(id: number) {
    await invoke("delete_clipboard_history", { id });
  }

  async function updateQuickToolIds(quickToolIds: readonly QuickToolId[]) {
    settings.value = await invoke<AppSettings>("update_quick_tools", {
      quickToolIds: [...quickToolIds],
    });
  }

  async function createTag(input: ClipboardTagInput) {
    const nextTags = await invoke<ClipboardTag[]>("create_clipboard_tag", { input });
    updateTags(nextTags);
  }

  async function updateTag(id: number, input: ClipboardTagInput) {
    const nextTags = await invoke<ClipboardTag[]>("update_clipboard_tag", { id, input });
    updateTags(nextTags);
  }

  async function deleteTag(id: number) {
    const nextTags = await invoke<ClipboardTag[]>("delete_clipboard_tag", { id });
    updateTags(nextTags);
  }

  async function setCardTags(id: number, tagIds: readonly number[]) {
    await invoke("set_clipboard_history_tags", {
      id,
      tagIds,
    });
    updateCardTags(id, tagIds);
  }

  async function startMonitoring() {
    unlistenHistoryUpdate = await listen(HISTORY_UPDATED_EVENT, () => {
      requestSequence += 1;
      void reload(currentQuery.value).catch(reportHistoryError);
    });
    unlistenPanelFocus = await listen(MAIN_PANEL_FOCUS_EVENT, () => {
      void refreshSettings().catch(reportHistoryError);
    });
    unlistenSettingsUpdate = await listen<AppSettings>(SETTINGS_UPDATED_EVENT, (event) => {
      settings.value = event.payload;
    });
    unlistenTagsUpdate = await listen<ClipboardTag[]>(TAGS_UPDATED_EVENT, (event) => {
      updateTags(event.payload);
    });
    unlistenTextUpdate = await onTextUpdate(recordText);
    unlistenImageUpdate = await onImageUpdate(recordImage);
    unlistenFilesUpdate = await onFilesUpdate(recordFiles);
    stopListening = await startListening();
  }

  async function initialize() {
    await startMonitoring();
    await load();
  }

  function dispose() {
    unlistenTextUpdate?.();
    unlistenImageUpdate?.();
    unlistenFilesUpdate?.();
    unlistenHistoryUpdate?.();
    unlistenPanelFocus?.();
    unlistenSettingsUpdate?.();
    unlistenTagsUpdate?.();
    if (stopListening) {
      void stopListening().catch(reportHistoryError);
    }
  }

  onMounted(() => {
    void initialize().catch(reportHistoryError);
  });

  onUnmounted(dispose);

  return {
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
    updateQuickToolIds,
    updateTag,
  };
}
