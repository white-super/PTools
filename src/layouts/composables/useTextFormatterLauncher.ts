import { invoke } from "@tauri-apps/api/core";
import { detectTextFormat } from "../features/text-formatter/formatters/registry";
import type { TextFormat, TextFormatterInput } from "../features/text-formatter/types";
import type { ClipboardHistoryEntry } from "../types/settings";

export function useTextFormatterLauncher(reportError: (error: unknown, card: ClipboardHistoryEntry) => void) {
  function openTextFormatter(card: ClipboardHistoryEntry) {
    void launchTextFormatter(card).catch((error) => reportError(error, card));
  }

  function openTextFormatterWithFormat(card: ClipboardHistoryEntry, format: TextFormat) {
    void launchTextFormatter(card, format).catch((error) => reportError(error, card));
  }

  return { openTextFormatter, openTextFormatterWithFormat };
}

async function launchTextFormatter(card: ClipboardHistoryEntry, requestedFormat?: TextFormat) {
  if (card.format !== "text") {
    throw new Error("当前卡片不是文本内容");
  }
  const format = requestedFormat ?? detectTextFormat(card.content);
  if (!format) {
    throw new Error("当前文本暂不支持格式化工具");
  }
  const input: TextFormatterInput = {
    format,
    content: card.content,
    updatedAt: card.updatedAt,
  };
  await invoke("show_text_formatter", { input });
}
