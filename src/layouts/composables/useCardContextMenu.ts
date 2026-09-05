import { shallowRef, type Ref } from "vue";
import { getTextFormatterOptions } from "../features/text-formatter/formatters/registry";
import type { TextFormatterOption } from "../features/text-formatter/types";
import type { ClipboardHistoryEntry, ClipboardTag } from "../types/settings";

const MENU_WIDTH = 152;
const MENU_MAX_HEIGHT = 264;
const MENU_BASE_HEIGHT = 112;
const MENU_ROW_HEIGHT = 32;
const MENU_VERTICAL_PADDING = 8;
const VIEWPORT_MARGIN = 8;

interface UseCardContextMenuOptions {
  readonly tags: Readonly<Ref<readonly ClipboardTag[]>>;
}

export interface CardContextMenuState {
  readonly card: ClipboardHistoryEntry;
  readonly cardId: number;
  readonly x: number;
  readonly y: number;
  readonly assignedTagIds: readonly number[];
  readonly tools: readonly TextFormatterOption[];
}

export function useCardContextMenu(options: UseCardContextMenuOptions) {
  const contextMenu = shallowRef<CardContextMenuState>();

  function closeCardContextMenu() {
    contextMenu.value = undefined;
  }

  function openCardContextMenu(card: ClipboardHistoryEntry, event: MouseEvent) {
    const tools = availableTools(card);
    contextMenu.value = {
      card,
      cardId: card.id,
      x: Math.min(
        event.clientX,
        Math.max(VIEWPORT_MARGIN, window.innerWidth - MENU_WIDTH - VIEWPORT_MARGIN),
      ),
      y: Math.min(
        event.clientY,
        Math.max(VIEWPORT_MARGIN, window.innerHeight - menuHeight(tools.length) - VIEWPORT_MARGIN),
      ),
      assignedTagIds: card.tagIds,
      tools,
    };
  }

  function menuHeight(toolCount: number) {
    const toolHeight = toolCount * MENU_ROW_HEIGHT + MENU_VERTICAL_PADDING;
    const tagHeight = options.tags.value.length > 0 ? MENU_MAX_HEIGHT : 0;
    return Math.max(MENU_BASE_HEIGHT, toolHeight, tagHeight);
  }

  return { closeCardContextMenu, contextMenu, openCardContextMenu };
}

function availableTools(card: ClipboardHistoryEntry) {
  return card.format === "text" ? getTextFormatterOptions() : [];
}
