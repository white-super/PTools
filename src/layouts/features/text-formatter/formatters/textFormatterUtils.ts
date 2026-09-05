import type { TextTransformResult } from "../types";

export function replaceResult(output: string, message: string): TextTransformResult {
  return { output, effect: "replace", message };
}

export function copyResult(output: string, message: string): TextTransformResult {
  return { output, effect: "copy", message };
}

export function formatMarkup(content: string) {
  const normalized = content.replace(/>\s*</g, "><").trim();
  if (!normalized) {
    return "";
  }
  const tokens = normalized.split(/(?=<)|(?<=>)/).filter(Boolean);
  let depth = 0;
  const lines: string[] = [];
  for (const token of tokens) {
    const value = token.trim();
    if (!value) continue;
    if (value.startsWith("</")) depth = Math.max(0, depth - 1);
    lines.push(`${"  ".repeat(depth)}${value}`);
    if (isOpeningTag(value)) depth += 1;
  }
  return lines.join("\n");
}

function isOpeningTag(value: string) {
  return value.startsWith("<")
    && !value.startsWith("</")
    && !value.startsWith("<?")
    && !value.startsWith("<!")
    && !value.endsWith("/>");
}

export function encodeBase64(value: string) {
  const bytes = new TextEncoder().encode(value);
  let binary = "";
  for (const byte of bytes) binary += String.fromCharCode(byte);
  return btoa(binary);
}

export function decodeBase64(value: string) {
  const binary = atob(value.replace(/\s/g, ""));
  const bytes = Uint8Array.from(binary, (character) => character.charCodeAt(0));
  return new TextDecoder("utf-8", { fatal: true }).decode(bytes);
}

export function formatDateValue(value: string) {
  const date = parseDateValue(value);
  if (!date) throw new Error("无法识别日期或时间戳");
  return date.toISOString();
}

export function dateToTimestamp(value: string) {
  const date = parseDateValue(value);
  if (!date) throw new Error("无法识别日期");
  return String(date.getTime());
}

export function timestampToDate(value: string) {
  const timestamp = value.trim();
  if (!/^\d{10,13}$/.test(timestamp)) throw new Error("请输入 10 位或 13 位时间戳");
  const milliseconds = timestamp.length === 10 ? Number(timestamp) * 1000 : Number(timestamp);
  return new Date(milliseconds).toISOString();
}

function parseDateValue(value: string) {
  const normalized = value.trim();
  if (/^\d{10,13}$/.test(normalized)) {
    const milliseconds = normalized.length === 10 ? Number(normalized) * 1000 : Number(normalized);
    const timestampDate = new Date(milliseconds);
    return Number.isNaN(timestampDate.getTime()) ? undefined : timestampDate;
  }
  const parsed = Date.parse(normalized);
  if (Number.isNaN(parsed)) return undefined;
  return new Date(parsed);
}
