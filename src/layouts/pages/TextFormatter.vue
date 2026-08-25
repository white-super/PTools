<script setup lang="ts">
import { useRoute } from "vue-router";
import TextFormatterWorkspace from "../features/text-formatter/TextFormatterWorkspace.vue";

const route = useRoute();
const windowId = typeof route.query.windowId === "string" ? route.query.windowId : "";
</script>

<template>
  <TextFormatterWorkspace :window-id="windowId" />
</template>
