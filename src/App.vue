<script setup lang="ts">
import { useAppTheme } from "./layouts/composables/useAppTheme";

useAppTheme();
</script>

<template>
  <router-view />
</template>
