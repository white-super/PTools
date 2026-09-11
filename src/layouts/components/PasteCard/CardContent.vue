<script setup lang="ts">
import { convertFileSrc } from "@tauri-apps/api/core";
import { computed, shallowRef } from "vue";
import type { ClipboardFormat } from "../../types/settings";

const IMAGE_FILE_EXTENSIONS = ["avif", "gif", "heic", "heif", "jpeg", "jpg", "png", "tif", "tiff", "webp"];
const GENERIC_FILE_LABEL = "文件";
const TEXT_PREVIEW_CHARACTER_LIMIT = 2000;

const props = defineProps<{
  content: string;
  format: ClipboardFormat;
  filePaths: readonly string[];
}>();

const textPreview = computed(() => props.content.length > TEXT_PREVIEW_CHARACTER_LIMIT
  ? `${props.content.slice(0, TEXT_PREVIEW_CHARACTER_LIMIT)}…` : props.content);
const imageSource = computed(() => props.content.startsWith("data:")
  ? props.content : `data:image/png;base64,${props.content}`);
const firstFilePath = computed(() => props.filePaths[0]);
const fileName = computed(() => getFileName(firstFilePath.value));
const fileExtension = computed(() => getFileExtension(fileName.value));
const filePreviewFailed = shallowRef(false);
const fileThumbnailSource = computed(() => {
  const filePath = firstFilePath.value;
  if (!filePath || filePreviewFailed.value || !isImageFile(fileExtension.value)) return undefined;
  return convertFileSrc(filePath);
});
const fileTypeLabel = computed(() => fileExtension.value?.toUpperCase() ?? GENERIC_FILE_LABEL);
const additionalFileCount = computed(() => Math.max(0, props.filePaths.length - 1));

function getFileName(filePath?: string) {
  return filePath?.split(/[\\/]/).pop() ?? GENERIC_FILE_LABEL;
}

function getFileExtension(name: string) {
  const extensionPosition = name.lastIndexOf(".");
  if (extensionPosition <= 0 || extensionPosition === name.length - 1) return undefined;
  return name.slice(extensionPosition + 1).toLowerCase();
}

function isImageFile(extension?: string) {
  return extension !== undefined && IMAGE_FILE_EXTENSIONS.includes(extension);
}
</script>

<template>
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
        @error="filePreviewFailed = true"
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
</template>

<style scoped>
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

.file-preview-icon { width: 38px; height: 38px; }
.file-preview-type { color: var(--panel-card-chip-text, #64748b); font-size: 11px; font-weight: 650; }
.file-details { flex: 0 0 auto; }
.file-name, .file-count { margin: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.file-name { color: var(--panel-card-file-name, #475569); font-size: 12px; }
.file-count { margin-top: 2px; color: var(--panel-card-placeholder-text, #94a3b8); font-size: 11px; }
</style>
