import { isQuickToolId } from "./quickToolRegistry";
import type { QuickToolId } from "./types";

export const MAX_QUICK_TOOLS = 5;

export function validateQuickToolOrder(toolIds: readonly string[]): asserts toolIds is readonly QuickToolId[] {
  if (toolIds.length > MAX_QUICK_TOOLS) throw new Error("快捷工具最多只能添加 5 个");
  if (new Set(toolIds).size !== toolIds.length) throw new Error("快捷工具不能重复添加");
  const unknownTool = toolIds.find((toolId) => !isQuickToolId(toolId));
  if (unknownTool) throw new Error(`未知快捷工具：${unknownTool}`);
}

export function placeQuickTool(
  currentToolIds: readonly QuickToolId[],
  toolId: QuickToolId,
  targetIndex: number,
) {
  validateQuickToolOrder(currentToolIds);
  if (!Number.isInteger(targetIndex) || targetIndex < 0 || targetIndex > currentToolIds.length) {
    throw new Error("快捷工具的目标位置无效");
  }
  const sourceIndex = currentToolIds.indexOf(toolId);
  if (sourceIndex < 0 && currentToolIds.length >= MAX_QUICK_TOOLS) {
    throw new Error("快捷工具最多只能添加 5 个");
  }
  const remainingToolIds = currentToolIds.filter((currentToolId) => currentToolId !== toolId);
  const insertionIndex = sourceIndex >= 0 && sourceIndex < targetIndex
    ? targetIndex - 1
    : targetIndex;
  return [
    ...remainingToolIds.slice(0, insertionIndex),
    toolId,
    ...remainingToolIds.slice(insertionIndex),
  ];
}

export function removeQuickTool(currentToolIds: readonly QuickToolId[], toolId: QuickToolId) {
  validateQuickToolOrder(currentToolIds);
  return currentToolIds.filter((currentToolId) => currentToolId !== toolId);
}
