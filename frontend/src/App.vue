<script setup lang="ts">
import { ref, onBeforeUnmount } from 'vue'
import { useGitStore } from './composables/useGitStore'
import { useAiChat } from './composables/useAiChat'
import Sidebar from './components/Sidebar.vue'
import MainContent from './components/MainContent.vue'
import AIPanel from './components/AIPanel.vue'

// ---- Panel resize ----

const sidebarWidth = ref(260)
const aiPanelWidth = ref(340)
const MIN_SIDEBAR = 180
const MAX_SIDEBAR = 500
const MIN_AI_PANEL = 260
const MAX_AI_PANEL = 600

interface DragState {
  handle: 'sidebar' | 'ai-panel'
  startX: number
  startWidth: number
}

let dragState: DragState | null = null

function onHandleMouseDown(handle: 'sidebar' | 'ai-panel', e: MouseEvent) {
  dragState = {
    handle,
    startX: e.clientX,
    startWidth: handle === 'sidebar' ? sidebarWidth.value : aiPanelWidth.value,
  }
  document.addEventListener('mousemove', onMouseMove)
  document.addEventListener('mouseup', onMouseUp)
  document.body.style.cursor = 'col-resize'
  document.body.style.userSelect = 'none'
  e.preventDefault()
}

function onMouseMove(e: MouseEvent) {
  if (!dragState) return
  const delta = e.clientX - dragState.startX
  if (dragState.handle === 'sidebar') {
    sidebarWidth.value = Math.min(MAX_SIDEBAR, Math.max(MIN_SIDEBAR, dragState.startWidth + delta))
  } else {
    aiPanelWidth.value = Math.min(MAX_AI_PANEL, Math.max(MIN_AI_PANEL, dragState.startWidth - delta))
  }
}

function onMouseUp() {
  dragState = null
  document.removeEventListener('mousemove', onMouseMove)
  document.removeEventListener('mouseup', onMouseUp)
  document.body.style.cursor = ''
  document.body.style.userSelect = ''
}

onBeforeUnmount(() => {
  document.removeEventListener('mousemove', onMouseMove)
  document.removeEventListener('mouseup', onMouseUp)
})

// ---- Keyboard shortcuts ----

function onKeyDown(e: KeyboardEvent) {
  if (e.key === 'Escape' && error.value) {
    error.value = ''
  }
}

// Git store
const {
  repoPath,
  branches,
  currentBranch,
  baseBranch,
  compareBranch,
  diffData,
  commits,
  fileHistory,
  loading,
  activeTab,
  error,
  diffStats,
  hasRepo,
  openRepo,
  fetchBranches,
  fetchCommits,
  fetchFileHistory,
  fetchCommitDiff,
  onBaseBranchChange,
  onCompareBranchChange,
  onCompare,
} = useGitStore()

// AI chat
const {
  aiMessages,
  toolStatus,
  isStreaming,
  handleChat,
  handleAnalyze,
  clearChat,
} = useAiChat(
  () => baseBranch.value,
  () => compareBranch.value,
  () => diffData.value,
)
</script>

<template>
  <div class="app-layout" @keydown="onKeyDown">
    <!-- 顶部栏 -->
    <header class="app-header">
      <div class="header-left">
        <svg class="logo-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20"/>
          <path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z"/>
          <line x1="8" y1="7" x2="16" y2="7"/>
          <line x1="8" y1="11" x2="14" y2="11"/>
        </svg>
        <span class="app-title">GitX</span>
        <span class="app-subtitle">AI Git Diff Analyzer</span>
      </div>
      <div class="header-right">
        <span v-if="repoPath" class="repo-indicator">
          <svg viewBox="0 0 16 16" width="14" height="14" fill="currentColor">
            <path d="M2 2.5A2.5 2.5 0 0 1 4.5 0h8.75a.75.75 0 0 1 .75.75v12.5a.75.75 0 0 1-.75.75h-2.5a.75.75 0 0 1 0-1.5h1.75v-2h-8a1 1 0 0 0-.714 1.7.75.75 0 1 1-1.072 1.05A2.495 2.495 0 0 1 2 11.5Zm10.5-1h-8a1 1 0 0 0-1 1v6.708A2.486 2.486 0 0 1 4.5 9h8Z"/>
          </svg>
          {{ repoPath.split('/').pop() || repoPath }}
        </span>
        <span v-if="currentBranch" class="branch-badge badge badge-accent">
          {{ currentBranch }}
        </span>
      </div>
    </header>

    <!-- 错误提示 -->
    <Transition name="error">
      <div v-if="error" class="error-bar" role="alert">
        <span>{{ error }}</span>
        <button class="btn btn-ghost btn-sm" @click="error = ''">&#10005;</button>
      </div>
    </Transition>

    <!-- 主内容区 -->
    <div class="app-body">
      <Sidebar
        :style="{ width: sidebarWidth + 'px', minWidth: sidebarWidth + 'px' }"
        :repo-path="repoPath"
        :branches="branches"
        :current-branch="currentBranch"
        :base-branch="baseBranch"
        :compare-branch="compareBranch"
        :loading="loading"
        @open-repo="openRepo"
        @refresh-branches="fetchBranches"
        @update:base-branch="onBaseBranchChange"
        @update:compare-branch="onCompareBranchChange"
        @compare="onCompare"
      />

      <div
        class="resize-handle"
        @mousedown="onHandleMouseDown('sidebar', $event)"
      ></div>

      <MainContent
        :active-tab="activeTab"
        :diff-data="diffData"
        :diff-stats="diffStats"
        :commits="commits"
        :file-history="fileHistory"
        :base-branch="baseBranch"
        :compare-branch="compareBranch"
        :loading="loading"
        :has-repo="hasRepo"
        @tab-change="activeTab = $event"
        @commit-select="fetchCommitDiff"
        @fetch-commits="fetchCommits"
        @fetch-file-history="fetchFileHistory"
      />

      <div
        class="resize-handle"
        @mousedown="onHandleMouseDown('ai-panel', $event)"
      ></div>

      <AIPanel
        :style="{ width: aiPanelWidth + 'px', minWidth: aiPanelWidth + 'px' }"
        :messages="aiMessages"
        :has-diff="diffData.length > 0"
        :loading="loading"
        :is-streaming="isStreaming"
        :tool-status="toolStatus"
        @send="handleChat"
        @analyze="handleAnalyze"
        @clear="clearChat"
      />
    </div>
  </div>
</template>

<style scoped>
.app-layout {
  display: flex;
  flex-direction: column;
  height: 100vh;
  overflow: hidden;
}

.app-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: var(--header-height);
  padding: 0 var(--space-lg);
  background-color: var(--bg-secondary);
  border-bottom: 1px solid var(--border-default);
  flex-shrink: 0;
}

.header-left {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
}

.logo-icon {
  width: 22px;
  height: 22px;
  color: var(--accent-default);
}

.app-title {
  font-size: var(--text-md);
  font-weight: 700;
  color: var(--text-primary);
  letter-spacing: -0.02em;
}

.app-subtitle {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  margin-left: var(--space-sm);
  padding-left: var(--space-sm);
  border-left: 1px solid var(--border-default);
}

.header-right {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
}

.repo-indicator {
  display: flex;
  align-items: center;
  gap: var(--space-xs);
  font-size: var(--text-sm);
  color: var(--text-secondary);
  padding: 2px var(--space-sm);
  background-color: var(--bg-tertiary);
  border-radius: var(--radius-sm);
  max-width: 200px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.branch-badge {
  font-family: var(--font-mono);
}

.error-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-sm) var(--space-lg);
  background-color: var(--color-danger-subtle);
  border-bottom: 1px solid var(--color-danger);
  color: var(--color-danger);
  font-size: var(--text-sm);
  flex-shrink: 0;
}

.error-enter-active,
.error-leave-active {
  transition: all var(--transition-normal);
}

.error-enter-from,
.error-leave-to {
  opacity: 0;
  transform: translateY(-100%);
}

.app-body {
  display: flex;
  flex: 1;
  overflow: hidden;
}

.resize-handle {
  width: 4px;
  cursor: col-resize;
  background-color: transparent;
  transition: background-color var(--transition-fast);
  flex-shrink: 0;
  z-index: 10;
}

.resize-handle:hover {
  background-color: var(--accent-muted);
}
</style>
