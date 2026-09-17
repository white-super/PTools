import { ElMessageBox } from "element-plus";
import type { Ref } from "vue";
import type {
  ClipboardFilter,
  ClipboardTag,
  ClipboardTagInput,
} from "../types/settings";

interface UseClipboardTagActionsOptions {
  readonly activeFilter: Ref<ClipboardFilter>;
  readonly tags: Readonly<Ref<readonly ClipboardTag[]>>;
  readonly createTag: (input: ClipboardTagInput) => Promise<void>;
  readonly updateTag: (id: number, input: ClipboardTagInput) => Promise<void>;
  readonly deleteTag: (id: number) => Promise<void>;
  readonly reportError: (action: string, error: unknown) => void;
}

export function useClipboardTagActions(options: UseClipboardTagActionsOptions) {
  async function createTag(input: ClipboardTagInput) {
    try {
      await options.createTag(input);
    } catch (error) {
      options.reportError("create clipboard tag", error);
    }
  }

  async function updateTag(id: number, input: ClipboardTagInput) {
    try {
      await options.updateTag(id, input);
    } catch (error) {
      options.reportError("update clipboard tag", error);
    }
  }

  async function deleteTag(id: number) {
    const tag = options.tags.value.find((entry) => entry.id === id);
    if (!tag) return;
    let confirmed = false;
    try {
      confirmed = await confirmDeleteTag(tag.name);
    } catch (error) {
      options.reportError("confirm clipboard tag deletion", error);
      return;
    }
    if (!confirmed) return;
    try {
      await options.deleteTag(id);
      if (options.activeFilter.value === id) options.activeFilter.value = "all";
    } catch (error) {
      options.reportError("delete clipboard tag", error);
    }
  }

  return { createTag, deleteTag, updateTag };
}

async function confirmDeleteTag(name: string) {
  try {
    await ElMessageBox.confirm(
      `删除“${name}”后，所有卡片上的该标签关联都会被移除，且无法恢复。`,
      "确认删除标签",
      { type: "warning", confirmButtonText: "删除", cancelButtonText: "取消" },
    );
    return true;
  } catch (error) {
    if (error === "cancel" || error === "close") return false;
    throw error;
  }
}
