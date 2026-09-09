<script setup lang="ts">
import type { DiffFormatChoice } from "../types";

const format = defineModel<DiffFormatChoice>("format", { required: true });
const formatted = defineModel<boolean>("formatted", { required: true });
const wrapping = defineModel<boolean>("wrapping", { required: true });
const collapse = defineModel<boolean>("collapse", { required: true });
defineProps<{ navigable: boolean; importing: boolean }>();
const emit = defineEmits<{ compare: []; navigate: [direction: -1 | 1]; swap: [] }>();
</script>

<template>
  <div class="diff-toolbar" role="toolbar" aria-label="文本对比工具">
    <select v-model="format" aria-label="对比文本类型" title="选择两侧共同的文本类型">
      <option value="auto">自动识别</option><option value="plain">纯文本</option>
      <option value="json">JSON</option><option value="xml">XML</option><option value="html">HTML</option>
    </select>
    <label class="switch" title="仅格式化比较副本，不修改原始内容；XML/HTML 保留有意义的空白">
      <input v-model="formatted" type="checkbox" role="switch" /><span class="track"></span>格式化对比
    </label>
    <button type="button" title="重新对比 (Ctrl+R)" aria-label="重新对比" @click="emit('compare')">↻ <kbd>Ctrl+R</kbd></button>
    <button type="button" title="上一处差异 (Ctrl+Q)" aria-label="上一处差异" :disabled="!navigable" @click="emit('navigate', -1)">↑ <kbd>Ctrl+Q</kbd></button>
    <button type="button" title="下一处差异 (Ctrl+E)" aria-label="下一处差异" :disabled="!navigable" @click="emit('navigate', 1)">↓ <kbd>Ctrl+E</kbd></button>
    <button type="button" title="交换左右内容" aria-label="交换左右内容" :disabled="importing" @click="emit('swap')">⇄</button>
    <details class="more">
      <summary aria-label="更多对比选项" title="更多选项">•••</summary>
      <div class="more-menu">
        <label class="switch"><input v-model="wrapping" type="checkbox" role="switch" /><span class="track"></span>自动换行</label>
        <label class="switch"><input v-model="collapse" type="checkbox" role="switch" /><span class="track"></span>折叠相同区域</label>
        <small>左右同步滚动<br />本地计算，不上传内容</small>
      </div>
    </details>
  </div>
</template>

<style scoped>
.diff-toolbar { display: flex; align-items: center; flex-wrap: nowrap; gap: 6px; padding: 4px 10px; border-bottom: 1px solid var(--app-border); background: var(--app-surface); font-size: 12px; white-space: nowrap; }
button, select, summary { border: 0; border-radius: 5px; padding: 5px 6px; background: transparent; color: var(--app-text); font: inherit; cursor: pointer; }
button { display: inline-flex; align-items: center; gap: 4px; }
button:hover, summary:hover { background: var(--app-active); }
button:disabled { opacity: .45; cursor: default; }
button:focus-visible, summary:focus-visible, select:focus-visible { outline: 2px solid var(--app-primary); }
select, option { background: var(--app-menu-background); }
kbd { font: inherit; font-size: 10px; padding: 1px 3px; border: 1px solid var(--app-border); border-radius: 4px; color: var(--app-muted); }
.switch { display: inline-flex; align-items: center; gap: 5px; cursor: pointer; }
.switch input { position: absolute; opacity: 0; width: 1px; height: 1px; }
.track { box-sizing: border-box; width: 26px; height: 15px; border-radius: 10px; background: var(--app-border); padding: 2px; }
.track::after { content: ""; display: block; width: 11px; height: 11px; border-radius: 50%; background: var(--app-menu-background); transition: transform .12s; }
input:checked + .track { background: var(--app-primary); }
input:checked + .track::after { transform: translateX(11px); }
input:focus-visible + .track { outline: 2px solid var(--app-primary); outline-offset: 2px; }
.more { position: relative; margin-left: auto; }
summary { list-style: none; }
summary::-webkit-details-marker { display: none; }
.more-menu { position: absolute; z-index: 20; right: 0; top: 100%; display: grid; gap: 12px; padding: 12px; background: var(--app-menu-background); border: 1px solid var(--app-border); border-radius: 8px; box-shadow: var(--app-shadow); }
small { color: var(--app-muted); line-height: 1.6; }
</style>
