export type DiffFormat = "plain" | "json" | "xml" | "html";
export type DiffFormatChoice = "auto" | DiffFormat;
export type DiffSide = "left" | "right";

export interface DiffSource {
  readonly name: string;
  readonly content: string;
}

export interface DiffInput {
  readonly left: DiffSource;
  readonly right: DiffSource;
}

export interface DiffRequest {
  readonly left: string;
  readonly right: string;
  readonly format: DiffFormatChoice;
  readonly formatted: boolean;
}

export interface DiffChange {
  readonly fromA: number;
  readonly toA: number;
  readonly fromB: number;
  readonly toB: number;
}

export interface DiffResult {
  readonly left: string;
  readonly right: string;
  readonly format: DiffFormat;
  readonly changes: readonly DiffChange[];
}

export type DiffResponse = { readonly result: DiffResult } | { readonly error: string };
