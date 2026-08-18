import { invoke } from "@tauri-apps/api/core";
import { ElMessage } from "element-plus";
import { computed, onMounted, onUnmounted, shallowRef } from "vue";
import { writeText } from "tauri-plugin-clipboard-api";
import { getTextFormatter } from "../formatters/registry";
import type { TextFormatterInput, TextTransformAction } from "../types";

interface UseTextFormatterOptions {
  readonly windowId: string;
}

export function useTextFormatter(options: UseTextFormatterOptions) {
  const input = shallowRef<TextFormatterInput>();
  const content = shallowRef("");
  const isPinned = shallowRef(false);

  const formatter = computed(() => input.value
    ? getTextFormatter(input.value.format)
    : undefined);
  const title = computed(() => formatter.value?.title ?? "文本格式化");
  const supportedActions = computed(() => formatter.value?.supportedActions ?? []);

  function receiveInput(nextInput: TextFormatterInput) {
    input.value = nextInput;
    try {
      content.value = getTextFormatter(nextInput.format).prepare(nextInput.content);
    } catch (error) {
      content.value = nextInput.content;
      reportFormatterError(error);
    }
  }

  async function togglePinned() {
    try {
      isPinned.value = await invoke<boolean>("set_text_formatter_pinned", {
        windowId: options.windowId,
        pinned: !isPinned.value,
      });
    } catch (error) {
      reportFormatterError(error);
    }
  }

  async function applyTransform(action: TextTransformAction) {
    const activeFormatter = formatter.value;
    if (!activeFormatter) {
      return;
    }
    try {
      const result = activeFormatter.transform(action, content.value);
      if (result.effect === "replace") {
        content.value = result.output;
      } else {
        await writeText(result.output);
      }
      ElMessage.success(result.message);
    } catch (error) {
      reportFormatterError(error);
    }
  }

  function close() {
    isPinned.value = false;
    void invoke<void>("close_text_formatter", { windowId: options.windowId }).catch(reportFormatterError);
  }

  function handleEscape(event: KeyboardEvent) {
    if (event.key !== "Escape") {
      if (!isPinShortcut(event)) {
        return;
      }
      event.preventDefault();
      void togglePinned();
      return;
    }
    event.preventDefault();
    close();
  }

  async function initialize() {
    const currentInput = await invoke<TextFormatterInput>("get_text_formatter_input", {
      windowId: options.windowId,
    });
    input.value = currentInput;
    isPinned.value = await invoke<boolean>("get_text_formatter_pinned", {
      windowId: options.windowId,
    });
    receiveInput(currentInput);
  }

  onMounted(() => {
    window.addEventListener("keydown", handleEscape, true);
    void initialize().catch(reportFormatterError);
  });

  onUnmounted(() => {
    window.removeEventListener("keydown", handleEscape, true);
  });

  return { applyTransform, close, content, isPinned, supportedActions, title, togglePinned };
}

function isPinShortcut(event: KeyboardEvent) {
  const commandShortcut = event.metaKey
    && !event.altKey
    && !event.ctrlKey
    && !event.shiftKey;
  const altShortcut = event.altKey
    && !event.metaKey
    && !event.ctrlKey
    && !event.shiftKey;

  return event.code === "KeyD"
    && (commandShortcut || altShortcut);
}

function reportFormatterError(error: unknown) {
  console.error("Failed to operate text formatter", error);
  ElMessage.error(error instanceof Error ? error.message : String(error));
}
