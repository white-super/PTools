import type { TextFormatter, TextTransformAction } from "../types";
import { decodeBase64, encodeBase64, replaceResult } from "./textFormatterUtils";

const SUPPORTED_ACTIONS: readonly TextTransformAction[] = ["base64-encode", "base64-decode"];

export const base64Formatter: TextFormatter = {
  format: "base64",
  title: "Base64 编解码",
  editorLanguage: "plain",
  supportedActions: SUPPORTED_ACTIONS,
  matches: isBase64Text,
  prepare: (content) => content.trim(),
  transform: transformBase64,
};

function transformBase64(action: TextTransformAction, content: string) {
  if (action === "base64-encode") return replaceResult(encodeBase64(content), "已 Base64 编码");
  if (action === "base64-decode") return replaceResult(decodeBase64(content), "已 Base64 解码");
  throw new Error("Base64 不支持此操作");
}

function isBase64Text(content: string) {
  const value = content.trim();
  return value.length >= 8 && value.length % 4 === 0 && /^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?$/.test(value);
}
