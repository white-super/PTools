import type { TextFormatter, TextTransformAction } from "../types";
import { replaceResult } from "./textFormatterUtils";

const SUPPORTED_ACTIONS: readonly TextTransformAction[] = ["url-encode", "url-decode"];

export const urlFormatter: TextFormatter = {
  format: "url",
  title: "URL 编解码",
  editorLanguage: "plain",
  supportedActions: SUPPORTED_ACTIONS,
  matches: isUrlText,
  prepare: (content) => content.trim(),
  transform: transformUrl,
};

function transformUrl(action: TextTransformAction, content: string) {
  if (action === "url-encode") return replaceResult(encodeURIComponent(content), "已 URL 编码");
  if (action === "url-decode") return replaceResult(decodeURIComponent(content), "已 URL 解码");
  throw new Error("URL 不支持此操作");
}

function isUrlText(content: string) {
  return /^(?:https?|ftp):\/\/\S+$/i.test(content.trim());
}
