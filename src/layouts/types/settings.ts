export type ClipboardFormat = "text" | "image" | "file";
export type ClipboardFilter = "all" | ClipboardFormat | number;
export type AppTheme = "soft-glow" | "classic" | "dark";

export interface AppSettings {
  readonly theme: AppTheme;
  readonly mainShortcut: string;
  readonly previousFilterShortcut: string;
  readonly nextFilterShortcut: string;
  readonly previousCardShortcut: string;
  readonly nextCardShortcut: string;
  readonly showFormatFilters: boolean;
  readonly historyRetentionDays: number;
  readonly maxHistoryEntries: number;
  readonly recordText: boolean;
  readonly recordImages: boolean;
  readonly recordFiles: boolean;
  readonly autoPaste: boolean;
}

export type PasteFlowShortcutSettings = Pick<
  AppSettings,
  "mainShortcut" | "previousFilterShortcut" | "nextFilterShortcut" | "previousCardShortcut" | "nextCardShortcut"
>;

export interface ClipboardHistoryEntry {
  readonly id: number;
  readonly format: ClipboardFormat;
  readonly content: string;
  readonly filePaths: readonly string[];
  readonly updatedAt: number;
  readonly tagIds: readonly number[];
}

export interface ClipboardHistoryQuery {
  readonly search: string;
  readonly format?: ClipboardFormat;
  readonly tagId?: number;
}

export interface ClipboardHistoryPage {
  readonly entries: ClipboardHistoryEntry[];
  readonly hasMore: boolean;
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
