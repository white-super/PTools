import { computed, type Ref } from "vue";
import type { ClipboardFilter, ClipboardTag } from "../types/settings";

interface UsePanelEmptyStateOptions {
  readonly activeFilter: Readonly<Ref<ClipboardFilter>>;
  readonly isLoading: Readonly<Ref<boolean>>;
  readonly searchQuery: Readonly<Ref<string>>;
  readonly tags: Readonly<Ref<readonly ClipboardTag[]>>;
}

export function usePanelEmptyState(options: UsePanelEmptyStateOptions) {
  const title = computed(() => {
    if (options.isLoading.value) return "正在加载历史内容";
    if (options.searchQuery.value.trim()) return "未找到匹配内容";
    if (typeof options.activeFilter.value === "number") {
      const tag = options.tags.value.find((entry) => entry.id === options.activeFilter.value);
      return `暂无“${tag?.name ?? "该标签"}”内容`;
    }
    if (options.activeFilter.value === "all") return "暂无历史内容";
    return "当前格式暂无内容";
  });

  const description = computed(() => {
    if (options.isLoading.value) return "请稍候";
    if (options.searchQuery.value.trim()) return "请尝试其他关键词或清空搜索";
    if (typeof options.activeFilter.value === "number") return "右键卡片即可为它添加标签";
    if (options.activeFilter.value === "all") return "复制文本、图片或文件后会自动显示在这里";
    return "请选择其他格式查看历史内容";
  });

  return { description, title };
}
