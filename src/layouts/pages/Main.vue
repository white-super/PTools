<script lang="ts" setup>
import BodyMode from "../modes/PasteFlowPanel.vue"
</script>

<template>
  <div class="main-panel">
    <BodyMode/>
  </div>
</template>

<style scoped>
:global(html),
:global(body),
:global(#app) {
  width: 100%;
  height: 100%;
  margin: 0;
}

.main-panel {
  display: flex;
  position: fixed;
  inset: 0;
  min-height: 0;
  overflow: hidden;
  background: var(--app-background);
}

.main-panel > :last-child {
  flex: 1;
  min-height: 0;
  min-width: 0;
}
</style>
