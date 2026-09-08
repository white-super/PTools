import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import ts from "typescript";

// Use the project's TypeScript compiler so these checks also run on Node 20.
const source = await readFile(new URL("../src/layouts/utils/horizontalCardLayout.ts", import.meta.url), "utf8");
const { outputText } = ts.transpileModule(source, {
  compilerOptions: { module: ts.ModuleKind.ESNext, target: ts.ScriptTarget.ES2022 },
});
const { cardWidthForViewport, horizontalCardLayout, scrollOffsetForCard } = await import(
  `data:text/javascript;base64,${Buffer.from(outputText).toString("base64")}`
);
const options = { count: 200, cardWidth: 240, viewportWidth: 1486, scrollLeft: 20000 };

test("render only nearby cards, independently of history size", () => {
  for (const count of [200, 2000, 20000]) {
    const layout = horizontalCardLayout({ ...options, count });
    assert.ok(layout.end - layout.start < 20);
    assert.ok(layout.start * layout.stride <= options.scrollLeft);
    assert.ok(layout.end * layout.stride >= options.scrollLeft + options.viewportWidth);
  }
});

test("appending a page preserves the current card positions and rendered range", () => {
  const before = horizontalCardLayout(options);
  const after = horizontalCardLayout({ ...options, count: options.count + 20 });
  assert.equal(after.start, before.start);
  assert.equal(after.end, before.end);
  assert.equal(after.stride, before.stride);
  assert.ok(after.width > before.width);
});

test("the first and last records remain reachable, including overscroll", () => {
  const first = horizontalCardLayout({ ...options, scrollLeft: -100 });
  const last = horizontalCardLayout({ ...options, scrollLeft: 1000000 });
  assert.equal(first.start, 0);
  assert.equal(last.end, options.count);
  assert.ok(last.start < last.end);
});

test("empty lists and shorter search results never use an obsolete scroll range", () => {
  for (const count of [0, 1, 3]) {
    const layout = horizontalCardLayout({ ...options, count });
    assert.equal(layout.start, 0);
    assert.equal(layout.end, count);
    assert.ok(layout.width >= 0);
  }
});

test("keyboard navigation reveals a card even when it is not currently mounted", () => {
  for (const index of [0, 18, 199]) {
    const scrollLeft = scrollOffsetForCard(options, index);
    const layout = horizontalCardLayout({ ...options, scrollLeft });
    assert.ok(index >= layout.start && index < layout.end);
    const left = index * layout.stride;
    assert.ok(left >= scrollLeft);
    assert.ok(left + options.cardWidth <= scrollLeft + options.viewportWidth);
  }
});

test("keyboard selection of an already visible card does not move the list", () => {
  const layout = horizontalCardLayout(options);
  const index = Math.ceil(options.scrollLeft / layout.stride);
  assert.equal(scrollOffsetForCard(options, index), options.scrollLeft);
});

test("resize preserves coverage across laptop and wide monitor widths", () => {
  for (const viewportWidth of [800, 1100, 2000, 3840]) {
    const cardWidth = cardWidthForViewport(viewportWidth);
    assert.ok(cardWidth >= 190 && cardWidth <= 240);
    const layout = horizontalCardLayout({ ...options, viewportWidth, cardWidth });
    assert.ok(layout.start * layout.stride <= options.scrollLeft);
    assert.ok(layout.end * layout.stride >= options.scrollLeft + viewportWidth);
  }
});
