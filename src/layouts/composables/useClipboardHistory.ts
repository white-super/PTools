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
  ClipboardTag,
  ClipboardTagInput,
  SystemPermissionStatus,
} from "../types/settings";

const HISTORY_UPDATED_EVENT = "clipboard-history-updated";
const SETTINGS_UPDATED_EVENT = "app-settings-updated";
const TAGS_UPDATED_EVENT = "clipboard-tags-updated";

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
  let unlistenSettingsUpdate: UnlistenFn | undefined;
  let unlistenTagsUpdate: UnlistenFn | undefined;
  let stopListening: (() => Promise<void>) | undefined;
  let historyQueue: Promise<void> = Promise.resolve();

  function updateCards(nextCards: ClipboardHistoryEntry[]) {
    cards.value = nextCards;
  }

  function updateTags(nextTags: ClipboardTag[]) {
    tags.value = nextTags;
  }

  async function load() {
    const [nextCards, nextSettings, nextTags] = await Promise.all([
      invoke<ClipboardHistoryEntry[]>("get_clipboard_history"),
      invoke<AppSettings>("get_app_settings"),
      invoke<ClipboardTag[]>("get_clipboard_tags"),
    ]);
    updateCards(nextCards);
    settings.value = nextSettings;
    updateTags(nextTags);
  }

  async function record(entry: ClipboardHistoryInput) {
    const nextCards = await invoke<ClipboardHistoryEntry[]>("record_clipboard_history", { entry });
    updateCards(nextCards);
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
    if (!status.accessibilityPermissionGranted) {
      status = await invoke<SystemPermissionStatus>("request_system_permission");
    }
    if (!status.accessibilityPermissionSupported) {
      throw new Error(`${status.systemName} 暂不支持自动粘贴`);
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
    const nextCards = await invoke<ClipboardHistoryEntry[]>("delete_clipboard_history", { id });
    updateCards(nextCards);
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
    const nextCards = await invoke<ClipboardHistoryEntry[]>("set_clipboard_history_tags", {
      id,
      tagIds,
    });
    updateCards(nextCards);
  }

  async function startMonitoring() {
    unlistenHistoryUpdate = await listen<ClipboardHistoryEntry[]>(HISTORY_UPDATED_EVENT, (event) => {
      updateCards(event.payload);
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
    await load();
    await startMonitoring();
  }

  function dispose() {
    unlistenTextUpdate?.();
    unlistenImageUpdate?.();
    unlistenFilesUpdate?.();
    unlistenHistoryUpdate?.();
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
    pasteCard,
    setCardTags,
    tags,
    updateTag,
  };
}
