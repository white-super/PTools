<script setup lang="ts">
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { computed, onMounted, onUnmounted, shallowRef, useTemplateRef } from "vue";
import PasteCard from "./PasteCard/index.vue";
import type { PanelNoticeState } from "../composables/usePanelNotice";
import type { DiffSide } from "../features/text-diff/types";
import type { ClipboardHistoryEntry } from "../types/settings";
import { MAX_QUICK_SELECT_CARDS } from "../composables/usePasteFlowKeyboard";
import {
  CARD_LIST_BLEED,
  CARD_LIST_PADDING,
  cardWidthForViewport,
  horizontalCardLayout,
  scrollOffsetForCard,
} from "../utils/horizontalCardLayout";

const props = defineProps<{
  cards: readonly ClipboardHistoryEntry[];
  selectedCardId?: number;
  diffLeftCardId?: number;
  notice?: PanelNoticeState;
}>();
const emit = defineEmits<{
  select: [id: number];
  paste: [card: ClipboardHistoryEntry];
  contextMenu: [card: ClipboardHistoryEntry, event: MouseEvent];
  scroll: [];
  cancelDiff: [];
}>();
const element = useTemplateRef<HTMLDivElement>("element");
const viewportWidth = shallowRef(0);
const cardWidth = shallowRef(cardWidthForViewport(window.innerWidth));
const scrollLeft = shallowRef(0);
const layoutOptions = computed(() => ({
  count: props.cards.length,
  cardWidth: cardWidth.value,
  viewportWidth: viewportWidth.value,
  scrollLeft: scrollLeft.value,
}));
const layout = computed(() => horizontalCardLayout(layoutOptions.value));
const renderedCards = computed(() => props.cards.slice(layout.value.start, layout.value.end)
  .map((card, offset) => ({ card, index: layout.value.start + offset })));
let resizeObserver: ResizeObserver;
let wheelFrame: number | undefined;
let pendingWheelOffset = 0;
const WHEEL_LINE_HEIGHT_PX = 16;
const MAIN_PANEL_FOCUS_EVENT = "ptools://main-panel-focus";
let unlistenPanelFocus: UnlistenFn | undefined;
let viewportFrame: number | undefined;
let disposed = false;

function updateViewport() {
  const container = element.value;
  if (!container || container.clientWidth <= 0) return;
  viewportWidth.value = container.clientWidth;
  cardWidth.value = cardWidthForViewport(container.clientWidth);
  handleScroll();
}

function scheduleViewportUpdate() {
  if (viewportFrame !== undefined) return;
  viewportFrame = window.requestAnimationFrame(() => {
    viewportFrame = undefined;
    updateViewport();
  });
}

function handleScroll() {
  scrollLeft.value = element.value!.scrollLeft;
  emit("scroll");
}

function handleWheel(event: WheelEvent) {
  // Keep native horizontal trackpad scrolling and its momentum on the compositor.
  if (event.ctrlKey || event.shiftKey || Math.abs(event.deltaX) >= Math.abs(event.deltaY)) return;
  event.preventDefault();
  const unit = event.deltaMode === WheelEvent.DOM_DELTA_LINE ? WHEEL_LINE_HEIGHT_PX
    : event.deltaMode === WheelEvent.DOM_DELTA_PAGE ? viewportWidth.value : 1;
  pendingWheelOffset += event.deltaY * unit;
  if (wheelFrame !== undefined) return;
  wheelFrame = window.requestAnimationFrame(() => {
    wheelFrame = undefined;
    element.value!.scrollLeft += pendingWheelOffset;
    pendingWheelOffset = 0;
    handleScroll();
  });
}

function scrollToCard(id: number) {
  const index = props.cards.findIndex((card) => card.id === id);
  if (index < 0 || !element.value) return;
  const left = scrollOffsetForCard({ ...layoutOptions.value, scrollLeft: element.value.scrollLeft }, index);
  element.value.scrollTo({ left, behavior: "smooth" });
}

function diffSideForCard(card: ClipboardHistoryEntry): DiffSide | undefined {
  if (props.diffLeftCardId === card.id) return "left";
  if (props.diffLeftCardId === undefined || props.selectedCardId !== card.id) return undefined;
  return card.format === "image" ? undefined : "right";
}

onMounted(() => {
  resizeObserver = new ResizeObserver(updateViewport);
  resizeObserver.observe(element.value!);
  updateViewport();
  void listen(MAIN_PANEL_FOCUS_EVENT, scheduleViewportUpdate)
    .then((unlisten) => {
      if (disposed) {
        unlisten();
        return;
      }
      unlistenPanelFocus = unlisten;
    })
    .catch((error) => console.error("Failed to listen for main panel layout updates", error));
});

onUnmounted(() => {
  disposed = true;
  resizeObserver.disconnect();
  if (wheelFrame !== undefined) window.cancelAnimationFrame(wheelFrame);
  if (viewportFrame !== undefined) window.cancelAnimationFrame(viewportFrame);
  unlistenPanelFocus?.();
});

defineExpose({ element, scrollToCard });
</script>

<template>
  <div
    ref="element"
    class="card-container"
    :style="{ margin: `-${CARD_LIST_BLEED}px`, padding: `${CARD_LIST_PADDING}px` }"
    @scroll="handleScroll"
    @wheel="handleWheel"
  >
    <div class="card-track" :style="{ width: `${layout.width}px` }">
      <PasteCard
        v-for="{ card, index } in renderedCards"
        :key="card.id"
        class="positioned-card"
        :style="{ left: `${index * layout.stride}px`, width: `${cardWidth}px` }"
        :data-card-id="card.id"
        :content="card.content"
        :format="card.format"
        :file-paths="card.filePaths"
        :is-selected="props.selectedCardId === card.id"
        :diff-side="diffSideForCard(card)"
        :notice="props.notice?.cardId === card.id ? props.notice : undefined"
        :quick-key="index < MAX_QUICK_SELECT_CARDS ? index + 1 : undefined"
        @select="emit('select', card.id)"
        @cancel-diff="emit('cancelDiff')"
        @paste="emit('paste', card)"
        @context-menu="emit('contextMenu', card, $event)"
      />
    </div>
  </div>
</template>

<style scoped>
.card-container {
  box-sizing: border-box;
  flex: 1;
  min-height: 0;
  min-width: 0;
  overflow-x: auto;
  overflow-y: hidden;
  scrollbar-width: none;
  overscroll-behavior: contain;
  overflow-anchor: none;
}

.card-container::-webkit-scrollbar {
  display: none;
}

.card-track {
  position: relative;
  height: 100%;
}

.positioned-card {
  position: absolute;
  top: 0;
  height: 100%;
}
</style>
