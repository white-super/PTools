import { computed, onUnmounted, shallowRef, watch, type Ref } from "vue";
import type {
  ClipboardFilter,
  ClipboardHistoryEntry,
  ClipboardHistoryQuery,
} from "../types/settings";

const SEARCH_DEBOUNCE_DELAY_MS = 180;
const PREFETCH_VIEWPORT_COUNT = 1.5;
const MIN_PREFETCH_DISTANCE_PX = 600;

interface UseClipboardHistoryQueryOptions {
  readonly cards: Readonly<Ref<readonly ClipboardHistoryEntry[]>>;
  readonly hasMore: Readonly<Ref<boolean>>;
  readonly isLoading: Readonly<Ref<boolean>>;
  readonly loadMore: () => Promise<void>;
  readonly reload: (query: ClipboardHistoryQuery) => Promise<void>;
  readonly reportError: (error: unknown) => void;
  readonly resetScroll: () => void;
  readonly getCardContainer: () => HTMLElement | null;
}

export function useClipboardHistoryQuery(options: UseClipboardHistoryQueryOptions) {
  const activeFilter = shallowRef<ClipboardFilter>("all");
  const searchQuery = shallowRef("");
  let pendingSearchTimer: number | undefined;
  let prefetchFrame: number | undefined;
  let loadedQueryKey: string | undefined;
  let activeReload: { readonly key: string; readonly promise: Promise<void> } | undefined;
  const historyQuery = computed<ClipboardHistoryQuery>(() => {
    const filter = activeFilter.value;
    return {
      search: searchQuery.value,
      ...(typeof filter === "number"
        ? { tagId: filter }
        : filter === "all"
          ? {}
          : { format: filter }),
    };
  });

  function reloadHistory(query: ClipboardHistoryQuery) {
    options.resetScroll();
    const key = getQueryKey(query);
    const request = options.reload(query);
    const promise = request
      .then(() => {
        if (activeReload?.promise === promise) {
          loadedQueryKey = key;
        }
      })
      .finally(() => {
        if (activeReload?.promise === promise) {
          activeReload = undefined;
        }
      });
    activeReload = { key, promise };
    return promise;
  }

  function clearPendingSearch(timer?: number) {
    if (timer !== undefined) {
      window.clearTimeout(timer);
    }
    if (timer === undefined || pendingSearchTimer === timer) {
      pendingSearchTimer = undefined;
    }
  }

  async function flushSearch() {
    clearPendingSearch(pendingSearchTimer);
    const query = historyQuery.value;
    const key = getQueryKey(query);
    if (activeReload?.key === key) {
      await activeReload.promise;
      return;
    }
    if (loadedQueryKey !== key) {
      await reloadHistory(query);
    }
  }

  function loadMoreWhenNearEnd(container: HTMLElement) {
    const remainingScroll = container.scrollWidth - container.clientWidth - container.scrollLeft;
    const prefetchDistance = Math.max(
      container.clientWidth * PREFETCH_VIEWPORT_COUNT,
      MIN_PREFETCH_DISTANCE_PX,
    );
    if (
      container.clientWidth === 0
      || remainingScroll > prefetchDistance
      || !options.hasMore.value
      || options.isLoading.value
      || pendingSearchTimer !== undefined
    ) {
      return;
    }
    void options.loadMore().catch(options.reportError);
  }

  function schedulePrefetch() {
    if (prefetchFrame !== undefined) return;
    prefetchFrame = window.requestAnimationFrame(() => {
      prefetchFrame = undefined;
      const container = options.getCardContainer();
      if (container) loadMoreWhenNearEnd(container);
    });
  }

  // Recheck after appended cards render, even if scrolling stopped at the old edge.
  watch(() => options.cards.value.length, schedulePrefetch, { flush: "post" });

  onUnmounted(() => {
    if (prefetchFrame !== undefined) window.cancelAnimationFrame(prefetchFrame);
    clearPendingSearch(pendingSearchTimer);
  });

  watch(
    historyQuery,
    (query, previousQuery, onCleanup) => {
      const searchChanged = query.search !== previousQuery?.search;
      if (!searchChanged || !query.search.trim()) {
        void reloadHistory(query).catch(options.reportError);
        return;
      }
      const timer = window.setTimeout(() => {
        clearPendingSearch(timer);
        void reloadHistory(query).catch(options.reportError);
      }, SEARCH_DEBOUNCE_DELAY_MS);
      pendingSearchTimer = timer;
      onCleanup(() => clearPendingSearch(timer));
    },
    { immediate: true },
  );

  return {
    activeFilter,
    flushSearch,
    handleCardContainerScroll: schedulePrefetch,
    searchQuery,
    visibleCards: options.cards,
  };
}

function getQueryKey(query: ClipboardHistoryQuery) {
  return JSON.stringify([query.search, query.format ?? null, query.tagId ?? null]);
}
