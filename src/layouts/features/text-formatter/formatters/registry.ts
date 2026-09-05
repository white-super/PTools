import type { TextFormat, TextFormatter, TextFormatterOption } from "../types";
import { base64Formatter } from "./base64Formatter";
import { dateFormatter } from "./dateFormatter";
import { htmlFormatter } from "./htmlFormatter";
import { jsonFormatter } from "./jsonFormatter";
import { urlFormatter } from "./urlFormatter";
import { xmlFormatter } from "./xmlFormatter";

const FORMATTERS: readonly TextFormatter[] = [
  jsonFormatter,
  htmlFormatter,
  xmlFormatter,
  urlFormatter,
  dateFormatter,
  base64Formatter,
];

export function detectTextFormat(content: string): TextFormat | undefined {
  return FORMATTERS.find((formatter) => formatter.matches(content))?.format;
}

export function getTextFormatterOptions(): readonly TextFormatterOption[] {
  return FORMATTERS.map(({ format, title }) => ({ format, title }));
}

export function getTextFormatter(format: TextFormat) {
  const formatter = FORMATTERS.find((entry) => entry.format === format);
  if (!formatter) {
    throw new Error(`暂不支持 ${format} 格式`);
  }
  return formatter;
}
