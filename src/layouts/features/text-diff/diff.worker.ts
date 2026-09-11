import { computeDiffChanges } from "./diffChanges";
import { prepareDiff } from "./diffFormat";
import type { DiffRequest, DiffResponse } from "./types";

self.onmessage = async (event: MessageEvent<DiffRequest>) => {
  let response: DiffResponse;
  try {
    const prepared = await prepareDiff(event.data);
    // Exact diffing stays off the UI thread and can be cancelled by terminating this worker.
    const changes = computeDiffChanges(prepared.left, prepared.right);
    response = { result: { ...prepared, changes } };
  } catch (error) {
    response = { error: error instanceof Error ? error.message : String(error) };
  }
  self.postMessage(response);
};
