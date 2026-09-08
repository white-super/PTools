import {
  applyEdits,
  format as formatDocument,
  parse,
  printParseErrorCode,
  type ParseError,
} from "jsonc-parser";

export type JsonValue =
  | null
  | boolean
  | number
  | string
  | readonly JsonValue[]
  | { readonly [key: string]: JsonValue };

const JSON_FORMAT_OPTIONS = {
  insertSpaces: true,
  tabSize: 2,
  eol: "\n",
} as const;

const JSON_PARSE_OPTIONS = {
  allowTrailingComma: true,
  disallowComments: false,
} as const;

export function isJsonText(content: string) {
  try {
    const value = parseJsonText(content);
    return value !== null && typeof value === "object";
  } catch {
    return false;
  }
}

export function parseJsonText(content: string): JsonValue {
  return parseJsonValue(normalizeJsonText(content));
}

export function normalizeJsonText(content: string) {
  try {
    parseJsonValue(content);
    return content;
  } catch (directError) {
    const decoded = decodeEscapedJsonText(content);
    if (decoded !== undefined) {
      return decoded;
    }
    throw directError;
  }
}

function parseJsonValue(content: string): JsonValue {
  const errors: ParseError[] = [];
  const value: unknown = parse(content, errors, JSON_PARSE_OPTIONS);
  if (errors.length > 0) {
    throw createJsonError(content, errors[0]);
  }
  if (value === undefined) {
    throw new Error("JSON 内容不能为空");
  }
  return value as JsonValue;
}

export function formatJsonText(content: string) {
  const normalizedContent = normalizeJsonText(content);
  const edits = formatDocument(normalizedContent, undefined, JSON_FORMAT_OPTIONS);
  return applyEdits(normalizedContent, edits);
}

export function removeJsonComments(content: string) {
  return JSON.stringify(parseJsonText(content), null, JSON_FORMAT_OPTIONS.tabSize);
}

export function minifyJson(content: string) {
  return JSON.stringify(parseJsonText(content));
}

export function escapeMinifiedJson(content: string) {
  const escaped = JSON.stringify(minifyJson(content));
  return escaped.slice(1, -1);
}

function createJsonError(content: string, error: ParseError) {
  const beforeError = content.slice(0, error.offset);
  const line = beforeError.split("\n").length;
  const column = error.offset - beforeError.lastIndexOf("\n");
  const errorName = printParseErrorCode(error.error);
  return new Error(`JSON 格式错误：${errorName}（第 ${line} 行，第 ${column} 列）`);
}

function decodeEscapedJsonText(content: string) {
  const candidates: string[] = [];

  try {
    // Protect literal whitespace in the string wrapper without re-escaping existing escapes.
    const wrappedContent = content.replace(/[\r\n\t]/g, (character) => JSON.stringify(character).slice(1, -1));
    const decoded = JSON.parse(`"${wrappedContent}"`);
    if (typeof decoded === "string") {
      candidates.push(decoded);
    }
  } catch {
  }

  try {
    const decoded = JSON.parse(content);
    if (typeof decoded === "string") {
      candidates.push(decoded);
    }
  } catch {
  }

  for (const candidate of candidates) {
    if (candidate === content) {
      continue;
    }

    try {
      parseJsonValue(candidate);
      return candidate;
    } catch {
    }
  }

  return undefined;
}
