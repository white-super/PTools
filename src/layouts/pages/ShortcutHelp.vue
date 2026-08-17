<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { onMounted, onUnmounted, shallowRef } from "vue";
import ShortcutHelpContent from "../components/ShortcutHelpContent.vue";
import type { AppSettings } from "../types/settings";

const SETTINGS_UPDATED_EVENT = "app-settings-updated";
const settings = shallowRef<AppSettings>();
let disposed = false;
let unlistenSettings: UnlistenFn | undefined;

function reportHelpError(error: unknown) {
  console.error("Failed to operate shortcut help window", error);
}

async function initializeSettings() {
  unlistenSettings = await listen<AppSettings>(SETTINGS_UPDATED_EVENT, (event) => {
    settings.value = event.payload;
  });
  if (disposed) {
    unlistenSettings();
    return;
  }
  settings.value = await invoke<AppSettings>("get_app_settings");
}

function closeHelp() {
  void invoke("hide_shortcut_help").catch(reportHelpError);
}

function closeOnEscape(event: KeyboardEvent) {
  if (event.key === "Escape") {
    event.preventDefault();
    closeHelp();
  }
}

onMounted(() => {
  window.addEventListener("keydown", closeOnEscape);
  void initializeSettings().catch(reportHelpError);
});

onUnmounted(() => {
  disposed = true;
  window.removeEventListener("keydown", closeOnEscape);
  unlistenSettings?.();
});
</script>

<template>
  <main class="shortcut-help-page">
    <ShortcutHelpContent :shortcut-settings="settings" @close="closeHelp" />
  </main>
</template>

<style scoped>
:global(html),
:global(body),
:global(#app) {
  background: transparent;
}

.shortcut-help-page {
  display: flex;
  box-sizing: border-box;
  width: 100%;
  height: 100%;
  padding: 10px;
  background: transparent;
}
</style>
