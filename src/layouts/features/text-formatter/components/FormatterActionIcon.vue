<script setup lang="ts">
import type { FormatterToolbarAction } from "../types";

defineProps<{ action: FormatterToolbarAction }>();

const ENCODE_PATH = "M12 3v12m-4-4 4 4 4-4M4 16v5h16v-5";
const DECODE_PATH = "M12 15V3m-4 4 4-4 4 4M4 16v5h16v-5";
const ICON_PATHS: Readonly<Record<FormatterToolbarAction, string>> = {
  format: "M4 5h16M8 10h12M8 15h9M4 20h16M3 10l3 2.5L3 15",
  "fold-all": "M5 3l7 6 7-6M5 21l7-6 7 6M4 12h16",
  "unfold-all": "M5 8l7-6 7 6M5 16l7 6 7-6M4 12h16",
  "remove-comments": "M8 5H4v14h4M16 5h4v14h-4M10 8l-2 8M16 8l-2 8M3 3l18 18",
  "minify-copy": "M9 3h11v14M4 7h12v14H4zM6 11h8M6 14h8M6 17h5",
  "escape-copy": "M9 3h11v14M4 7h12v14H4zM7 11l6 6M6 11h2M12 17h2",
  "to-xml-copy": "m7 6-5 6 5 6m10-12 5 6-5 6M14 4l-4 16",
  "to-typescript-copy": "M2 6h9M6.5 6v13M21 7c-1-2-7-2-7 2 0 4 7 2 7 6 0 4-6 4-8 2",
  "url-encode": ENCODE_PATH,
  "url-decode": DECODE_PATH,
  "base64-encode": ENCODE_PATH,
  "base64-decode": DECODE_PATH,
  "date-to-timestamp": "M9 3 7 21M17 3l-2 18M3 9h18M2 15h18",
  "timestamp-to-date": "M5 5h14v16H5zM8 3v4M16 3v4M5 10h14M9 14h2M14 14h1M9 17h2",
};
</script>

<template>
  <svg
    class="formatter-action-icon" viewBox="0 0 24 24" width="16" height="16"
    fill="none" stroke="currentColor" stroke-width="1.7"
    stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" focusable="false"
  >
    <path :d="ICON_PATHS[action]" />
  </svg>
</template>

<style scoped>
.formatter-action-icon {
  flex: 0 0 auto;
}
</style>
