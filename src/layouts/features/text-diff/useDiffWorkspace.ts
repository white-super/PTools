import { computed, onMounted, onUnmounted, shallowRef, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { ElMessageBox } from "element-plus";
import { writeText } from "tauri-plugin-clipboard-api";
import { readTextFile } from "./readTextFile";
import { useDiffComputation } from "./useDiffComputation";
import type { DiffFormatChoice, DiffInput, DiffSide, DiffSource } from "./types";

export function useDiffWorkspace() {
  const original = shallowRef<DiffInput>();
  const left = shallowRef<DiffSource>({ name: "左侧内容", content: "" });
  const right = shallowRef<DiffSource>({ name: "右侧内容", content: "" });
  const format = shallowRef<DiffFormatChoice>("auto");
  const formatted = shallowRef(false);
  const wrapping = shallowRef(true);
  const collapse = shallowRef(false);
  const pinned = shallowRef(false);
  const operationError = shallowRef("");
  const status = shallowRef("");
  const loading = shallowRef<Readonly<Record<DiffSide, boolean>>>({ left: false, right: false });
  const computation = useDiffComputation();
  const displayed = shallowRef<{ left: string; right: string }>();
  const hasLiveEdits = computed(() => !!displayed.value && !!computation.result.value &&
    (displayed.value.left !== computation.result.value.left ||
      displayed.value.right !== computation.result.value.right));
  const imports = { left: 0, right: 0 };
  let disposed = false;
  let unlisten: UnlistenFn | undefined;
  let closing = false;
  let pinning = false;
  let statusTimer: ReturnType<typeof setTimeout> | undefined;
  const dirty = computed(
    () =>
      original.value &&
      (left.value.content !== original.value.left.content ||
        right.value.content !== original.value.right.content),
  );
  watch(computation.result, result => {
    displayed.value = result ? { left: result.left, right: result.right } : undefined;
  });

  function edit(side: DiffSide, content: string) {
    const target = side === "left" ? left : right;
    target.value = { ...target.value, content };
    if (displayed.value) displayed.value = { ...displayed.value, [side]: content };
  }

  function report(error: unknown) {
    operationError.value = error instanceof Error ? error.message : String(error);
  }
  function compare() {
    if (!original.value) return;
    computation.compute({
      left: left.value.content,
      right: right.value.content,
      format: format.value,
      formatted: formatted.value,
    });
  }
  watch([format, formatted], compare);

  async function confirmDiscard(message: string) {
    try {
      await ElMessageBox.confirm(message, "确认放弃临时修改", {
        confirmButtonText: "继续",
        cancelButtonText: "取消",
        type: "warning",
      });
      return true;
    } catch (error) {
      if (error !== "cancel" && error !== "close") report(error);
      return false;
    }
  }

  async function close() {
    if (closing) return;
    closing = true;
    try {
      if (dirty.value && !(await confirmDiscard("关闭后将丢弃临时编辑的内容，原文件和历史记录不会改变。")))
        return;
      await invoke("close_text_diff");
    } catch (error) {
      report(error);
    } finally {
      closing = false;
    }
  }

  async function togglePinned() {
    if (pinning) return;
    pinning = true;
    try {
      await invoke("set_text_diff_pinned", { pinned: !pinned.value });
      pinned.value = !pinned.value;
    } catch (error) {
      report(error);
    } finally {
      pinning = false;
    }
  }

  async function importFile(side: DiffSide, file: File) {
    if (!original.value || loading.value[side]) return;
    const revision = ++imports[side];
    const target = side === "left" ? left : right;
    if (
      target.value.content !== original.value?.[side].content &&
      !(await confirmDiscard("导入新文件会替换这一侧的临时修改。"))
    )
      return;
    if (disposed || imports[side] !== revision) return;
    const previous = target.value;
    loading.value = { ...loading.value, [side]: true };
    operationError.value = "";
    try {
      const source = await readTextFile(file);
      if (disposed || imports[side] !== revision) return;
      if (
        target.value !== previous &&
        !(await confirmDiscard("文件读取期间原文已更改，是否仍用新文件替换？"))
      )
        return;
      if (disposed || imports[side] !== revision) return;
      original.value = { ...original.value!, [side]: source };
      target.value = source;
      compare();
    } catch (error) {
      if (!disposed && imports[side] === revision) report(error);
    } finally {
      if (imports[side] === revision) loading.value = { ...loading.value, [side]: false };
    }
  }

  async function restore(side: DiffSide) {
    if (!original.value) return;
    const target = side === "left" ? left : right;
    if (
      target.value.content !== original.value[side].content &&
      !(await confirmDiscard("恢复原文会放弃这一侧的临时修改。"))
    )
      return;
    target.value = original.value[side];
    compare();
  }

  function swap() {
    if (!original.value || loading.value.left || loading.value.right) return;
    [left.value, right.value] = [right.value, left.value];
    original.value = { left: original.value.right, right: original.value.left };
    compare();
  }

  async function copy(side: DiffSide) {
    const content = displayed.value?.[side];
    if (content === undefined) return;
    try {
      await writeText(content);
      status.value = `${side === "left" ? "左侧" : "右侧"}已复制`;
      clearTimeout(statusTimer);
      statusTimer = setTimeout(() => {
        status.value = "";
      }, 1800);
    } catch (error) {
      report(error);
    }
  }

  async function initialize() {
    try {
      const [input, initialPinned] = await Promise.all([
        invoke<DiffInput>("get_text_diff_input"),
        invoke<boolean>("get_text_diff_pinned"),
      ]);
      if (disposed) return;
      pinned.value = initialPinned;
      original.value = input;
      left.value = input.left;
      right.value = input.right;
      compare();
      unlisten = await getCurrentWindow().onCloseRequested((event) => {
        event.preventDefault();
        void close();
      });
      if (disposed) unlisten();
    } catch (error) {
      if (!disposed) report(error);
    }
  }

  onMounted(() => {
    void initialize();
  });
  onUnmounted(() => {
    disposed = true;
    unlisten?.();
    clearTimeout(statusTimer);
  });
  return {
    ...computation,
    original,
    left,
    right,
    format,
    formatted,
    edit,
    hasLiveEdits,
    wrapping,
    collapse,
    pinned,
    operationError,
    status,
    loading,
    compare,
    close,
    togglePinned,
    importFile,
    restore,
    swap,
    copy,
  };
}
