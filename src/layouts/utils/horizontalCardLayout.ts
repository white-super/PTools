const MIN_CARD_WIDTH = 190;
const MAX_CARD_WIDTH = 240;
const CARD_VIEWPORT_RATIO = 0.18;
export const CARD_GAP = 12;
export const CARD_LIST_PADDING = 2;
const OVERSCAN_CARD_COUNT = 4;

export function cardWidthForViewport(width: number) {
  return Math.min(MAX_CARD_WIDTH, Math.max(MIN_CARD_WIDTH, width * CARD_VIEWPORT_RATIO));
}

interface CardLayoutOptions {
  readonly count: number;
  readonly cardWidth: number;
  readonly viewportWidth: number;
  readonly scrollLeft: number;
}

export function horizontalCardLayout(options: CardLayoutOptions) {
  const { count, cardWidth, viewportWidth } = options;
  const stride = cardWidth + CARD_GAP;
  const width = Math.max(0, count * stride - CARD_GAP);
  const maxScroll = Math.max(0, width + CARD_LIST_PADDING * 2 - viewportWidth);
  const scrollLeft = Math.min(maxScroll, Math.max(0, options.scrollLeft));
  const firstVisible = Math.floor(scrollLeft / stride);
  const endVisible = Math.ceil((scrollLeft + viewportWidth) / stride);
  return {
    width,
    stride,
    start: Math.max(0, firstVisible - OVERSCAN_CARD_COUNT),
    end: Math.min(count, endVisible + OVERSCAN_CARD_COUNT),
  };
}

export function scrollOffsetForCard(options: CardLayoutOptions, index: number) {
  const start = index * (options.cardWidth + CARD_GAP);
  const end = start + options.cardWidth + CARD_LIST_PADDING * 2;
  if (start < options.scrollLeft) return start;
  if (end > options.scrollLeft + options.viewportWidth) return end - options.viewportWidth;
  return options.scrollLeft;
}
