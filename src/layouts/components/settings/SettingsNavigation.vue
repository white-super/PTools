<script setup lang="ts">
type SettingsTab = "system" | "updates";

const tabs = [
  { id: "system", label: "系统设置", icon: "⚙" },
  { id: "updates", label: "软件更新", icon: "↻" },
] as const;

const activeTab = defineModel<SettingsTab>({ required: true });
</script>

<template>
  <nav class="settings-navigation" aria-label="设置分类">
    <button
      v-for="tab in tabs"
      :key="tab.id"
      class="navigation-item"
      :class="{ 'is-active': activeTab === tab.id }"
      type="button"
      @click="activeTab = tab.id"
    >
      <span class="navigation-icon" aria-hidden="true">{{ tab.icon }}</span>
      <span>{{ tab.label }}</span>
    </button>
  </nav>
</template>

<style scoped>
.settings-navigation {
  display: grid;
  gap: 4px;
}

.navigation-item {
  display: flex;
  align-items: center;
  width: 100%;
  gap: 10px;
  border: 0;
  border-radius: 10px;
  padding: 12px 14px;
  color: var(--app-secondary);
  background: transparent;
  font: inherit;
  font-size: 15px;
  font-weight: 600;
  text-align: left;
  cursor: pointer;
}

.navigation-item:hover {
  background: var(--settings-navigation-hover);
}

.navigation-item.is-active {
  color: var(--app-heading);
  background: var(--settings-navigation-active);
}

.navigation-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  color: currentColor;
  font-size: 22px;
  line-height: 1;
}
</style>
