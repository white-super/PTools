import { invoke } from "@tauri-apps/api/core";
import { ElMessage } from "element-plus";
import { computed, onMounted, onUnmounted, shallowRef } from "vue";
import { writeText } from "tauri-plugin-clipboard-api";
import { formatterActionFromKeyboard } from "../formatterShortcuts";
import { getTextFormatter } from "../formatters/registry";
import { isLineWrappingShortcut } from "../../../utils/toolWindowShortcuts";
import type { TextFormatterInput, TextTransformAction } from "../types";

interface UseTextFormatterOptions {
  readonly windowId: string;
  readonly closeSearch?: () => boolean;
  readonly getSelectedText?: () => string | undefined;
  readonly replaceSelectedText?: (value: string) => boolean;
  readonly toggleLineWrapping: () => void;
}

const STATUS_VISIBLE_DURATION_MS = 1800;

export function useTextFormatter(options: UseTextFormatterOptions) {
  const input = shallowRef<TextFormatterInput>();
  const content = shallowRef("");
  const isPinned = shallowRef(false);
  const statusMessage = shallowRef("");
  let statusTimer: ReturnType<typeof setTimeout> | undefined;

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
      const selectedText = options.getSelectedText?.();
      const source = selectedText ?? content.value;
      const result = activeFormatter.transform(action, source);
      if (result.effect === "replace") {
        if (selectedText !== undefined && options.replaceSelectedText?.(result.output)) {
          showStatus(result.message);
          return;
        }
        content.value = result.output;
      } else {
        await writeText(result.output);
      }
      showStatus(result.message);
    } catch (error) {
      clearStatus();
      reportFormatterError(error);
    }
  }

  function showStatus(message: string) {
    clearTimeout(statusTimer);
    statusMessage.value = message;
    statusTimer = setTimeout(clearStatus, STATUS_VISIBLE_DURATION_MS);
  }

  function clearStatus() {
    clearTimeout(statusTimer);
    statusTimer = undefined;
    statusMessage.value = "";
  }

  function close() {
    isPinned.value = false;
    void invoke<void>("close_text_formatter", { windowId: options.windowId }).catch(reportFormatterError);
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      event.preventDefault();
      event.stopPropagation();
      if (options.closeSearch?.()) {
        return;
      }
      close();
      return;
    }
    if (isLineWrappingShortcut(event)) {
      event.preventDefault();
      options.toggleLineWrapping();
      return;
    }
    const transformAction = formatterActionFromKeyboard(event, input.value?.format);
    if (transformAction) {
      event.preventDefault();
      void applyTransform(transformAction);
      return;
    }
    if (isPinShortcut(event)) {
      event.preventDefault();
      void togglePinned();
    }
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
    window.addEventListener("keydown", handleKeydown, true);
    void initialize().catch(reportFormatterError);
  });

  onUnmounted(() => {
    window.removeEventListener("keydown", handleKeydown, true);
    clearTimeout(statusTimer);
  });

  const format = computed(() => input.value?.format);
  const editorLanguage = computed(() => formatter.value?.editorLanguage ?? "plain");

  return {
    applyTransform,
    close,
    content,
    editorLanguage,
    format,
    isPinned,
    supportedActions,
    statusMessage,
    title,
    togglePinned,
  };
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
