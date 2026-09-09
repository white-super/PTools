import type { DiffSource } from "./types";

export function decodeTextFile(bytes: ArrayBuffer): string {
  let content: string;
  try {
    content = new TextDecoder("utf-8", { fatal: true }).decode(bytes);
  } catch {
    throw new Error("文件不是 UTF-8 文本，请转换编码后重试");
  }
  if (/[\u0000-\u0008\u000b\u000c\u000e-\u001f\u007f-\u009f]/.test(content)) {
    throw new Error("文件包含二进制控制字符，无法进行文本对比");
  }
  return content;
}

export async function readTextFile(file: File): Promise<DiffSource> {
  return { name: file.name, content: decodeTextFile(await file.arrayBuffer()) };
}
