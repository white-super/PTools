import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { onMounted, onUnmounted, shallowRef, watch, type Ref } from "vue";
import type {
  ClipboardFilter,
  ClipboardHistoryEntry,
  PasteFlowShortcutSettings,
} from "../types/settings";
import { matchesKeyboardShortcut } from "../utils/keyboardShortcut";

type ShortcutSettingsRef = Readonly<Ref<PasteFlowShortcutSettings | undefined>>;

interface FilterNavigationOptions {
  readonly activeFilter: Ref<ClipboardFilter>;
  readonly filters: Readonly<Ref<readonly ClipboardFilter[]>>;
  readonly shortcutSettings: ShortcutSettingsRef;
}

interface CardNavigationOptions {
  readonly shortcutSettings: ShortcutSettingsRef;
}

interface UsePasteFlowKeyboardOptions {
  readonly cards: Readonly<Ref<readonly ClipboardHistoryEntry[]>>;
  readonly getCardContainer: () => HTMLElement | null;
  readonly cardNavigation: CardNavigationOptions;
  readonly closeMenus: () => void;
  readonly focusSearch: () => void;
  readonly filterNavigation: FilterNavigationOptions;
  readonly formatCard: (card: ClipboardHistoryEntry) => void;
  readonly pasteCard: (card: ClipboardHistoryEntry) => void;
  readonly reportError: (error: unknown) => void;
}

const MAIN_PANEL_FOCUS_EVENT = "ptools://main-panel-focus";
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

  function formatSelectedCard() {
    const card = options.cards.value.find((entry) => entry.id === selectedCardId.value)
      ?? options.cards.value[0];
    if (card) {
      options.formatCard(card);
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

  function selectAdjacentFilter(direction: -1 | 1) {
    const { activeFilter, filters } = options.filterNavigation;
    if (filters.value.length === 0) {
      return;
    }
    const currentIndex = Math.max(filters.value.indexOf(activeFilter.value), 0);
    const nextIndex = (currentIndex + direction + filters.value.length) % filters.value.length;
    options.closeMenus();
    activeFilter.value = filters.value[nextIndex];
  }

  function handleFilterShortcut(event: KeyboardEvent) {
    const shortcuts = options.filterNavigation.shortcutSettings.value;
    const direction = configuredDirection(
      event,
      shortcuts?.previousFilterShortcut ?? "",
      shortcuts?.nextFilterShortcut ?? "",
    );
    if (!direction) {
      return false;
    }
    event.preventDefault();
    selectAdjacentFilter(direction);
    return true;
  }

  function handleCardShortcut(event: KeyboardEvent) {
    const shortcuts = options.cardNavigation.shortcutSettings.value;
    const direction = configuredDirection(
      event,
      shortcuts?.previousCardShortcut ?? "",
      shortcuts?.nextCardShortcut ?? "",
    );
    if (!direction) {
      return false;
    }
    event.preventDefault();
    selectAdjacentCard(direction);
    return true;
  }

  function handleNavigationKeydown(event: KeyboardEvent) {
    if (event.metaKey && event.key.toLowerCase() === "f") {
      event.preventDefault();
      options.closeMenus();
      options.focusSearch();
      return;
    }
    if (handleFilterShortcut(event)) {
      return;
    }
    if (handleCardShortcut(event)) {
      return;
    }
    if (isEditableTarget(event.target)) {
      return;
    }
    if (event.metaKey || event.ctrlKey || event.altKey || event.shiftKey) {
      return;
    }
    if (event.key.toLowerCase() === "f") {
      event.preventDefault();
      options.closeMenus();
      formatSelectedCard();
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

  function isEditableTarget(target: EventTarget | null) {
    return target instanceof HTMLInputElement
      || target instanceof HTMLTextAreaElement
      || (target instanceof HTMLElement && target.isContentEditable);
  }

  watch(options.cards, (cards) => {
    if (!cards.some((card) => card.id === selectedCardId.value)) {
      selectedCardId.value = cards[0]?.id;
    }
  });

  watch(options.filterNavigation.filters, (filters) => {
    if (!filters.includes(options.filterNavigation.activeFilter.value)) {
      options.filterNavigation.activeFilter.value = "all";
    }
  });

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

function configuredDirection(event: KeyboardEvent, previous: string, next: string) {
  if (matchesKeyboardShortcut(event, previous)) return -1;
  if (matchesKeyboardShortcut(event, next)) return 1;
  return undefined;
}
