<script setup lang="ts">
import { convertFileSrc } from "@tauri-apps/api/core";
import { computed, shallowRef } from "vue";
import CardHeader from "./CardHeader.vue";
import type { ClipboardFormat } from "../../types/settings";

const IMAGE_FILE_EXTENSIONS = ["avif", "gif", "heic", "heif", "jpeg", "jpg", "png", "tif", "tiff", "webp"];
const GENERIC_FILE_LABEL = "文件";
const TEXT_PREVIEW_CHARACTER_LIMIT = 2000;

interface Props {
  content: string;
  format: ClipboardFormat;
  isSelected: boolean;
  filePaths: readonly string[];
  quickKey?: number;
  isDiffSource?: boolean;
}

const props = defineProps<Props>();
// Only the card preview is shortened; paste and formatter actions use the full entry.
const textPreview = computed(() => props.content.length > TEXT_PREVIEW_CHARACTER_LIMIT
  ? `${props.content.slice(0, TEXT_PREVIEW_CHARACTER_LIMIT)}…` : props.content);
const emit = defineEmits<{
  select: [];
  paste: [];
  contextMenu: [event: MouseEvent];
  cancelDiff: [];
}>();

const imageSource = computed(() =>
  props.content.startsWith("data:")
    ? props.content
    : `data:image/png;base64,${props.content}`,
);
const firstFilePath = computed(() => props.filePaths[0]);
const fileName = computed(() => getFileName(firstFilePath.value));
const fileExtension = computed(() => getFileExtension(fileName.value));
const filePreviewFailed = shallowRef(false);
const fileThumbnailSource = computed(() => {
  const filePath = firstFilePath.value;
  if (!filePath || filePreviewFailed.value || !isImageFile(fileExtension.value)) {
    return undefined;
  }
  return convertFileSrc(filePath);
});
const fileTypeLabel = computed(() => fileExtension.value?.toUpperCase() ?? GENERIC_FILE_LABEL);
const additionalFileCount = computed(() => Math.max(0, props.filePaths.length - 1));

function handleDoubleClick() {
  emit("paste");
}

function getFileName(filePath?: string) {
  return filePath?.split(/[\\/]/).pop() ?? GENERIC_FILE_LABEL;
}

function getFileExtension(fileName: string) {
  const extensionPosition = fileName.lastIndexOf(".");
  if (extensionPosition <= 0 || extensionPosition === fileName.length - 1) {
    return undefined;
  }
  return fileName.slice(extensionPosition + 1).toLowerCase();
}

function isImageFile(extension?: string) {
  return extension !== undefined && IMAGE_FILE_EXTENSIONS.includes(extension);
}

function handleFilePreviewError() {
  filePreviewFailed.value = true;
}
</script>

<template>
  <div
    class="card"
    :class="{ 'card-selected': props.isSelected }"
    @click="emit('select')"
    @dblclick.prevent="handleDoubleClick"
    @contextmenu.prevent="emit('contextMenu', $event)"
  >
    <CardHeader
      :format="format"
      :quick-key="quickKey"
      :is-diff-source="isDiffSource"
      :is-selected="props.isSelected"
      @cancel-diff="emit('cancelDiff')"
    />
    <div v-if="props.format === 'image'" class="card-media">
      <img class="card-image" :src="imageSource" alt="剪贴板图片" draggable="false" loading="lazy" decoding="async" />
    </div>
    <div v-else-if="props.format === 'file'" class="file-content">
      <div class="file-preview">
        <img
          v-if="fileThumbnailSource"
          class="file-preview-image"
          :src="fileThumbnailSource"
          :alt="fileName"
          draggable="false"
          loading="lazy"
          decoding="async"
          @error="handleFilePreviewError"
        />
        <div v-else class="file-preview-placeholder">
          <svg class="file-preview-icon" viewBox="0 0 32 32" aria-hidden="true">
            <path d="M8 3.5h10l6 6v19H8zM18 3.5v6h6" fill="none" stroke="currentColor" stroke-width="2" />
          </svg>
          <span class="file-preview-type">{{ fileTypeLabel }}</span>
        </div>
      </div>
      <div class="file-details">
        <p class="file-name">{{ fileName }}</p>
        <p v-if="additionalFileCount" class="file-count">另有 {{ additionalFileCount }} 个文件</p>
      </div>
    </div>
    <p v-else class="card-text">{{ textPreview }}</p>
  </div>
</template>

<style scoped>
.card {
  position: relative;
  display: flex;
  box-sizing: border-box;
  height: 100%;
  overflow: hidden;
  flex-direction: column;
  border: 1px solid var(--panel-card-border, #e5e7eb);
  border-radius: 12px;
  padding: 12px;
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", "PingFang SC", "Microsoft YaHei", sans-serif;
  -webkit-font-smoothing: antialiased;
  color: var(--panel-card-text, #374151);
  background: var(--panel-card-background, #ffffff);
  box-shadow: var(--panel-card-shadow, 0 1px 2px rgba(15, 23, 42, 0.06));
  backdrop-filter: var(--panel-card-backdrop, none);
  -webkit-backdrop-filter: var(--panel-card-backdrop, none);
  cursor: default;
  --card-header-background-current: var(--panel-card-header-background);
  --card-header-border-current: var(--panel-card-header-border);
  --card-header-text-current: var(--panel-card-header-text);
  --card-header-overlay-current: var(--panel-card-header-overlay, transparent);
  --card-header-accent-current: var(--panel-card-header-accent, transparent);
  --card-header-overlay-opacity: 0;
  --card-header-key-background: var(--panel-card-header-key-background);
  --card-header-key-border: var(--panel-card-header-key-border);
  --card-header-key-text: var(--panel-card-header-key-text);
  --card-header-selected-key-background: var(--panel-card-header-selected-key-background);
  --card-header-selected-key-border: var(--panel-card-header-selected-key-border);
  --card-header-selected-key-text: var(--panel-card-header-selected-key-text);
  transition: border-color 220ms ease, box-shadow 220ms ease, transform 220ms cubic-bezier(0.22, 1, 0.36, 1), background-color 220ms ease;
  will-change: transform;
  user-select: none;
  -webkit-user-select: none;
}

.card :deep(*) {
  user-select: none;
  -webkit-user-select: none;
}

.card-text {
  display: -webkit-box;
  flex: 1;
  margin: 0;
  overflow: hidden;
  font-size: 12px;
  line-height: 1.6;
  white-space: pre-wrap;
  overflow-wrap: anywhere;
  text-overflow: ellipsis;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 12;
  user-select: none;
}

.card-media {
  display: flex;
  min-height: 0;
  flex: 1;
  align-items: center;
  justify-content: center;
  border-radius: 8px;
  background: var(--panel-card-preview-background, #f8fafc);
}

.card-image {
  max-width: 100%;
  max-height: 100%;
  object-fit: contain;
  border-radius: 6px;
  -webkit-user-drag: none;
}

.file-content {
  display: flex;
  min-height: 0;
  flex: 1;
  flex-direction: column;
  gap: 7px;
}

.file-preview {
  display: flex;
  min-height: 0;
  flex: 1;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  border-radius: 8px;
  background: var(--panel-card-preview-background, #f8fafc);
}

.file-preview-image {
  width: 100%;
  height: 100%;
  object-fit: contain;
  -webkit-user-drag: none;
}

.file-preview-placeholder {
  display: flex;
  align-items: center;
  flex-direction: column;
  gap: 5px;
  color: var(--panel-card-placeholder-text, #94a3b8);
}

.file-preview-icon {
  width: 38px;
  height: 38px;
}

.file-preview-type {
  color: var(--panel-card-chip-text, #64748b);
  font-size: 11px;
  font-weight: 650;
}

.file-details {
  flex: 0 0 auto;
}

.file-name,
.file-count {
  margin: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.file-name {
  color: var(--panel-card-file-name, #475569);
  font-size: 12px;
}

.file-count {
  margin-top: 2px;
  color: var(--panel-card-placeholder-text, #94a3b8);
  font-size: 11px;
}

.card:hover {
  --card-header-background-current: var(--panel-card-header-hover-background, var(--panel-card-header-background));
  --card-header-border-current: var(--panel-card-header-hover-border, var(--panel-card-header-border));
  --card-header-text-current: var(--panel-card-header-hover-text, var(--panel-card-header-text));
  --card-header-overlay-current: var(--panel-card-header-hover-overlay, var(--panel-card-header-overlay, transparent));
  --card-header-accent-current: var(--panel-card-header-hover-accent, transparent);
  --card-header-overlay-opacity: 1;
  border-color: var(--panel-card-hover-border, #cbd5e1);
  box-shadow: var(--panel-card-hover-shadow, 0 10px 24px rgba(15, 23, 42, 0.12));
  transform: translateY(-3px);
  z-index: 1;
}

.card-selected {
  --card-header-background-current: var(--panel-card-header-selected-background, var(--panel-card-header-background));
  --card-header-border-current: var(--panel-card-header-selected-border, var(--panel-card-header-border));
  --card-header-text-current: var(--panel-card-header-selected-text, var(--panel-card-header-text));
  --card-header-overlay-current: var(--panel-card-header-selected-overlay, var(--panel-card-header-overlay, transparent));
  --card-header-accent-current: var(--panel-card-header-selected-accent, var(--app-primary));
  --card-header-overlay-opacity: 1;
  border-color: var(--panel-card-selected-border, #93c5fd);
  background: var(--panel-card-selected-background, #f8fbff);
  box-shadow: var(--panel-card-selected-shadow, 0 0 0 2px rgba(147, 197, 253, 0.34), 0 8px 22px rgba(15, 23, 42, 0.1));
  transform: translateY(-2px);
  z-index: 1;
}

.card-selected:hover {
  --card-header-background-current: var(--panel-card-header-selected-hover-background, var(--panel-card-header-selected-background, var(--panel-card-header-background)));
  --card-header-border-current: var(--panel-card-header-selected-border, var(--panel-card-header-border));
  --card-header-text-current: var(--panel-card-header-selected-text, var(--panel-card-header-text));
  --card-header-overlay-current: var(--panel-card-header-selected-hover-overlay, var(--panel-card-header-selected-overlay, transparent));
  --card-header-accent-current: var(--panel-card-header-selected-accent, var(--app-primary));
  --card-header-overlay-opacity: 1;
  box-shadow: var(--panel-card-selected-hover-shadow, var(--panel-card-selected-shadow));
  transform: translateY(-4px);
  z-index: 2;
}

@media (prefers-reduced-motion: reduce) {
  .card {
    transition: border-color 0.01ms linear, box-shadow 0.01ms linear, background-color 0.01ms linear;
    transform: none;
  }

  .card:hover,
  .card-selected,
  .card-selected:hover {
    transform: none;
  }
}
</style>
