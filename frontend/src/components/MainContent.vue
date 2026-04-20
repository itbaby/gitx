<script setup lang="ts">
import { ref, computed } from 'vue'
import type { DiffInfo, CommitInfo, DiffStats } from '../types'
import DiffViewer from './DiffViewer.vue'
import CommitList from './CommitList.vue'
import FileHistoryList from './FileHistoryList.vue'

const props = defineProps<{
  activeTab: 'diff' | 'history' | 'commits'
  diffData: DiffInfo[]
  diffStats: DiffStats
  commits: CommitInfo[]
  fileHistory: CommitInfo[]
  baseBranch: string
  compareBranch: string
  loading: boolean
  hasRepo: boolean
}>()

const emit = defineEmits<{
  'tab-change': [tab: 'diff' | 'history' | 'commits']
  'file-select': [file: string]
  'fetch-commits': [branch?: string]
  'fetch-file-history': [file: string, timeRange: string]
}>()

// 文件历史查询
const historyFile = ref('')
const historyTimeRange = ref('3d')

const timeRangeOptions = [
  { label: '3 天', value: '3d' },
  { label: '7 天', value: '7d' },
  { label: '14 天', value: '14d' },
  { label: '30 天', value: '30d' },
]

const searchFileHistory = () => {
  if (historyFile.value.trim()) {
    emit('fetch-file-history', historyFile.value.trim(), historyTimeRange.value)
  }
}

// 空状态
const showEmptyState = computed(() => {
  return !props.hasRepo || (props.activeTab === 'diff' && props.diffData.length === 0)
})

const emptyTitle = computed(() => {
  if (!props.hasRepo) return '打开一个 Git 仓库'
  return '选择分支开始比较'
})

const emptyDescription = computed(() => {
  if (!props.hasRepo) return '在左侧输入仓库路径，打开一个本地 Git 仓库'
  return '在左侧选择基准分支和对比分支，点击"比较分支"按钮'
})
</script>

<template>
  <main class="main-content">
    <!-- 标签栏 -->
    <div class="tab-bar">
      <button
        class="tab-item"
        :class="{ active: activeTab === 'diff' }"
        @click="emit('tab-change', 'diff')"
      >
        <svg viewBox="0 0 16 16" width="14" height="14" fill="currentColor">
          <path d="M8 0c-.366 0-.72.106-1.021.304L2.5 3.248a1.75 1.75 0 0 0-.73 1.063v6.932a1.75 1.75 0 0 0 .912 1.536l4.479 2.586a1.748 1.748 0 0 0 1.756 0l4.479-2.586a1.75 1.75 0 0 0 .912-1.536V4.311a1.75 1.75 0 0 0-.73-1.063L9.021.304A1.75 1.75 0 0 0 8 0ZM6.5 2.064 8 1.115l1.5.949V4.5h-3ZM4.5 4.5V3.418l2 1.25v2.5l-2 1.25ZM4 9.322l2.5-1.562 2.5 1.562V12l-2.5 1.562L4 12ZM8.5 8.256l2.5-1.562V11l-2.5 1.562ZM11 7.068l2-1.25V4.5h-2ZM13 5.5V7l-2 1.25V6.25Z"/>
        </svg>
        差异视图
        <span v-if="diffData.length > 0" class="tab-count badge badge-info">{{ diffStats.totalFiles }}</span>
      </button>
      <button
        class="tab-item"
        :class="{ active: activeTab === 'commits' }"
        @click="emit('tab-change', 'commits')"
      >
        <svg viewBox="0 0 16 16" width="14" height="14" fill="currentColor">
          <path d="M7.177 3.073L9.573.677A.25.25 0 0 1 10 .854V2.5h1A2.5 2.5 0 0 1 13.5 5v5.628a2.251 2.251 0 1 1-1.5 0V5a1 1 0 0 0-1-1h-1v1.646a.25.25 0 0 1-.427.177L7.177 3.427a.25.25 0 0 1 0-.354ZM3.75 2.5a.75.75 0 1 0 0 1.5.75.75 0 0 0 0-1.5Zm0 9.5a.75.75 0 1 0 0 1.5.75.75 0 0 0 0-1.5Zm8.25.75a.75.75 0 1 0 1.5 0 .75.75 0 0 0-1.5 0Z"/>
        </svg>
        提交历史
      </button>
      <button
        class="tab-item"
        :class="{ active: activeTab === 'history' }"
        @click="emit('tab-change', 'history')"
      >
        <svg viewBox="0 0 16 16" width="14" height="14" fill="currentColor">
          <path d="M.5 3a.5.5 0 0 0-1 0v2.762l.008.01a2.002 2.002 0 0 0 .727 1.238 10 10 0 0 0 3.467 2.218c.198.047.41.07.629.07.496 0 .93-.13 1.248-.36.298-.215.558-.548.67-.954.112-.406.084-.84-.065-1.2-.148-.361-.407-.67-.748-.888a4.04 4.04 0 0 0-1.434-.505c-.168-.032-.335-.05-.5-.058a10.5 10.5 0 0 0-1-.028V3Zm1 0v2.253c.355.003.71.014 1.062.043.147.01.29.024.43.043.338.057.625.15.847.283a.93.93 0 0 1 .346.397c.055.135.057.285-.01.434-.066.149-.205.28-.39.385a1.5 1.5 0 0 1-.76.188 2.4 2.4 0 0 1-.474-.053 9 9 0 0 1-3.112-1.99V5.5A2 2 0 0 1 1.5 3ZM8 3v2.282c.15.003.3.01.452.021.2.015.393.04.58.076.37.07.71.183 1 .344.298.165.55.403.72.713.168.31.208.662.098 1.013-.11.35-.34.667-.66.904a2.42 2.42 0 0 1-1.166.526c-.395.057-.797.057-1.186.005A10.05 10.05 0 0 1 5 8.267V7a9 9 0 0 0 2.69 1.13c.33.057.657.064.982.02a1.42 1.42 0 0 0 .684-.308.9.9 0 0 0 .288-.543c.05-.252.017-.485-.077-.674a1.46 1.46 0 0 0-.488-.497 3.87 3.87 0 0 0-.796-.335A6.3 6.3 0 0 0 8 5.648V3Z"/>
        </svg>
        文件历史
      </button>

      <!-- Diff 统计信息 -->
      <div v-if="activeTab === 'diff' && diffData.length > 0" class="diff-stats">
        <span class="stat-item added">+{{ diffStats.totalAdded }}</span>
        <span class="stat-item deleted">-{{ diffStats.totalDeleted }}</span>
        <span class="stat-item files">{{ diffStats.totalFiles }} 个文件</span>
      </div>
    </div>

    <!-- 工具栏（文件历史模式） -->
    <div v-if="activeTab === 'history'" class="toolbar">
      <input
        v-model="historyFile"
        type="text"
        placeholder="输入文件路径，例如: src/main.go"
        class="toolbar-input"
        @keydown.enter="searchFileHistory"
      />
      <select v-model="historyTimeRange" class="toolbar-select">
        <option v-for="opt in timeRangeOptions" :key="opt.value" :value="opt.value">{{ opt.label }}</option>
      </select>
      <button class="btn btn-primary btn-sm" :disabled="loading || !historyFile.trim()" @click="searchFileHistory">
        查询
      </button>
    </div>

    <!-- 内容区域 -->
    <div class="content-area">
      <!-- 空状态 -->
      <div v-if="showEmptyState" class="empty-state">
        <div class="empty-icon">
          <svg viewBox="0 0 24 24" width="48" height="48" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M9 5H7a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V7a2 2 0 0 0-2-2h-2M9 5a2 2 0 0 0 2 2h2a2 2 0 0 0 2-2M9 5a2 2 0 0 1 2-2h2a2 2 0 0 1 2 2"/>
            <line x1="9" y1="12" x2="15" y2="12"/>
            <line x1="9" y1="16" x2="13" y2="16"/>
          </svg>
        </div>
        <h3>{{ emptyTitle }}</h3>
        <p>{{ emptyDescription }}</p>
      </div>

      <!-- Loading 状态 -->
      <div v-else-if="loading" class="loading-state">
        <div class="spinner"></div>
        <span>加载中...</span>
      </div>

      <!-- Diff 视图 -->
      <DiffViewer
        v-else-if="activeTab === 'diff' && diffData.length > 0"
        :diff-data="diffData"
        :base-branch="baseBranch"
        :compare-branch="compareBranch"
      />

      <!-- 提交历史 -->
      <CommitList
        v-else-if="activeTab === 'commits'"
        :commits="commits"
        @refresh="emit('fetch-commits')"
      />

      <!-- 文件历史 -->
      <FileHistoryList
        v-else-if="activeTab === 'history'"
        :commits="fileHistory"
        :file="historyFile"
      />
    </div>
  </main>
</template>

<style scoped>
.main-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  min-width: 0;
}

/* 标签栏 */
.tab-bar {
  display: flex;
  align-items: center;
  gap: var(--space-xs);
  padding: var(--space-sm) var(--space-lg);
  border-bottom: 1px solid var(--border-default);
  background-color: var(--bg-secondary);
  flex-shrink: 0;
  overflow-x: auto;
}

.tab-item {
  display: inline-flex;
  align-items: center;
  gap: var(--space-xs);
  padding: var(--space-xs) var(--space-md);
  font-size: var(--text-sm);
  color: var(--text-secondary);
  background: transparent;
  border: 1px solid transparent;
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all var(--transition-fast);
  white-space: nowrap;
}

.tab-item:hover {
  color: var(--text-primary);
  background-color: var(--bg-tertiary);
}

.tab-item.active {
  color: var(--text-primary);
  background-color: var(--bg-tertiary);
  border-color: var(--border-default);
}

.tab-count {
  font-size: 10px;
  min-width: 16px;
  height: 16px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 0 4px;
}

.diff-stats {
  display: flex;
  align-items: center;
  gap: var(--space-md);
  margin-left: auto;
  padding-left: var(--space-lg);
  border-left: 1px solid var(--border-muted);
}

.stat-item {
  font-size: var(--text-sm);
  font-weight: 600;
  font-family: var(--font-mono);
}

.stat-item.added {
  color: var(--diff-added-text);
}

.stat-item.deleted {
  color: var(--diff-removed-text);
}

.stat-item.files {
  color: var(--text-secondary);
  font-weight: 400;
}

/* 工具栏 */
.toolbar {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  padding: var(--space-sm) var(--space-lg);
  border-bottom: 1px solid var(--border-default);
  background-color: var(--bg-secondary);
  flex-shrink: 0;
}

.toolbar-input {
  flex: 1;
  font-size: var(--text-sm);
  font-family: var(--font-mono);
}

.toolbar-select {
  width: 80px;
  font-size: var(--text-sm);
}

/* 内容区域 */
.content-area {
  flex: 1;
  overflow: auto;
  padding: var(--space-lg);
}

/* 空状态 */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--text-tertiary);
  text-align: center;
  animation: fadeIn 0.3s ease;
}

.empty-icon {
  margin-bottom: var(--space-xl);
  opacity: 0.4;
}

.empty-state h3 {
  font-size: var(--text-md);
  color: var(--text-secondary);
  margin-bottom: var(--space-sm);
}

.empty-state p {
  font-size: var(--text-sm);
  max-width: 300px;
}

/* Loading */
.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  gap: var(--space-md);
  color: var(--text-tertiary);
}

@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}
</style>
