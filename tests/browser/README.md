# Text Diff Browser Checks

Run the project's Vite development server, open its reported local URL in the managed browser, and evaluate:

```js
async () => {
  const checks = await import("/tests/browser/textDiff.browser.mjs");
  const formatter = await import("/tests/browser/textFormatter.browser.mjs");
  const entry = await import("/tests/browser/textDiffEntry.browser.mjs");
  const dismissal = await import("/tests/browser/panelDismissal.browser.mjs");
  const cards = await import("/tests/browser/cardPreview.browser.mjs");
  const contextMenu = await import("/tests/browser/contextMenu.browser.mjs");
  const quickTools = await import("/tests/browser/quickTools.browser.mjs");
  const alignment = await import("/tests/browser/diffAlignment.browser.mjs");
  const sequentialPaste = await import("/tests/browser/sequentialPaste.browser.mjs");
  return {
    workspace: await checks.runTextDiffBrowserChecks(),
    diffEntry: await entry.runTextDiffEntryChecks(),
    panelDismissal: await dismissal.runPanelDismissalChecks(),
    formatterKeyboard: await formatter.runTextFormatterKeyboardChecks(),
    emptyFormatter: await formatter.runEmptyTextFormatterChecks(),
    keyboard: await checks.runDiffKeyboardChecks(),
    worker: await checks.runDiffWorkerChecks(),
    diffAlignment: await alignment.runDiffAlignmentChecks(),
    cardHeader: await cards.runCardHeaderChecks(),
    cardViewport: await cards.runCardViewportRecoveryChecks(),
    cardContextMenu: await contextMenu.runCardContextMenuChecks(),
    quickTools: await quickTools.runQuickToolsBrowserChecks(),
    quickToolLaunch: await quickTools.runQuickToolLaunchChecks(),
    sequentialPaste: await sequentialPaste.runSequentialPasteBrowserChecks(),
  };
}
```

Vite may reload the page when optimizing newly imported dependencies for the first time; run again once dependency optimization finishes.

These checks mount actual Vue components, CodeMirror editors, and Web Workers. The desktop IPC boundary is mocked: no real clipboard data, files, history, or native windows are modified. Native window focus, multi-monitor behavior, file dialogs, and global shortcuts still require macOS/Windows application testing.

Includes direct editing on both sides, live highlights, caret/focus retention, undo/redo, copying edited content, invalid JSON recovery, the pinned empty-comparison entry, unchanged card geometry when adding/removing Diff-L/DIFF-R markers, and virtual-card viewport recovery after the hidden main panel is shown.

Node regression tests: `node --test tests/*.test.mjs`.

For visual inspection, import `cardPreview.browser.mjs` and call `mountCardPreview("soft-glow")` (also accepts `classic` or `dark`). It mounts real cards using synthetic sample content and returns a cleanup function. No clipboard data or files are read.

Call `mountCardStatePreview("classic")` from the same preview module to compare the default, hovered, and selected card styles side by side. Hover the Diff-L marker on the middle card to activate its real hover state.

Import `contextMenu.browser.mjs` and call `mountCardContextMenuPreview("classic")` to inspect the card menu and its real tool/tag submenus.

Import `quickTools.browser.mjs` and call `mountQuickToolsPreview("classic")` to inspect the independent shortcut slots and tool manager.

Import `sequentialPaste.browser.mjs` and call `mountSequentialPastePreview("classic")` to inspect capture mode, or pass `"paste"` as the second argument to inspect paste mode.

Import `diffEditing.browser.mjs` and call `mountCardNoticePreview("classic")` to inspect card-anchored error notices without reading clipboard data.
