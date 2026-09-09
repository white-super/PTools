import { onUnmounted, shallowRef } from "vue";
import type { DiffRequest, DiffResponse, DiffResult } from "./types";

const EDIT_DEBOUNCE_MS = 180;

export function useDiffComputation() {
  const result = shallowRef<DiffResult>();
  const busy = shallowRef(false);
  const error = shallowRef("");
  let worker: Worker | undefined;
  let timer: ReturnType<typeof setTimeout> | undefined;
  let revision = 0;

  function stop() {
    revision++;
    clearTimeout(timer);
    worker?.terminate();
    worker = undefined;
    busy.value = false;
  }

  function compute(request: DiffRequest) {
    stop();
    result.value = undefined;
    error.value = "";
    busy.value = true;
    const currentRevision = revision;
    timer = setTimeout(() => {
      try {
        worker = new Worker(new URL("./diff.worker.ts", import.meta.url), { type: "module" });
        worker.onmessage = ({ data }: MessageEvent<DiffResponse>) => {
          if (currentRevision !== revision) return;
          if ("error" in data) error.value = data.error;
          else result.value = data.result;
          stop();
        };
        worker.onerror = (event) => {
          if (currentRevision !== revision) return;
          error.value = `对比计算失败：${event.message}`;
          stop();
        };
        worker.postMessage(request);
      } catch (cause) {
        error.value = `无法启动对比计算：${cause instanceof Error ? cause.message : String(cause)}`;
        stop();
      }
    }, EDIT_DEBOUNCE_MS);
  }

  function cancel() {
    stop();
    error.value = "已取消计算，原文未修改。可点击重新对比。";
  }

  onUnmounted(stop);
  return { result, busy, error, compute, cancel };
}
