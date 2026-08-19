import { spawn } from "node:child_process";
import process from "node:process";
import { fileURLToPath } from "node:url";
import { dirname, resolve } from "node:path";

const args = process.argv.slice(2);
const isMacOS = process.platform === "darwin";
const command = args[0];

// The custom runner creates and signs the temporary app bundle needed by macOS
// development. Other platforms can invoke the Tauri CLI directly.
if (isMacOS && command === "dev") {
  const hasRunner = args.some(
    (argument) =>
      argument === "-r" ||
      argument === "--runner" ||
      argument.startsWith("--runner=")
  );

  if (!hasRunner) {
    const scriptDirectory = dirname(fileURLToPath(import.meta.url));
    args.push("--runner", resolve(scriptDirectory, "run-macos-dev.sh"));
  }
}

const tauriCommand = process.platform === "win32" ? "tauri.cmd" : "tauri";
const child = spawn(tauriCommand, args, {
  stdio: "inherit",
  shell: process.platform === "win32",
});

child.on("error", (error) => {
  console.error(`Failed to start Tauri CLI: ${error.message}`);
  process.exitCode = 1;
});

child.on("exit", (code, signal) => {
  if (signal) {
    process.kill(process.pid, signal);
  } else {
    process.exitCode = code ?? 1;
  }
});
