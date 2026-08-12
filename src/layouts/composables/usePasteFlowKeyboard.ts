import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { onMounted, onUnmounted, shallowRef, type Ref } from "vue";
import type { ClipboardHistoryEntry } from "../types/settings";

interface UsePasteFlowKeyboardOptions {
  readonly cards: Readonly<Ref<readonly ClipboardHistoryEntry[]>>;
  readonly getCardContainer: () => HTMLElement | null;
  readonly closeMenus: () => void;
  readonly pasteCard: (card: ClipboardHistoryEntry) => void;
  readonly reportError: (error: unknown) => void;
}

const MAIN_PANEL_FOCUS_EVENT = "tauri://focus";
export const MAX_QUICK_SELECT_CARDS = 5;

export function usePasteFlowKeyboard(options: UsePasteFlowKeyboardOptions) {
  const selectedCardId = shallowRef<number>();
  let disposed = false;
  let unlistenPanelFocus: UnlistenFn | undefined;

  function selectCard(cardId: number) {
    selectedCardId.value = cardId;
  }

  function scrollSelectedCardIntoView(cardId: number) {
    const selector = `[data-card-id="${cardId}"]`;
    const card = options.getCardContainer()?.querySelector<HTMLElement>(selector);
    card?.scrollIntoView({ behavior: "smooth", block: "nearest", inline: "nearest" });
  }

  function resetSelection() {
    options.closeMenus();
    selectedCardId.value = options.cards.value[0]?.id;
    options.getCardContainer()?.scrollTo({ left: 0, behavior: "auto" });
  }

  function pasteSelectedCard() {
    const card = options.cards.value.find((entry) => entry.id === selectedCardId.value);
    if (card) {
      options.pasteCard(card);
    }
  }

  function pasteQuickSelectedCard(key: string) {
    const cardIndex = Number(key) - 1;
    if (cardIndex < 0 || cardIndex >= MAX_QUICK_SELECT_CARDS) {
      return false;
    }
    const card = options.cards.value[cardIndex];
    if (!card) {
      return true;
    }
    selectedCardId.value = card.id;
    options.pasteCard(card);
    return true;
  }

  function selectAdjacentCard(direction: -1 | 1) {
    const currentIndex = options.cards.value.findIndex((card) => card.id === selectedCardId.value);
    if (currentIndex < 0) {
      selectFirstCard();
      return;
    }
    const nextCard = options.cards.value[currentIndex + direction];
    if (nextCard) {
      selectCard(nextCard.id);
      scrollSelectedCardIntoView(nextCard.id);
    }
  }

  function selectFirstCard() {
    const firstCard = options.cards.value[0];
    if (firstCard) {
      selectCard(firstCard.id);
      scrollSelectedCardIntoView(firstCard.id);
    }
  }

  function handleNavigationKeydown(event: KeyboardEvent) {
    if (event.metaKey || event.ctrlKey || event.altKey || event.shiftKey) {
      return;
    }
    if (/^[1-5]$/.test(event.key)) {
      handleQuickSelectKey(event);
      return;
    }
    if (event.key === "Enter") {
      event.preventDefault();
      pasteSelectedCard();
      return;
    }
    if (event.key === "ArrowLeft" || event.key === "ArrowRight") {
      event.preventDefault();
      selectAdjacentCard(event.key === "ArrowLeft" ? -1 : 1);
    }
  }

  function handleQuickSelectKey(event: KeyboardEvent) {
    if (pasteQuickSelectedCard(event.key)) {
      event.preventDefault();
    }
  }

  onMounted(() => {
    window.addEventListener("keydown", handleNavigationKeydown);
    void listen(MAIN_PANEL_FOCUS_EVENT, resetSelection)
      .then((unlisten) => {
        if (disposed) {
          unlisten();
          return;
        }
        unlistenPanelFocus = unlisten;
      })
      .catch(options.reportError);
  });

  onUnmounted(() => {
    disposed = true;
    window.removeEventListener("keydown", handleNavigationKeydown);
    unlistenPanelFocus?.();
  });

  return { selectedCardId, selectCard };
}
