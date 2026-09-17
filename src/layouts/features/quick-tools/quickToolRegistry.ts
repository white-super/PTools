import type { QuickToolDefinition, QuickToolId } from "./types";
import { QUICK_TOOL_IDS } from "./types";

export const DEFAULT_QUICK_TOOL_IDS: readonly QuickToolId[] = [
  "text-diff",
  "json",
  "url",
  "base64",
  "date",
];

export const QUICK_TOOL_DEFINITIONS: readonly QuickToolDefinition[] = [
  { id: "text-diff", label: "文本对比", tone: "blue", launch: { kind: "text-diff" } },
  { id: "json", label: "JSON 格式化", tone: "orange", launch: { kind: "formatter", format: "json" } },
  { id: "xml", label: "XML 格式化", tone: "purple", launch: { kind: "formatter", format: "xml" } },
  { id: "html", label: "HTML 格式化", tone: "cyan", launch: { kind: "formatter", format: "html" } },
  { id: "url", label: "URL 编解码", tone: "purple", launch: { kind: "formatter", format: "url" } },
  { id: "base64", label: "Base64 编解码", tone: "cyan", launch: { kind: "formatter", format: "base64" } },
  { id: "date", label: "日期转换", tone: "green", launch: { kind: "formatter", format: "date" } },
];

const QUICK_TOOL_BY_ID = new Map(QUICK_TOOL_DEFINITIONS.map((tool) => [tool.id, tool]));

export function isQuickToolId(value: string): value is QuickToolId {
  return QUICK_TOOL_IDS.some((toolId) => toolId === value);
}

export function getQuickToolDefinition(toolId: QuickToolId) {
  const tool = QUICK_TOOL_BY_ID.get(toolId);
  if (!tool) throw new Error(`未知快捷工具：${toolId}`);
  return tool;
}
