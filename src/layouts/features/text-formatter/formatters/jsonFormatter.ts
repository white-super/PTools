import type {
  TextFormatter,
  TextTransformAction,
  TextTransformResult,
} from "../types";
import { jsonToTypeScript, jsonToXml } from "./jsonConverters";
import {
  escapeMinifiedJson,
  formatJsonText,
  isJsonText,
  minifyJson,
  parseJsonText,
  removeJsonComments,
} from "./jsonTools";

const SUPPORTED_ACTIONS: readonly TextTransformAction[] = [
  "format",
  "remove-comments",
  "minify-copy",
  "escape-copy",
  "to-xml-copy",
  "to-typescript-copy",
];

export const jsonFormatter: TextFormatter = {
  format: "json",
  title: "JSON 格式化",
  supportedActions: SUPPORTED_ACTIONS,
  matches: isJsonText,
  prepare: formatJsonText,
  transform: transformJson,
};

function transformJson(action: TextTransformAction, content: string): TextTransformResult {
  switch (action) {
    case "format":
      return replaceResult(formatJsonText(content), "已重新格式化");
    case "remove-comments":
      return replaceResult(removeJsonComments(content), "已去除注释");
    case "minify-copy":
      return copyResult(minifyJson(content), "已压缩并复制");
    case "escape-copy":
      return copyResult(escapeMinifiedJson(content), "已压缩转义并复制");
    case "to-xml-copy":
      return copyResult(jsonToXml(parseJsonText(content)), "已转换为 XML 并复制");
    case "to-typescript-copy":
      return copyResult(jsonToTypeScript(parseJsonText(content)), "已转换为 TypeScript 并复制");
  }
}

function replaceResult(output: string, message: string): TextTransformResult {
  return { output, effect: "replace", message };
}

function copyResult(output: string, message: string): TextTransformResult {
  return { output, effect: "copy", message };
}
