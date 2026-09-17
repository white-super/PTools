<script setup lang="ts">
import ClipboardMoreMenu from "./ClipboardMoreMenu.vue";
import QuickToolsBar from "../features/quick-tools/components/QuickToolsBar.vue";
import type { QuickToolId } from "../features/quick-tools/types";

interface Props {
  readonly toolIds: readonly QuickToolId[];
  readonly toolShortcuts: readonly string[];
  readonly activeToolIds: readonly QuickToolId[];
  readonly savingToolOrder: boolean;
}

const props = defineProps<Props>();
const moreMenuOpen = defineModel<boolean>({ required: true });
const toolManagerOpen = defineModel<boolean>("toolManagerOpen", { required: true });
const emit = defineEmits<{
  settings: [];
  executeTool: [toolId: QuickToolId];
  toolOrderChange: [toolIds: readonly QuickToolId[]];
  toolError: [error: unknown];
}>();
</script>

<template>
  <div class="toolbar-actions">
    <div class="quick-tools-position">
      <QuickToolsBar
        v-model="toolManagerOpen"
        :tool-ids="props.toolIds"
        :tool-shortcuts="props.toolShortcuts"
        :active-tool-ids="props.activeToolIds"
        :saving="props.savingToolOrder"
        @execute="emit('executeTool', $event)"
        @order-change="emit('toolOrderChange', $event)"
        @error="emit('toolError', $event)"
      />
    </div>
    <div class="more-menu-position">
      <ClipboardMoreMenu v-model="moreMenuOpen" @settings="emit('settings')" />
    </div>
  </div>
</template>

<style scoped>
.toolbar-actions {
  display: grid;
  width: 100%;
  min-width: 0;
  grid-column: 3;
  grid-template-columns: minmax(0, 1fr) auto minmax(28px, 1fr);
  align-items: flex-start;
  background: transparent;
  box-shadow: none;
}

.quick-tools-position {
  grid-column: 2;
}

.more-menu-position {
  display: flex;
  grid-column: 3;
  justify-content: flex-end;
}
</style>
