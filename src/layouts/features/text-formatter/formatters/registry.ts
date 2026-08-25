import type { TextFormat, TextFormatter } from "../types";
import { jsonFormatter } from "./jsonFormatter";

const FORMATTERS: readonly TextFormatter[] = [jsonFormatter];

export function detectTextFormat(content: string): TextFormat | undefined {
  return FORMATTERS.find((formatter) => formatter.matches(content))?.format;
}

export function getTextFormatter(format: TextFormat) {
  const formatter = FORMATTERS.find((entry) => entry.format === format);
  if (!formatter) {
    throw new Error(`暂不支持 ${format} 格式`);
  }
  return formatter;
}
