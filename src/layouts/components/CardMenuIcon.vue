<script setup lang="ts">
interface Props {
  name: "tools" | "tag" | "diff" | "trash";
}

const props = defineProps<Props>();
</script>

<template>
  <svg class="card-menu-icon" viewBox="0 0 20 20" fill="none" aria-hidden="true">
    <template v-if="props.name === 'tools'">
      <path d="m3.5 15.5 8.8-8.8 3 3-8.8 8.8h-3v-3Z" />
      <path d="M14.5 2.5v3M13 4h3M17.5 6v2M16.5 7h2" />
    </template>
    <template v-else-if="props.name === 'tag'">
      <path d="M3 4.5V9l7.2 7.2a1.8 1.8 0 0 0 2.6 0l3.4-3.4a1.8 1.8 0 0 0 0-2.6L9 3H4.5A1.5 1.5 0 0 0 3 4.5Z" />
      <circle cx="6.5" cy="6.5" r="1" />
    </template>
    <template v-else-if="props.name === 'diff'">
      <rect x="2.5" y="3" width="5.5" height="14" rx="1.5" />
      <rect x="12" y="3" width="5.5" height="14" rx="1.5" />
      <path d="M9.5 7.25h1M9.5 12.75h1" />
    </template>
    <template v-else>
      <path d="M3.5 5.5h13M7.5 2.75h5M5.5 5.5l.7 11.75h7.6l.7-11.75M8 8.5v5.75M12 8.5v5.75" />
    </template>
  </svg>
</template>

<style scoped>
.card-menu-icon {
  width: 18px;
  height: 18px;
  flex: 0 0 auto;
  stroke: currentColor;
  stroke-linecap: round;
  stroke-linejoin: round;
  stroke-width: 1.45;
}
</style>
