# Text Diff Browser Checks

Run the project's Vite development server, open its reported local URL in the managed browser, and evaluate:

```js
async () => {
  const checks = await import("/tests/browser/textDiff.browser.mjs");
  const editing = await import("/tests/browser/diffEditing.browser.mjs");
  return {
    workspace: await checks.runTextDiffBrowserChecks(),
    keyboard: await checks.runDiffKeyboardChecks(),
    worker: await checks.runDiffWorkerChecks(),
    cardHeader: await editing.runCardHeaderChecks(),
  };
}
```

Vite may reload the page when optimizing newly imported dependencies for the first time; run again once dependency optimization finishes.

These checks mount actual Vue components, CodeMirror editors, and Web Workers. The desktop IPC boundary is mocked: no real clipboard data, files, history, or native windows are modified. Native window focus, multi-monitor behavior, file dialogs, and global shortcuts still require macOS/Windows application testing.

Includes direct editing on both sides, live highlights, caret/focus retention, undo/redo, copying edited content, invalid JSON recovery, and unchanged card geometry when adding/removing the Diff icon.

Node regression tests: `node --test tests/*.test.mjs`.

For visual inspection, call `mountCardPreview("soft-glow")` from `diffEditing.browser.mjs` (also accepts `classic` or `dark`). It mounts real cards using synthetic sample content and returns a cleanup function. No clipboard data or files are read.
