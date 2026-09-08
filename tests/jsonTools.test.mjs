import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import ts from "typescript";

const source = await readFile(new URL("../src/layouts/features/text-formatter/formatters/jsonTools.ts", import.meta.url), "utf8");
const { outputText } = ts.transpileModule(source, {
  compilerOptions: { module: ts.ModuleKind.ESNext, target: ts.ScriptTarget.ES2022 },
});
const moduleText = outputText.replace('"jsonc-parser"', JSON.stringify(import.meta.resolve("jsonc-parser")));
const { isJsonText, formatJsonText, parseJsonText, normalizeJsonText, escapeMinifiedJson } = await import(
  `data:text/javascript;base64,${Buffer.from(moduleText).toString("base64")}`
);

const expected = { name: "PTools", message: "escaped json", items: [1, 2, 3] };
const escaped = String.raw`{\"name\":\"PTools\",\"message\":\"escaped json\",\"items\":[1,2,3]}`;

test("recognizes the original single-line escaped JSON", () => {
  assert.equal(isJsonText(escaped), true);
  assert.deepEqual(JSON.parse(formatJsonText(escaped)), expected);
});

for (const whitespace of ["\n", "\r\n", "\r", "\t"]) {
  test(`escaped JSON accepts actual whitespace ${JSON.stringify(whitespace)}`, () => {
    for (const content of [
      whitespace + escaped,
      escaped + whitespace,
      escaped.replaceAll(",", "," + whitespace),
      whitespace + escaped.replaceAll(",", "," + whitespace) + whitespace,
    ]) {
      assert.equal(isJsonText(content), true);
      assert.deepEqual(parseJsonText(content), expected);
      assert.deepEqual(JSON.parse(formatJsonText(content)), expected);
    }
  });
}

test("preserves escaped newlines, tabs, quotes and backslashes in string values", () => {
  const value = { message: 'first\nsecond\t"quoted"', path: "C:\\new\\test", literal: String.raw`\n` };
  const content = escapeMinifiedJson(JSON.stringify(value));
  assert.deepEqual(parseJsonText("\r\n\t" + content + "\n"), value);
  assert.deepEqual(JSON.parse(formatJsonText(content)), value);
});

test("preserves multiline JSONC comments when decoding escaped text", () => {
  const content = '{\n\t// comment\n\t"name": "PTools",\n}';
  const escapedQuotes = content.replaceAll('"', String.raw`\"`);
  assert.equal(normalizeJsonText(escapedQuotes), content);
  assert.deepEqual(parseJsonText(escapedQuotes), { name: "PTools" });
});

test("does not change ordinary JSON or already encoded whitespace", () => {
  const ordinary = "\n\t" + JSON.stringify(expected, null, 2) + "\r\n";
  assert.equal(normalizeJsonText(ordinary), ordinary);
  assert.deepEqual(parseJsonText(escaped + String.raw`\n`), expected);
});

test("still rejects malformed JSON and raw newlines inside string values", () => {
  for (const content of [
    escaped.slice(0, -1) + "\n",
    escaped.replace("escaped json", "escaped\njson"),
    String.raw`{\"name\":\"bad\x escape\"}`,
    "\nnot json\n",
  ]) {
    assert.equal(isJsonText(content), false);
    assert.throws(() => formatJsonText(content), /JSON/);
  }
});
