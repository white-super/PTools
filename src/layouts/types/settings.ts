export type ClipboardFormat = "text" | "image" | "file";
export type ClipboardFilter = "all" | ClipboardFormat | number;

export interface AppSettings {
  readonly mainShortcut: string;
  readonly historyRetentionDays: number;
  readonly maxHistoryEntries: number;
  readonly recordText: boolean;
  readonly recordImages: boolean;
  readonly recordFiles: boolean;
  readonly autoPaste: boolean;
}

export interface ClipboardHistoryEntry {
  readonly id: number;
  readonly format: ClipboardFormat;
  readonly content: string;
  readonly filePaths: readonly string[];
  readonly updatedAt: number;
  readonly tagIds: readonly number[];
}

export interface ClipboardTag {
  readonly id: number;
  readonly name: string;
  readonly color: string;
}

export interface ClipboardTagInput {
  readonly name: string;
  readonly color: string;
}

export interface HistoryStats {
  readonly count: number;
  readonly storageBytes: number;
}

export type SystemType = "macos" | "windows" | "linux" | "other";

export interface SystemPermissionStatus {
  readonly systemType: SystemType;
  readonly systemName: string;
  readonly accessibilityPermissionGranted: boolean;
  readonly accessibilityPermissionSupported: boolean;
}
