export type TextFormat = "json";

export type TextTransformAction =
  | "format"
  | "remove-comments"
  | "minify-copy"
  | "escape-copy"
  | "to-xml-copy"
  | "to-typescript-copy";

export type FormatterToolbarAction = TextTransformAction | "fold-all" | "unfold-all";

export interface TextFormatterInput {
  readonly format: TextFormat;
  readonly content: string;
  readonly updatedAt: number;
}

export interface TextTransformResult {
  readonly output: string;
  readonly effect: "replace" | "copy";
  readonly message: string;
}

export interface TextFormatter {
  readonly format: TextFormat;
  readonly title: string;
  readonly supportedActions: readonly TextTransformAction[];
  matches(content: string): boolean;
  prepare(content: string): string;
  transform(action: TextTransformAction, content: string): TextTransformResult;
}
