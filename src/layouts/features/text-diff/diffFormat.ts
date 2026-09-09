import { formatJsonText, isJsonText } from "../text-formatter/formatters/jsonTools";
import type { DiffFormat, DiffRequest } from "./types";

export function detectDiffFormat(content: string): DiffFormat {
  if (isJsonText(content)) return "json";
  const value = content.trim();
  if (/^<!doctype\s+html\b|<(?:html|head|body|div|section|main|script|style)\b/i.test(value)) return "html";
  if (/^<\?xml\b|^<[A-Za-z][\w:.-]*(?:\s|>|\/)/i.test(value)) return "xml";
  return "plain";
}

async function formatContent(content: string, format: DiffFormat): Promise<string> {
  if (format === "json") return formatJsonText(content).trim();
  if (format === "plain") throw new Error("纯文本不支持格式化，请选择 JSON、XML 或 HTML");
  const prettier = await import("prettier/standalone");
  const plugin =
    format === "xml" ? await import("@prettier/plugin-xml") : await import("prettier/plugins/html");
  return prettier.format(content, {
    parser: format,
    plugins: [plugin.default],
    tabWidth: 2,
    htmlWhitespaceSensitivity: "strict",
    embeddedLanguageFormatting: "off",
    ...(format === "xml" ? { xmlWhitespaceSensitivity: "strict", xmlSortAttributesByKey: false } : {}),
  });
}

async function formatSide(content: string, format: DiffFormat, side: string) {
  try {
    return await formatContent(content, format);
  } catch (error) {
    throw new Error(`${side}格式化失败：${error instanceof Error ? error.message : String(error)}`);
  }
}

export async function prepareDiff(request: DiffRequest) {
  const leftFormat = detectDiffFormat(request.left);
  const rightFormat = detectDiffFormat(request.right);
  const format =
    request.format === "auto" ? (leftFormat === rightFormat ? leftFormat : "plain") : request.format;
  if (!request.formatted) return { left: request.left, right: request.right, format };
  if (format === "plain") throw new Error("双方不是同一种可格式化文本，请手动选择类型或关闭格式化对比");
  const left = await formatSide(request.left, format, "左侧");
  const right = await formatSide(request.right, format, "右侧");
  return { left, right, format };
}
