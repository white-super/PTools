import type { TextFormatter, TextTransformAction } from "../types";
import { formatMarkup, replaceResult } from "./textFormatterUtils";

const SUPPORTED_ACTIONS: readonly TextTransformAction[] = ["format"];

export const xmlFormatter: TextFormatter = {
  format: "xml",
  title: "XML 格式化",
  editorLanguage: "plain",
  supportedActions: SUPPORTED_ACTIONS,
  matches: isXmlText,
  prepare: formatMarkup,
  transform: (action, content) => {
    if (action !== "format") throw new Error("XML 不支持此操作");
    return replaceResult(formatMarkup(content), "已重新格式化 XML");
  },
};

function isXmlText(content: string) {
  const value = content.trim();
  return /^<\?xml\b/i.test(value) || /^<[A-Za-z][\w:.-]*(?:\s|>|\/)/.test(value);
}
