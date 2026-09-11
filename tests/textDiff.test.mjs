import assert from "node:assert/strict";
import test from "node:test";
import { sourceUrl } from "./loadTypeScript.mjs";
import { diff, Change, Chunk } from "@codemirror/merge";
import { Text } from "@codemirror/state";

const root = new URL("../src/layouts/features/text-diff/", import.meta.url);
const LONG_DESCRIPTION_LENGTH = 180;
const LONG_SCRIPT_LENGTH = 260;
const { prepareDiff, detectDiffFormat } = await import(await sourceUrl(new URL("diffFormat.ts", root)));
const { computeDiffChanges } = await import(await sourceUrl(new URL("diffChanges.ts", root)));
const { useDiffSelection } = await import(await sourceUrl(new URL("useDiffSelection.ts", root)));
const { decodeTextFile } = await import(await sourceUrl(new URL("readTextFile.ts", root)));
const request = (left, right, options = {}) => ({
  left,
  right,
  format: "auto",
  formatted: false,
  ...options,
});

test("raw comparison preserves CRLF, whitespace, empty content and JSON key order", async () => {
  for (const left of ["", "\r\n\t hello \n", '{"b":1,"a":2}']) {
    const prepared = await prepareDiff(request(left, left));
    assert.equal(prepared.left, left);
    assert.equal(prepared.right, left);
    assert.equal(diff(prepared.left, prepared.right).length, 0);
  }
});

test("formatted JSON comparison handles escaped multiline text without mutating source", async () => {
  const left = "\n" + String.raw`{\"name\":\"PTools\",\"items\":[1,2]}` + "\r\n";
  const right = '{ "name": "PTools", "items": [1, 2] }';
  const input = request(left, right, { formatted: true });
  const prepared = await prepareDiff(input);
  assert.equal(prepared.left, prepared.right);
  assert.equal(input.left, left);
  assert.equal(prepared.format, "json");
});

test("formatted JSON retains array order, key order and string whitespace differences", async () => {
  for (const [left, right] of [
    ['{"x":[1,2]}', '{"x":[2,1]}'],
    ['{"a":1,"b":2}', '{"b":2,"a":1}'],
    ['{"x":"a b"}', '{"x":"a  b"}'],
  ]) {
    const result = await prepareDiff(request(left, right, { formatted: true }));
    assert.ok(diff(result.left, result.right).length > 0);
  }
});

test("invalid and mixed formats report errors instead of silently changing mode", async () => {
  await assert.rejects(
    prepareDiff(request('{"a":', "{}", { format: "json", formatted: true })),
    /左侧格式化失败/,
  );
  await assert.rejects(
    prepareDiff(request("{}", "{", { format: "json", formatted: true })),
    /右侧格式化失败/,
  );
  await assert.rejects(prepareDiff(request("{}", "<root/>", { formatted: true })), /双方不是同一种/);
  await assert.rejects(prepareDiff(request("hello", "world", { formatted: true })), /双方不是同一种/);
  assert.equal((await prepareDiff(request("{}", "<root/>"))).format, "plain");
});

test("XML formatter preserves mixed text, comments, CDATA and xml:space", async () => {
  const content =
    '<root xml:space="preserve">before <b>bold</b> after<!--note--><![CDATA[a < b\n c]]></root>';
  const result = await prepareDiff(request(content, content, { format: "xml", formatted: true }));
  // Formatting may wrap within a start tag, but must not alter mixed text nodes.
  assert.match(result.left, /before <b\s*>bold<\/b> after/);
  assert.ok(result.left.includes("<!--note-->"));
  assert.ok(result.left.includes("<![CDATA[a < b\n c]]>"));
  assert.equal(result.left, result.right);
  await assert.rejects(
    prepareDiff(request("<root><x></root>", "<root/>", { format: "xml", formatted: true })),
    /左侧/,
  );
});

test("HTML formatting never executes scripts and preserves preformatted content", async () => {
  const content = "<div><pre> a\n   b </pre><script>globalThis.__diffExecuted = true;</script></div>";
  const result = await prepareDiff(request(content, content, { format: "html", formatted: true }));
  assert.ok(result.left.includes(" a\n   b "));
  assert.ok(result.left.includes("globalThis.__diffExecuted = true;"));
  assert.equal(globalThis.__diffExecuted, undefined);
  assert.equal(detectDiffFormat(content), "html");
});

test("serialized worker changes can be reconstructed into accurate merge chunks", () => {
  for (const [left, right] of [
    ["", "new"],
    ["old", ""],
    ["a\r\nb", "a\nb"],
    ["你好 world", "你好 PTools"],
  ]) {
    const changes = JSON.parse(JSON.stringify(diff(left, right))).map(
      (c) => new Change(c.fromA, c.toA, c.fromB, c.toB),
    );
    const chunks = Chunk.build(Text.of(left.split("\n")), Text.of(right.split("\n")), {
      override: () => changes,
    });
    assert.ok(chunks.length > 0);
  }
});

test("structured comparison keeps unchanged lines between distant changes", () => {
  const sharedBlock = [
    '  "parameters": {',
    '    "type": "object",',
    '    "properties": {',
    '      "question": { "type": "string" }',
    '    },',
    '    "method": "GET"',
    '  },',
  ].join("\n");
  const document = (description, script) => [
    "{",
    `  "description":"${description}",`,
    sharedBlock,
    `  "responseParser":"${script}"`,
    "}",
  ].join("\n");
  const left = document("左".repeat(LONG_DESCRIPTION_LENGTH), "A".repeat(LONG_SCRIPT_LENGTH));
  const right = document("右".repeat(LONG_DESCRIPTION_LENGTH), "B".repeat(LONG_SCRIPT_LENGTH));
  const changes = computeDiffChanges(left, right);
  const sharedPosition = left.indexOf('"method": "GET"');

  assert.equal(changes.length, 2);
  assert.ok(changes.every(change => sharedPosition < change.fromA || sharedPosition >= change.toA));
});

test("file decoding supports UTF-8 BOM and rejects binary and other encodings", () => {
  assert.equal(decodeTextFile(new Uint8Array([0xef, 0xbb, 0xbf, 97]).buffer), "a");
  assert.equal(decodeTextFile(new TextEncoder().encode("中文\r\n\t").buffer), "中文\r\n\t");
  assert.throws(() => decodeTextFile(new Uint8Array([0xff, 0xfe, 65, 0]).buffer), /UTF-8/);
  assert.throws(() => decodeTextFile(new Uint8Array([65, 0, 66]).buffer), /二进制/);
});

function selection(overrides = {}) {
  const launched = [],
    errors = [];
  const state = useDiffSelection({
    resolve: async (card) => ({ name: String(card.id), content: card.content }),
    launch: async (input) => {
      launched.push(input);
    },
    reportError: (error, card) => errors.push({ error, card }),
    ...overrides,
  });
  return { ...state, launched, errors };
}

test("double D snapshots first card and clears only after opening the second", async () => {
  const state = selection();
  const first = { id: 1, content: "old" };
  await state.select(first);
  first.content = "changed later";
  assert.equal(state.pending.value.source.content, "old");
  await state.select({ id: 2, content: "new" });
  assert.equal(state.launched[0].left.content, "old");
  assert.equal(state.launched[0].right.content, "new");
  assert.equal(state.pending.value, undefined);
});

test("same card cancels; failed launch retains source; cancelled selection is ignored", async () => {
  const state = selection({
    launch: async () => {
      throw new Error("open failed");
    },
  });
  await state.select({ id: 1, content: "left" });
  await state.select({ id: 1, content: "left" });
  assert.equal(state.pending.value, undefined);
  await state.select({ id: 1, content: "left" });
  await state.select({ id: 2, content: "right" });
  assert.equal(state.pending.value.cardId, 1);
  assert.equal(state.errors.length, 1);
  assert.equal(state.errors[0].card.id, 2);
  assert.match(state.errors[0].error.message, /open failed/);
  let resolve;
  const pending = selection({
    resolve: () =>
      new Promise((done) => {
        resolve = done;
      }),
  });
  const selecting = pending.select({ id: 3 });
  pending.cancel();
  resolve({ name: "late", content: "late" });
  await selecting;
  assert.equal(pending.pending.value, undefined);
});

test("repeated activation during async selection never opens duplicate windows", async () => {
  let resolve;
  const state = selection({
    resolve: () =>
      new Promise((done) => {
        resolve = done;
      }),
  });
  const first = state.select({ id: 1 });
  await state.select({ id: 2 });
  resolve({ name: "first", content: "first" });
  await first;
  assert.equal(state.pending.value.cardId, 1);
  assert.equal(state.launched.length, 0);
});
