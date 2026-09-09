import { h } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { ElMessage, ElMessageBox } from "element-plus";
import type { ClipboardHistoryEntry } from "../types/settings";
import type { DiffSource } from "../features/text-diff/types";
import { useDiffSelection } from "../features/text-diff/useDiffSelection";

const SOURCE_PREVIEW_LENGTH = 40;

async function chooseFile(paths: readonly string[]) {
  if (paths.length === 1) return paths[0];
  let selected = paths[0];
  try {
    await ElMessageBox({
      title: "选择要对比的文件",
      showCancelButton: true,
      confirmButtonText: "选择",
      cancelButtonText: "取消",
      message: h(
        "select",
        {
          "aria-label": "选择对比文件",
          value: selected,
          style: "width:100%;color:var(--app-text);background:var(--app-surface);padding:8px",
          onChange: (event: Event) => {
            selected = (event.target as HTMLSelectElement).value;
          },
        },
        paths.map((path) => h("option", { value: path }, path)),
      ),
    });
    return selected;
  } catch (error) {
    if (error === "cancel" || error === "close") return undefined;
    throw error;
  }
}

async function resolveSource(card: ClipboardHistoryEntry): Promise<DiffSource | undefined> {
  if (card.format === "text") {
    return {
      name: card.content.slice(0, SOURCE_PREVIEW_LENGTH).replace(/\s+/g, " ") || "空文本",
      content: card.content,
    };
  }
  if (card.format !== "file" || card.filePaths.length === 0)
    throw new Error("请选择文本或文本文件卡片，图片无法进行文本对比");
  const path = await chooseFile(card.filePaths);
  return path === undefined ? undefined : invoke<DiffSource>("read_diff_file", { path });
}

export function useTextDiffLauncher() {
  return useDiffSelection({
    resolve: resolveSource,
    launch: (input) => invoke("show_text_diff", { input }),
    reportError: (error) => {
      console.error("Failed to select diff source", error);
      ElMessage.error(error instanceof Error ? error.message : String(error));
    },
  });
}
