import { shallowRef, type Ref } from "vue";
import { getTextFormatterOptions } from "../features/text-formatter/formatters/registry";
import type { TextFormatterOption } from "../features/text-formatter/types";
import type { ClipboardHistoryEntry, ClipboardTag } from "../types/settings";
import {
  CARD_CONTEXT_MENU_BASE_HEIGHT_PX,
  CARD_CONTEXT_MENU_MAX_HEIGHT_PX,
  CARD_CONTEXT_MENU_ROW_HEIGHT_PX,
  CARD_CONTEXT_MENU_VERTICAL_PADDING_PX,
  CARD_CONTEXT_MENU_VIEWPORT_MARGIN_PX,
  CARD_CONTEXT_MENU_WIDTH_PX,
} from "../utils/cardContextMenuLayout";

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
        Math.max(
          CARD_CONTEXT_MENU_VIEWPORT_MARGIN_PX,
          window.innerWidth - CARD_CONTEXT_MENU_WIDTH_PX - CARD_CONTEXT_MENU_VIEWPORT_MARGIN_PX,
        ),
      ),
      y: Math.min(
        event.clientY,
        Math.max(
          CARD_CONTEXT_MENU_VIEWPORT_MARGIN_PX,
          window.innerHeight - menuHeight(tools.length) - CARD_CONTEXT_MENU_VIEWPORT_MARGIN_PX,
        ),
      ),
      assignedTagIds: card.tagIds,
      tools,
    };
  }

  function menuHeight(toolCount: number) {
    return Math.max(
      CARD_CONTEXT_MENU_BASE_HEIGHT_PX,
      submenuHeight(toolCount),
      submenuHeight(options.tags.value.length),
    );
  }

  return { closeCardContextMenu, contextMenu, openCardContextMenu };
}

function submenuHeight(rowCount: number) {
  return Math.min(
    CARD_CONTEXT_MENU_MAX_HEIGHT_PX,
    rowCount * CARD_CONTEXT_MENU_ROW_HEIGHT_PX + CARD_CONTEXT_MENU_VERTICAL_PADDING_PX,
  );
}

function availableTools(card: ClipboardHistoryEntry) {
  return card.format === "text" ? getTextFormatterOptions() : [];
}
