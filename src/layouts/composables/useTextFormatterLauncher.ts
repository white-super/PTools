import { invoke } from "@tauri-apps/api/core";
import { detectTextFormat } from "../features/text-formatter/formatters/registry";
import type { TextFormat, TextFormatterInput } from "../features/text-formatter/types";
import type { ClipboardHistoryEntry } from "../types/settings";

const EMPTY_SOURCE_UPDATED_AT = 0;

type ReportFormatterError = (error: unknown, card?: ClipboardHistoryEntry) => void;

export function useTextFormatterLauncher(reportError: ReportFormatterError) {
  function openTextFormatter(card: ClipboardHistoryEntry) {
    void launchTextFormatterFromCard(card).catch((error) => reportError(error, card));
  }

  function openTextFormatterWithFormat(card: ClipboardHistoryEntry, format: TextFormat) {
    void launchTextFormatterFromCard(card, format).catch((error) => reportError(error, card));
  }

  function openEmptyTextFormatter(format: TextFormat) {
    const input: TextFormatterInput = {
      format,
      content: "",
      updatedAt: EMPTY_SOURCE_UPDATED_AT,
    };
    void launchTextFormatter(input).catch(reportError);
  }

  return { openEmptyTextFormatter, openTextFormatter, openTextFormatterWithFormat };
}

async function launchTextFormatterFromCard(card: ClipboardHistoryEntry, requestedFormat?: TextFormat) {
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
  await launchTextFormatter(input);
}

async function launchTextFormatter(input: TextFormatterInput) {
  await invoke("show_text_formatter", { input });
}
