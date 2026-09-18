import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import {
  computed,
  onMounted,
  onUnmounted,
  readonly,
  shallowRef,
  type ShallowRef,
} from "vue";
import type { AppSettings } from "../../../types/settings";
import type {
  SequentialPasteDirection,
  SequentialPasteMode,
  SequentialPasteSnapshot,
} from "../types";

const STATE_CHANGED_EVENT = "sequential-paste-state-changed";
const SETTINGS_UPDATED_EVENT = "app-settings-updated";
const EMPTY_STATE: SequentialPasteSnapshot = {
  enabled: false,
  mode: "capture",
  direction: "forward",
  items: [],
};

function errorMessage(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}

interface SequentialPasteContext {
  readonly state: ShallowRef<SequentialPasteSnapshot>;
  readonly shortcut: ShallowRef<string>;
  readonly busy: ShallowRef<boolean>;
  readonly operationError: ShallowRef<string>;
  readonly initialized: ShallowRef<boolean>;
  readonly eventRevision: ShallowRef<number>;
}

function createCommandActions(context: SequentialPasteContext) {
  async function execute(command: string, args?: Record<string, unknown>) {
    if (context.busy.value) return;
    context.busy.value = true;
    context.operationError.value = "";
    const revision = context.eventRevision.value;
    try {
      const snapshot = await invoke<SequentialPasteSnapshot>(command, args);
      if (context.eventRevision.value === revision) context.state.value = snapshot;
    } catch (error) {
      context.operationError.value = errorMessage(error);
    } finally {
      context.busy.value = false;
    }
  }

  async function close() {
    if (context.busy.value) return;
    context.busy.value = true;
    context.operationError.value = "";
    try {
      await invoke("close_sequential_paste");
    } catch (error) {
      context.operationError.value = errorMessage(error);
    } finally {
      context.busy.value = false;
    }
  }

  return {
    clear: () => execute("clear_sequential_paste"),
    close,
    remove: (itemId: number) => execute("remove_sequential_paste_item", { itemId }),
    reorder: (itemIds: readonly number[]) => execute("reorder_sequential_paste", { itemIds: [...itemIds] }),
    setDirection: (direction: SequentialPasteDirection) => execute("set_sequential_paste_direction", { direction }),
    setMode: (mode: SequentialPasteMode) => execute("set_sequential_paste_mode", { mode }),
  };
}

function registerSubscriptions(context: SequentialPasteContext) {
  let disposed = false;
  let unlistenState: UnlistenFn | undefined;
  let unlistenSettings: UnlistenFn | undefined;

  async function initialize() {
    try {
      [unlistenState, unlistenSettings] = await Promise.all([
        listen<SequentialPasteSnapshot>(STATE_CHANGED_EVENT, ({ payload }) => {
          context.eventRevision.value += 1;
          context.state.value = payload;
          context.operationError.value = "";
        }),
        listen<AppSettings>(SETTINGS_UPDATED_EVENT, ({ payload }) => {
          context.shortcut.value = payload.sequentialPasteShortcut;
        }),
      ]);
      const stateRevision = context.eventRevision.value;
      const [snapshot, settings] = await Promise.all([
        invoke<SequentialPasteSnapshot>("get_sequential_paste_state"),
        invoke<AppSettings>("get_app_settings"),
      ]);
      if (disposed) return;
      if (stateRevision === context.eventRevision.value) context.state.value = snapshot;
      context.shortcut.value = settings.sequentialPasteShortcut;
    } catch (error) {
      if (!disposed) context.operationError.value = errorMessage(error);
    } finally {
      if (!disposed) context.initialized.value = true;
    }
  }

  onMounted(() => { void initialize(); });
  onUnmounted(() => {
    disposed = true;
    unlistenState?.();
    unlistenSettings?.();
  });
}

export function useSequentialPaste() {
  const context: SequentialPasteContext = {
    state: shallowRef<SequentialPasteSnapshot>(EMPTY_STATE),
    shortcut: shallowRef("Ctrl+Shift+V"),
    busy: shallowRef(false),
    operationError: shallowRef(""),
    initialized: shallowRef(false),
    eventRevision: shallowRef(0),
  };
  const nextItemId = computed(() => {
    const items = context.state.value.items;
    if (context.state.value.mode !== "paste" || items.length === 0) return undefined;
    return context.state.value.direction === "forward" ? items[0].id : items[items.length - 1]?.id;
  });
  const actions = createCommandActions(context);
  registerSubscriptions(context);

  return {
    ...actions,
    busy: readonly(context.busy),
    initialized: readonly(context.initialized),
    nextItemId,
    operationError: context.operationError,
    shortcut: readonly(context.shortcut),
    state: readonly(context.state),
  };
}
