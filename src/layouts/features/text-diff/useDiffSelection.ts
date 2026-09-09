import { shallowRef } from "vue";
import type { ClipboardHistoryEntry } from "../../types/settings";
import type { DiffInput, DiffSource } from "./types";

interface Options {
  readonly resolve: (card: ClipboardHistoryEntry) => Promise<DiffSource | undefined>;
  readonly launch: (input: DiffInput) => Promise<unknown>;
  readonly reportError: (error: unknown) => void;
}

export function useDiffSelection(options: Options) {
  const pending = shallowRef<{ readonly cardId: number; readonly source: DiffSource }>();
  const busy = shallowRef(false);
  let revision = 0;

  function cancel() {
    revision++;
    pending.value = undefined;
    busy.value = false;
  }

  async function select(card: ClipboardHistoryEntry) {
    if (busy.value) return;
    if (pending.value?.cardId === card.id) {
      cancel();
      return;
    }
    const currentRevision = ++revision;
    const left = pending.value;
    busy.value = true;
    try {
      const source = await options.resolve(card);
      if (revision !== currentRevision || !source) return;
      if (!left) {
        pending.value = { cardId: card.id, source };
        return;
      }
      await options.launch({ left: left.source, right: source });
      if (revision === currentRevision) pending.value = undefined;
    } catch (error) {
      if (revision === currentRevision) options.reportError(error);
    } finally {
      if (revision === currentRevision) busy.value = false;
    }
  }

  return { pending, busy, select, cancel };
}
