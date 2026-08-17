<script setup lang="ts">
import { computed } from "vue";
import { APP_THEME_OPTIONS } from "../../constants/appThemes";
import type { AppSettings, AppTheme } from "../../types/settings";

const settings = defineModel<AppSettings>({ required: true });

const theme = computed({
  get: () => settings.value.theme,
  set: (theme: AppTheme) => {
    settings.value = { ...settings.value, theme };
  },
});
</script>

<template>
  <section class="settings-section theme-settings-section">
    <div class="theme-row">
      <div class="setting-copy">
        <h2 class="setting-title">主题配色</h2>
        <p class="setting-description">选择应用的整体颜色，切换后自动生效。</p>
      </div>
      <div class="theme-options" role="radiogroup" aria-label="主题配色">
        <button
          v-for="option in APP_THEME_OPTIONS"
          :key="option.value"
          type="button"
          class="theme-option"
          :class="{ 'theme-option-active': theme === option.value }"
          role="radio"
          :aria-checked="theme === option.value"
          @click="theme = option.value"
        >
          <span class="theme-preview" :style="{ background: option.previewBackground }"></span>
          <span class="theme-option-copy">
            <span class="theme-option-name">{{ option.label }}</span>
            <span class="theme-option-description">{{ option.description }}</span>
          </span>
          <span v-if="theme === option.value" class="active-indicator" aria-hidden="true">✓</span>
        </button>
      </div>
    </div>
  </section>
</template>

<style scoped>
.theme-settings-section {
  min-width: 0;
}

.theme-row {
  display: flex;
  width: 100%;
  align-items: center;
  justify-content: space-between;
  gap: 24px;
}

.setting-copy {
  min-width: 0;
}

.setting-title,
.setting-description {
  margin: 0;
}

.setting-title {
  color: var(--app-heading);
  font-size: 15px;
  font-weight: 650;
}

.setting-description {
  margin-top: 6px;
  color: var(--app-muted);
  font-size: 13px;
}

.theme-options {
  display: grid;
  width: min(520px, 100%);
  flex: 0 0 auto;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 10px;
}

.theme-option {
  display: grid;
  position: relative;
  min-width: 0;
  appearance: none;
  border: 1px solid var(--settings-option-border);
  border-radius: 10px;
  padding: 7px;
  color: var(--app-text);
  background: var(--settings-option-background);
  font: inherit;
  text-align: left;
  outline: none;
  cursor: default;
  transition: border-color 0.16s ease, box-shadow 0.16s ease;
  -webkit-appearance: none;
}

.theme-option:hover {
  border-color: var(--settings-option-hover-border);
}

.theme-option-active {
  border-color: var(--app-primary);
  box-shadow: 0 0 0 2px var(--app-primary-ring);
}

.theme-preview {
  height: 38px;
  border: 1px solid var(--app-border);
  border-radius: 7px;
}

.theme-option-copy {
  display: grid;
  gap: 2px;
  padding: 7px 2px 1px;
}

.theme-option-name {
  font-size: 12px;
  font-weight: 650;
}

.theme-option-description {
  overflow: hidden;
  color: var(--app-muted);
  font-size: 10px;
  line-height: 1.35;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.active-indicator {
  display: flex;
  position: absolute;
  top: 11px;
  right: 11px;
  width: 16px;
  height: 16px;
  align-items: center;
  justify-content: center;
  border-radius: 999px;
  color: var(--app-on-primary);
  background: var(--app-primary);
  font-size: 10px;
  font-weight: 700;
}

@media (max-width: 760px) {
  .theme-row {
    align-items: flex-start;
    flex-direction: column;
    gap: 14px;
  }

  .theme-options {
    width: 100%;
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}
</style>
