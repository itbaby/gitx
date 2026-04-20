<script setup lang="ts">
import { ref, watch, nextTick, onMounted } from 'vue'
import type { DiffInfo } from '../types'
import * as Diff2Html from 'diff2html'
import hljs from 'highlight.js'
import 'diff2html/bundles/css/diff2html.min.css'
import 'highlight.js/styles/github-dark.css'

const props = defineProps<{
  diffData: DiffInfo[]
  baseBranch: string
  compareBranch: string
}>()

const diffHtml = ref('')
const selectedFile = ref<string | null>(null)
const viewMode = ref<string>('side-by-side')

const toggleViewMode = () => {
  viewMode.value = viewMode.value === 'side-by-side' ? 'line-by-line' : 'side-by-side'
  renderDiff()
}

const renderDiff = () => {
  if (!props.diffData || props.diffData.length === 0) {
    diffHtml.value = ''
    return
  }

  const filesToRender = selectedFile.value
    ? props.diffData.filter(d => d.file === selectedFile.value)
    : props.diffData

  const diffString = filesToRender.map(d => d.patch).join('\n')

  if (!diffString.trim()) {
    diffHtml.value = ''
    return
  }

  diffHtml.value = Diff2Html.html(diffString, {
    drawFileList: false,
    matching: 'lines',
    outputFormat: viewMode.value as any,
    renderNothingWhenEmpty: true,
  })

  nextTick(() => {
    applySyntaxHighlighting()
  })
}

const applySyntaxHighlighting = () => {
  document.querySelectorAll('#diff-container pre code').forEach((block) => {
    hljs.highlightElement(block as HTMLElement)
  })
}

watch(() => [props.diffData, selectedFile.value, viewMode.value], renderDiff, { deep: true })

onMounted(() => {
  renderDiff()
})
</script>

<template>
  <div class="diff-viewer">
    <!-- 工具栏 -->
    <div class="diff-toolbar">
      <div class="diff-branches">
        <span class="badge badge-accent">{{ compareBranch }}</span>
        <svg viewBox="0 0 16 16" width="14" height="14" fill="currentColor">
          <path d="M3.72 3.72a.75.75 0 0 1 1.06 1.06L2.56 7h10.88l-2.22-2.22a.75.75 0 0 1 1.06-1.06l3.5 3.5a.75.75 0 0 1 0 1.06l-3.5 3.5a.75.75 0 1 1-1.06-1.06L13.44 8.5H2.56l2.22 2.22a.75.75 0 0 1-1.06 1.06l-3.5-3.5a.75.75 0 0 1 0-1.06Z"/>
        </svg>
        <span class="badge badge-info">{{ baseBranch }}</span>
      </div>

      <div class="diff-actions">
        <button class="btn btn-ghost btn-sm" @click="toggleViewMode" :title="viewMode === 'side-by-side' ? '切换到行内模式' : '切换到并排模式'">
          <svg v-if="viewMode === 'side-by-side'" viewBox="0 0 16 16" width="14" height="14" fill="currentColor">
            <path d="M2 2h12a1 1 0 0 1 1 1v10a1 1 0 0 1-1 1H2a1 1 0 0 1-1-1V3a1 1 0 0 1 1-1Zm0 1v10h5V3Zm6 10h6V3H8Z"/>
          </svg>
          <svg v-else viewBox="0 0 16 16" width="14" height="14" fill="currentColor">
            <path d="M8 2h6a1 1 0 0 1 1 1v10a1 1 0 0 1-1 1H8ZM2 2h5v12H2a1 1 0 0 1-1-1V3a1 1 0 0 1 1-1Zm1 1v10h3V3Zm6 0v10h4V3Z"/>
          </svg>
        </button>
      </div>
    </div>

    <!-- 文件过滤标签 -->
    <div v-if="diffData.length > 1" class="file-tabs">
      <button
        class="file-tab"
        :class="{ active: !selectedFile }"
        @click="selectedFile = null"
      >
        全部文件
      </button>
      <button
        v-for="diff in diffData"
        :key="diff.file"
        class="file-tab"
        :class="{ active: selectedFile === diff.file }"
        @click="selectedFile = diff.file"
      >
        <span class="file-name">{{ diff.file }}</span>
        <span class="file-stats">
          <span class="added">+{{ diff.added }}</span>
          <span class="deleted">-{{ diff.deleted }}</span>
        </span>
      </button>
    </div>

    <!-- Diff 内容 -->
    <div v-if="diffHtml" id="diff-container" class="diff-content" v-html="diffHtml"></div>

    <!-- 无差异 -->
    <div v-else class="no-diff">
      <p>两个分支之间没有差异 🎉</p>
    </div>
  </div>
</template>

<style scoped>
.diff-viewer {
  display: flex;
  flex-direction: column;
  gap: var(--space-sm);
  animation: fadeIn 0.2s ease;
}

.diff-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-sm) 0;
}

.diff-branches {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
}

.diff-actions {
  display: flex;
  gap: var(--space-xs);
}

.file-tabs {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-xs);
  padding: var(--space-sm) 0;
  border-bottom: 1px solid var(--border-muted);
}

.file-tab {
  display: inline-flex;
  align-items: center;
  gap: var(--space-sm);
  padding: var(--space-xs) var(--space-sm);
  font-size: var(--text-xs);
  font-family: var(--font-mono);
  color: var(--text-secondary);
  background: var(--bg-tertiary);
  border: 1px solid transparent;
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all var(--transition-fast);
  max-width: 250px;
}

.file-tab:hover {
  color: var(--text-primary);
  border-color: var(--border-default);
}

.file-tab.active {
  color: var(--accent-default);
  border-color: var(--accent-muted);
  background-color: var(--accent-subtle);
}

.file-tab.all {
  color: var(--color-info);
}

.file-name {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.file-stats {
  display: flex;
  gap: var(--space-xs);
  flex-shrink: 0;
}

.file-stats .added {
  color: var(--diff-added-text);
}

.file-stats .deleted {
  color: var(--diff-removed-text);
}

.diff-content {
  overflow-x: auto;
  border-radius: var(--radius-md);
}

.diff-content :deep(.d2h-wrapper) {
  margin-bottom: var(--space-md);
}

.no-diff {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 200px;
  color: var(--text-tertiary);
  font-size: var(--text-md);
}

@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}
</style>
