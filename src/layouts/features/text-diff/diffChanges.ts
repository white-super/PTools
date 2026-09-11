import { diff } from "@codemirror/merge";
import type { DiffChange } from "./types";

export function computeDiffChanges(left: string, right: string): readonly DiffChange[] {
  return diff(left, right);
}
