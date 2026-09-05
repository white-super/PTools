import type { TextFormatter, TextTransformAction } from "../types";
import { dateToTimestamp, formatDateValue, replaceResult, timestampToDate } from "./textFormatterUtils";

const SUPPORTED_ACTIONS: readonly TextTransformAction[] = ["format", "date-to-timestamp", "timestamp-to-date"];

export const dateFormatter: TextFormatter = {
  format: "date",
  title: "日期格式化",
  editorLanguage: "plain",
  supportedActions: SUPPORTED_ACTIONS,
  matches: isDateText,
  prepare: formatDateValue,
  transform: transformDate,
};

function transformDate(action: TextTransformAction, content: string) {
  if (action === "format") return replaceResult(formatDateValue(content), "已格式化日期");
  if (action === "date-to-timestamp") return replaceResult(dateToTimestamp(content), "已转换为时间戳");
  if (action === "timestamp-to-date") return replaceResult(timestampToDate(content), "已转换为 ISO 日期");
  throw new Error("日期不支持此操作");
}

function isDateText(content: string) {
  const value = content.trim();
  if (/^\d{10,13}$/.test(value)) return true;
  return /^\d{4}-\d{2}-\d{2}(?:$|[T\s]\d{2}:\d{2})/.test(value);
}
