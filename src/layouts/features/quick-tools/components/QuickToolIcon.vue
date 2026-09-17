<script setup lang="ts">
import type { QuickToolId } from "../types";

defineProps<{ toolId: QuickToolId }>();
</script>

<template>
  <svg class="quick-tool-icon" viewBox="0 0 24 24" aria-hidden="true">
    <g v-if="toolId === 'text-diff'">
      <rect x="3" y="4" width="6.5" height="16" rx="1.5" />
      <rect x="14.5" y="4" width="6.5" height="16" rx="1.5" />
      <path d="M11.25 8h1.5M11.25 16h1.5" />
    </g>
    <path v-else-if="toolId === 'json'" d="M9 3.5H7.8c-1.5 0-2.1.7-2.1 2.2v3.1c0 1.5-.6 2.2-1.8 2.2 1.2 0 1.8.7 1.8 2.2v3.1c0 1.5.6 2.2 2.1 2.2H9M15 3.5h1.2c1.5 0 2.1.7 2.1 2.2v3.1c0 1.5.6 2.2 1.8 2.2-1.2 0-1.8.7-1.8 2.2v3.1c0 1.5-.6 2.2-2.1 2.2H15" />
    <path v-else-if="toolId === 'xml'" d="m8.5 7-5 5 5 5M15.5 7l5 5-5 5M13.8 4.5 10.2 19.5" />
    <g v-else-if="toolId === 'html'">
      <rect x="3" y="4" width="18" height="16" rx="2" />
      <path d="M3 8h18M9.5 11l-3 2.5 3 2.5M14.5 11l3 2.5-3 2.5" />
    </g>
    <path v-else-if="toolId === 'url'" d="m9.5 14.5 5-5M7.2 16.8l-1 .9a3.2 3.2 0 0 1-4.5-4.5l3.1-3.1a3.2 3.2 0 0 1 4.5 0M16.8 7.2l1-.9a3.2 3.2 0 0 1 4.5 4.5l-3.1 3.1a3.2 3.2 0 0 1-4.5 0" />
    <g v-else-if="toolId === 'base64'">
      <path d="M5 5.5h5v5H5zM14 13.5h5v5h-5z" />
      <path d="M14.5 6h4M16.5 4v4M5.5 16h4M7.5 14v4" />
    </g>
    <g v-else>
      <rect x="3.5" y="5" width="17" height="15" rx="2" />
      <path d="M7.5 3.5v3M16.5 3.5v3M3.5 9h17M12 12v3.5l2.5 1.5" />
    </g>
  </svg>
</template>

<style scoped>
.quick-tool-icon {
  width: 15px;
  height: 15px;
  fill: none;
  stroke: currentColor;
  stroke-linecap: round;
  stroke-linejoin: round;
  stroke-width: 1.55;
}
</style>
