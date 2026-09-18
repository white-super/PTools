import type { ClipboardFormat } from "../../types/settings";

export type SequentialPasteMode = "capture" | "paste";
export type SequentialPasteDirection = "forward" | "reverse";

export interface SequentialPasteItem {
  readonly id: number;
  readonly format: ClipboardFormat;
  readonly content: string;
  readonly filePaths: readonly string[];
}

export interface SequentialPasteError {
  readonly itemId?: number;
  readonly message: string;
}

export interface SequentialPasteSnapshot {
  readonly enabled: boolean;
  readonly mode: SequentialPasteMode;
  readonly direction: SequentialPasteDirection;
  readonly items: readonly SequentialPasteItem[];
  readonly error?: SequentialPasteError;
}
