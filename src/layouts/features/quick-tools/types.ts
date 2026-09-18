import type { TextFormat } from "../text-formatter/types";

export const QUICK_TOOL_IDS = [
  "text-diff",
  "sequential-paste",
  "json",
  "xml",
  "html",
  "url",
  "base64",
  "date",
] as const;

export type QuickToolId = typeof QUICK_TOOL_IDS[number];
export type QuickToolTone = "blue" | "purple" | "orange" | "cyan" | "green";

export type QuickToolLaunch =
  | { readonly kind: "text-diff" }
  | { readonly kind: "sequential-paste" }
  | { readonly kind: "formatter"; readonly format: TextFormat };

export interface QuickToolDefinition {
  readonly id: QuickToolId;
  readonly label: string;
  readonly tone: QuickToolTone;
  readonly launch: QuickToolLaunch;
}
