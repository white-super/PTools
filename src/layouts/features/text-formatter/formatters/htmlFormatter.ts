import type { TextFormatter, TextTransformAction } from "../types";
import { formatMarkup, replaceResult } from "./textFormatterUtils";

const SUPPORTED_ACTIONS: readonly TextTransformAction[] = ["format"];

export const htmlFormatter: TextFormatter = {
  format: "html",
  title: "HTML 格式化",
  editorLanguage: "plain",
  supportedActions: SUPPORTED_ACTIONS,
  matches: isHtmlText,
  prepare: formatMarkup,
  transform: (action, content) => {
    if (action !== "format") throw new Error("HTML 不支持此操作");
    return replaceResult(formatMarkup(content), "已重新格式化 HTML");
  },
};

function isHtmlText(content: string) {
  const value = content.trim();
  return /^<!doctype\s+html\b/i.test(value) || /<(?:html|head|body|div|section|main|script|style)\b/i.test(value);
}
