import { invoke } from "@tauri-apps/api/core";
import { ElMessage } from "element-plus";
import { detectTextFormat } from "../features/text-formatter/formatters/registry";
import type { TextFormatterInput } from "../features/text-formatter/types";
import type { ClipboardHistoryEntry } from "../types/settings";

export function useTextFormatterLauncher() {
  function openTextFormatter(card: ClipboardHistoryEntry) {
    void launchTextFormatter(card).catch(reportLauncherError);
  }

  return { openTextFormatter };
}

async function launchTextFormatter(card: ClipboardHistoryEntry) {
  if (card.format !== "text") {
    throw new Error("当前卡片不是文本内容");
  }
  const format = detectTextFormat(card.content);
  if (!format) {
    throw new Error("当前文本不是可识别的 JSON 内容");
  }
  const input: TextFormatterInput = {
    format,
    content: card.content,
    updatedAt: card.updatedAt,
  };
  await invoke("show_text_formatter", { input });
}

function reportLauncherError(error: unknown) {
  console.error("Failed to open text formatter", error);
  ElMessage.error(error instanceof Error ? error.message : String(error));
}
