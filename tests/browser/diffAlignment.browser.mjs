import { createApp, nextTick } from "vue";
import TextDiffWorkspace from "../../src/layouts/features/text-diff/TextDiffWorkspace.vue";

const LONG_DESCRIPTION_LENGTH = 180;
const LONG_SCRIPT_LENGTH = 260;
const EXPECTED_CHANGED_LINES = 2;

function assert(value, message) {
  if (!value) throw new Error(message);
}

async function waitFor(predicate, message) {
  const deadline = performance.now() + 10000;
  while (!predicate()) {
    if (performance.now() > deadline) throw new Error(message);
    await new Promise((resolve) => setTimeout(resolve, 25));
  }
}

function comparisonInput() {
  const sharedBlock = [
    '  "parameters": {',
    '    "type": "object",',
    '    "properties": {',
    '      "question": { "type": "string" }',
    '    },',
    '    "method": "GET"',
    '  },',
  ].join("\n");
  const content = (description, script) => [
    "{",
    `  "description":"${description}",`,
    sharedBlock,
    `  "responseParser":"${script}"`,
    "}",
  ].join("\n");
  return {
    left: { name: "left.json", content: content("左".repeat(LONG_DESCRIPTION_LENGTH), "A".repeat(LONG_SCRIPT_LENGTH)) },
    right: { name: "right.json", content: content("右".repeat(LONG_DESCRIPTION_LENGTH), "B".repeat(LONG_SCRIPT_LENGTH)) },
  };
}

function installNativeMock(input) {
  window.__TAURI_INTERNALS__ = {
    metadata: { currentWindow: { label: "text-diff-alignment-test" } },
    transformCallback: () => 1,
    invoke: async (command) => {
      if (command === "get_text_diff_input") return structuredClone(input);
      if (command === "get_text_diff_pinned") return false;
      if (command === "plugin:event|listen") return 1;
      if (command === "plugin:event|unlisten") return;
      throw new Error(`Unexpected native call: ${command}`);
    },
  };
}

function assertChangedLines(editor) {
  const lines = [...editor.querySelectorAll(".cm-changedLine")].map((line) => line.textContent);
  assert(lines.length === EXPECTED_CHANGED_LINES, `expected two changed lines, received ${lines.length}`);
  assert(lines.every((line) => line.includes("description") || line.includes("responseParser")),
    "unchanged structured lines are highlighted");
  return lines;
}

export async function runDiffAlignmentChecks() {
  const previous = window.__TAURI_INTERNALS__;
  const host = document.createElement("div");
  host.style.cssText = "position:fixed;inset:0;width:1000px;height:650px;z-index:999";
  document.body.append(host);
  installNativeMock(comparisonInput());
  const app = createApp(TextDiffWorkspace);
  try {
    app.mount(host);
    await waitFor(() => host.querySelectorAll(".cm-editor").length === 2, "diff editors did not mount");
    await waitFor(() => host.querySelectorAll(".cm-changedLine").length > 0, "diff highlights did not render");
    const changedLines = [...host.querySelectorAll(".cm-editor")].map(assertChangedLines);
    return { passed: true, changedLines };
  } finally {
    app.unmount();
    host.remove();
    await nextTick();
    window.__TAURI_INTERNALS__ = previous;
  }
}
