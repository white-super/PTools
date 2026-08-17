<script setup lang="ts">
import { useTemplateRef } from "vue";

interface Props {
  readonly open: boolean;
}

const props = defineProps<Props>();
const emit = defineEmits<{ openChange: [open: boolean] }>();
const helpButton = useTemplateRef<HTMLButtonElement>("helpButton");

function close() {
  if (props.open) {
    emit("openChange", false);
  }
  helpButton.value?.blur();
}

function toggle() {
  if (props.open) {
    close();
    return;
  }
  emit("openChange", true);
}

defineExpose({ close });
</script>

<template>
  <div class="shortcut-help" @click.stop>
    <button
      ref="helpButton"
      type="button"
      class="help-button"
      aria-label="查看快捷键帮助"
      title="快捷键帮助"
      :aria-expanded="props.open"
      @pointerdown.prevent
      @click="toggle"
    >
      ?
    </button>
  </div>
</template>

<style scoped>
.shortcut-help {
  position: relative;
  flex: 0 0 auto;
}

.help-button {
  border: 0;
  width: 24px;
  height: 26px;
  border-radius: 6px;
  outline: none;
  padding: 0;
  color: var(--panel-filter-text);
  background: transparent;
  font: inherit;
  font-size: 14px;
  font-weight: 700;
  cursor: default;
}

.help-button:hover,
.help-button[aria-expanded="true"] {
  color: var(--panel-filter-hover-text);
  background: var(--panel-control-hover);
}
</style>
