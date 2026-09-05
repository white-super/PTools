export type TextFormat = "json" | "xml" | "html" | "url" | "base64" | "date";

export type TextTransformAction =
  | "format"
  | "remove-comments"
  | "minify-copy"
  | "escape-copy"
  | "to-xml-copy"
  | "to-typescript-copy"
  | "url-encode"
  | "url-decode"
  | "base64-encode"
  | "base64-decode"
  | "date-to-timestamp"
  | "timestamp-to-date";

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
  readonly editorLanguage: "json" | "plain";
  readonly supportedActions: readonly TextTransformAction[];
  matches(content: string): boolean;
  prepare(content: string): string;
  transform(action: TextTransformAction, content: string): TextTransformResult;
}

export type TextFormatterOption = Pick<TextFormatter, "format" | "title">;
