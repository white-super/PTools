<script setup lang="ts">
import type { PanelNoticeState } from "../../composables/usePanelNotice";

defineProps<{ notice?: PanelNoticeState }>();
</script>

<template>
  <Transition name="card-notice">
    <div
      v-if="notice"
      class="card-notice"
      role="alert"
      :title="notice.message"
    >
      <svg viewBox="0 0 16 16" aria-hidden="true">
        <circle cx="8" cy="8" r="6.25" fill="none" stroke="currentColor" stroke-width="1.5" />
        <path d="M8 4.5v4M8 11.25h.01" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
      </svg>
      <span class="card-notice-message">{{ notice.message }}</span>
    </div>
  </Transition>
</template>

<style scoped>
.card-notice {
  display: flex;
  box-sizing: border-box;
  width: 100%;
  min-width: 0;
  align-items: flex-start;
  gap: 7px;
  border: 1px solid color-mix(in srgb, var(--app-danger) 32%, transparent);
  border-radius: 9px;
  padding: 7px 9px;
  color: var(--app-danger);
  background: color-mix(in srgb, var(--app-menu-background) 91%, var(--app-danger));
  box-shadow: 0 6px 16px rgba(25, 35, 45, 0.16);
  font-size: 12px;
  line-height: 1.35;
}

.card-notice svg { width: 14px; height: 14px; flex: 0 0 14px; margin-top: 1px; }

.card-notice-message {
  display: -webkit-box;
  min-width: 0;
  overflow: hidden;
  overflow-wrap: anywhere;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 2;
}

.card-notice-enter-active,
.card-notice-leave-active { transition: opacity 140ms ease, transform 140ms ease; }
.card-notice-enter-from,
.card-notice-leave-to { opacity: 0; transform: translateY(4px); }

@media (prefers-reduced-motion: reduce) {
  .card-notice-enter-active,
  .card-notice-leave-active { transition: none; }
}
</style>
