import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { computed, onMounted, onUnmounted, readonly, shallowRef, type Ref } from "vue";
import type { AppSettings, ClipboardHistoryEntry } from "../../types/settings";
import { DEFAULT_QUICK_TOOL_IDS, getQuickToolDefinition } from "./quickToolRegistry";
import { validateQuickToolOrder } from "./quickToolOrder";
import { DEFAULT_QUICK_TOOL_SHORTCUTS } from "./quickToolShortcut";
import type { QuickToolId } from "./types";

const MAIN_PANEL_FOCUS_EVENT = "ptools://main-panel-focus";
const MAIN_PANEL_QUICK_TOOL_SHORTCUT_EVENT = "ptools://main-panel-quick-tool-shortcut";

interface UseQuickToolsOptions {
  readonly cards: Readonly<Ref<readonly ClipboardHistoryEntry[]>>;
  readonly selectedCardId: Readonly<Ref<number | undefined>>;
  readonly settings: Readonly<Ref<AppSettings | undefined>>;
  readonly saveToolIds: (toolIds: readonly QuickToolId[]) => Promise<void>;
  readonly closeMenus: () => void;
  readonly openTextDiff: () => void;
  readonly openTextFormatter: (card: ClipboardHistoryEntry, format: Exclude<QuickToolId, "text-diff">) => void;
  readonly reportError: (error: unknown, cardId?: number) => void;
}

export function useQuickTools(options: UseQuickToolsOptions) {
  const activeToolIds = shallowRef<readonly QuickToolId[]>([]);
  const isSaving = shallowRef(false);
  const toolIds = computed<readonly QuickToolId[]>(
    () => options.settings.value?.quickToolIds ?? DEFAULT_QUICK_TOOL_IDS,
  );
  const toolShortcuts = computed<readonly string[]>(
    () => options.settings.value?.quickToolShortcuts ?? DEFAULT_QUICK_TOOL_SHORTCUTS,
  );
  let disposed = false;
  let unlistenPanelFocus: UnlistenFn | undefined;
  let unlistenQuickToolShortcut: UnlistenFn | undefined;

  async function refreshActiveTools() {
    activeToolIds.value = await invoke<QuickToolId[]>("get_active_quick_tools");
  }

  async function saveToolOrder(nextToolIds: readonly QuickToolId[]) {
    validateQuickToolOrder(nextToolIds);
    if (isSaving.value) {
      options.reportError(new Error("快捷工具顺序正在保存，请稍候"));
      return;
    }
    isSaving.value = true;
    try {
      await options.saveToolIds(nextToolIds);
    } catch (error) {
      options.reportError(error);
    } finally {
      isSaving.value = false;
    }
  }

  function selectedCard() {
    return options.cards.value.find((card) => card.id === options.selectedCardId.value)
      ?? options.cards.value[0];
  }

  function executeTool(toolId: QuickToolId) {
    const tool = getQuickToolDefinition(toolId);
    options.closeMenus();
    if (tool.launch.kind === "text-diff") {
      options.openTextDiff();
      return;
    }
    const card = selectedCard();
    if (!card) {
      options.reportError(new Error(`请先选择文本内容，再使用${tool.label}`));
      return;
    }
    if (card.format !== "text") {
      options.reportError(new Error(`${tool.label}仅支持文本内容`), card.id);
      return;
    }
    options.openTextFormatter(card, tool.launch.format);
  }

  function executeToolAtIndex(index: unknown) {
    if (typeof index !== "number" || !Number.isInteger(index)) {
      return;
    }
    const toolId = toolIds.value[index];
    if (toolId) {
      executeTool(toolId);
    }
  }

  onMounted(() => {
    void refreshActiveTools().catch(options.reportError);
    void listen(MAIN_PANEL_FOCUS_EVENT, () => {
      void refreshActiveTools().catch(options.reportError);
    }).then((unlisten) => {
      if (disposed) {
        unlisten();
        return;
      }
      unlistenPanelFocus = unlisten;
    }).catch(options.reportError);
    void listen<number>(MAIN_PANEL_QUICK_TOOL_SHORTCUT_EVENT, ({ payload }) => {
      executeToolAtIndex(payload);
    }).then((unlisten) => {
      if (disposed) {
        unlisten();
        return;
      }
      unlistenQuickToolShortcut = unlisten;
    }).catch(options.reportError);
  });

  onUnmounted(() => {
    disposed = true;
    unlistenPanelFocus?.();
    unlistenQuickToolShortcut?.();
  });

  return {
    activeToolIds: readonly(activeToolIds),
    executeTool,
    isSaving: readonly(isSaving),
    saveToolOrder,
    toolIds,
    toolShortcuts,
  };
}
