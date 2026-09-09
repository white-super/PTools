import { readFile } from "node:fs/promises";
import ts from "typescript";

// Resolve source imports in memory; tests exercise the actual implementation without emitted files.
export async function loadTypeScript(url) {
  const source = await readFile(url, "utf8");
  let { outputText } = ts.transpileModule(source, {
    compilerOptions: { module: ts.ModuleKind.ESNext, target: ts.ScriptTarget.ES2022 },
  });
  const imports = [...outputText.matchAll(/(?:from\s+|import\()(["'])([^"']+)\1/g)];
  for (const [, quote, specifier] of imports) {
    const resolved = specifier.startsWith(".")
      ? await sourceUrl(new URL(specifier + ".ts", url))
      : import.meta.resolve(specifier);
    outputText = outputText.replaceAll(`${quote}${specifier}${quote}`, JSON.stringify(resolved));
  }
  return outputText + `\n//# sourceURL=${url.pathname}\n`;
}

export async function sourceUrl(url) {
  return `data:text/javascript;base64,${Buffer.from(await loadTypeScript(url)).toString("base64")}`;
}
