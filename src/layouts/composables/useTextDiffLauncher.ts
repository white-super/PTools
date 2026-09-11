import { h } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { ElMessageBox } from "element-plus";
import type { ClipboardHistoryEntry } from "../types/settings";
import type { DiffInput, DiffSource } from "../features/text-diff/types";
import { useDiffSelection } from "../features/text-diff/useDiffSelection";

const SOURCE_PREVIEW_LENGTH = 40;
const EMPTY_DIFF_INPUT: DiffInput = {
  left: { name: "左侧内容", content: "" },
  right: { name: "右侧内容", content: "" },
};

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
  if (card.format === "image")
    throw new Error("图片卡片不支持文本对比");
  if (card.filePaths.length === 0) throw new Error("文件卡片没有可读取的文件路径");
  const path = await chooseFile(card.filePaths);
  return path === undefined ? undefined : invoke<DiffSource>("read_diff_file", { path });
}

export function useTextDiffLauncher(
  reportError: (error: unknown, card?: ClipboardHistoryEntry) => void,
) {
  const selection = useDiffSelection({
    resolve: resolveSource,
    launch: (input) => invoke("show_text_diff", { input, pinned: false }),
    reportError,
  });

  async function openEmptyTextDiff() {
    try {
      await invoke("show_text_diff", { input: EMPTY_DIFF_INPUT, pinned: true });
    } catch (error) {
      reportError(error);
    }
  }

  return { ...selection, openEmptyTextDiff };
}
