<script setup lang="ts">
import { convertFileSrc } from "@tauri-apps/api/core";
import { computed, shallowRef } from "vue";
import type { ClipboardFormat } from "../../types/settings";

const IMAGE_FILE_EXTENSIONS = ["avif", "gif", "heic", "heif", "jpeg", "jpg", "png", "tif", "tiff", "webp"];
const GENERIC_FILE_LABEL = "文件";

interface Props {
  content: string;
  format: ClipboardFormat;
  isSelected: boolean;
  filePaths: readonly string[];
  quickKey?: number;
}

const FORMAT_LABELS: Record<ClipboardFormat, string> = {
  text: "文本",
  image: "图片",
  file: "文件",
};

const props = defineProps<Props>();
const emit = defineEmits<{
  select: [];
  paste: [];
  contextMenu: [event: MouseEvent];
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
    <div class="card-header">
      <span v-if="props.quickKey" class="quick-key">{{ props.quickKey }}</span>
      <div class="card-badges">
        <span class="format-label">{{ FORMAT_LABELS[props.format] }}</span>
      </div>
    </div>
    <div v-if="props.format === 'image'" class="card-media">
      <img class="card-image" :src="imageSource" alt="剪贴板图片" draggable="false" decoding="async" />
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
    <p v-else class="card-text">{{ props.content }}</p>
  </div>
</template>

<style scoped>
.card {
  display: flex;
  box-sizing: border-box;
  height: 100%;
  overflow: hidden;
  flex-direction: column;
  border: 1px solid var(--panel-card-border, #e5e7eb);
  border-radius: 12px;
  padding: 10px;
  color: var(--panel-card-text, #374151);
  background: var(--panel-card-background, #ffffff);
  box-shadow: var(--panel-card-shadow, 0 1px 2px rgba(15, 23, 42, 0.06));
  backdrop-filter: var(--panel-card-backdrop, none);
  -webkit-backdrop-filter: var(--panel-card-backdrop, none);
  cursor: default;
  transition: border-color 0.16s ease, box-shadow 0.16s ease, transform 0.16s ease;
  user-select: none;
  -webkit-user-select: none;
}

.card :deep(*) {
  user-select: none;
  -webkit-user-select: none;
}

.card-header {
  display: flex;
  flex: 0 0 auto;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 6px;
}

.card-badges {
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.quick-key {
  display: inline-flex;
  width: 18px;
  height: 18px;
  align-items: center;
  justify-content: center;
  border-radius: 5px;
  color: var(--panel-card-chip-text, #64748b);
  background: var(--panel-card-chip-background, #f1f5f9);
  font-size: 11px;
  font-weight: 650;
}

.format-label {
  border-radius: 4px;
  padding: 2px 5px;
  color: var(--panel-card-chip-text, #64748b);
  background: var(--panel-card-chip-background, #f1f5f9);
  font-size: 11px;
}

.card-text {
  display: -webkit-box;
  flex: 1;
  margin: 0;
  overflow: hidden;
  font-size: 12px;
  line-height: 1.45;
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
  border-color: var(--panel-card-hover-border, #cbd5e1);
  box-shadow: var(--panel-card-hover-shadow, 0 7px 18px rgba(15, 23, 42, 0.08));
  transform: translateY(-1px);
}

.card-selected {
  border-color: var(--panel-card-selected-border, #93c5fd);
  background: var(--panel-card-selected-background, #f8fbff);
  box-shadow: var(--panel-card-selected-shadow, 0 0 0 2px rgba(147, 197, 253, 0.34));
}
</style>
