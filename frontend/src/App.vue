<script setup lang="ts">
import { ref, computed } from 'vue'
import type { DiffInfo, CommitInfo, ChatMessage, InputMessage, ChatContext } from './types'
import { gitApi, aiApi } from './api'
import Sidebar from './components/Sidebar.vue'
import MainContent from './components/MainContent.vue'
import AIPanel from './components/AIPanel.vue'

// 应用状态
const repoPath = ref('')
const branches = ref<string[]>([])
const currentBranch = ref('')
const baseBranch = ref('')
const compareBranch = ref('')
const diffData = ref<DiffInfo[]>([])
const commits = ref<CommitInfo[]>([])
const fileHistory = ref<CommitInfo[]>([])
const aiMessages = ref<ChatMessage[]>([])
const toolStatus = ref('')
const loading = ref(false)
const activeTab = ref<'diff' | 'history' | 'commits'>('diff')
const error = ref('')

// Diff 统计
const diffStats = computed(() => {
  const stats = { totalFiles: 0, totalAdded: 0, totalDeleted: 0 }
  for (const d of diffData.value) {
    stats.totalFiles++
    stats.totalAdded += d.added
    stats.totalDeleted += d.deleted
  }
  return stats
})

// 是否已打开仓库
const hasRepo = computed(() => !!repoPath.value)

// ---- Git 操作 ----

const openRepo = async (path: string) => {
  loading.value = true
  error.value = ''
  try {
    await gitApi.openRepo(path)
    repoPath.value = path
    const [branchesRes, currentRes] = await Promise.all([
      gitApi.getBranches(),
      gitApi.getCurrentBranch(),
    ])
    branches.value = branchesRes.branches
    currentBranch.value = currentRes.current_branch
    compareBranch.value = currentBranch.value
    baseBranch.value = branches.value.includes('main') ? 'main' : branches.value.includes('master') ? 'master' : branches.value[0]
  } catch (e: any) {
    error.value = e.message
  } finally {
    loading.value = false
  }
}

const fetchBranches = async () => {
  try {
    const res = await gitApi.getBranches()
    branches.value = res.branches
  } catch (e: any) {
    error.value = e.message
  }
}

const compareBranches = async (b1: string, b2: string) => {
  loading.value = true
  error.value = ''
  activeTab.value = 'diff'
  try {
    const res = await gitApi.getBranchDiff(b1, b2)
    diffData.value = res.diff
    const commitRes = await gitApi.getCommits(b2, 30)
    commits.value = commitRes.commits
  } catch (e: any) {
    error.value = e.message
    diffData.value = []
  } finally {
    loading.value = false
  }
}

const fetchCommits = async (branch?: string) => {
  loading.value = true
  error.value = ''
  activeTab.value = 'commits'
  try {
    const res = await gitApi.getCommits(branch || currentBranch.value, 50)
    commits.value = res.commits
  } catch (e: any) {
    error.value = e.message
  } finally {
    loading.value = false
  }
}

const fetchFileHistory = async (file: string, timeRange = '3d') => {
  loading.value = true
  error.value = ''
  activeTab.value = 'history'
  try {
    const res = await gitApi.getFileHistory(file, timeRange)
    fileHistory.value = res.commits
  } catch (e: any) {
    error.value = e.message
  } finally {
    loading.value = false
  }
}

// ---- AI 操作 ----

const handleChat = (text: string) => {
  const userMsg: ChatMessage = {
    id: crypto.randomUUID(),
    role: 'user',
    content: text,
    timestamp: new Date(),
  }
  aiMessages.value.push(userMsg)

  const aiMsg: ChatMessage = {
    id: crypto.randomUUID(),
    role: 'assistant',
    content: '',
    timestamp: new Date(),
    isStreaming: true,
  }
  aiMessages.value.push(aiMsg)

  const chatHistory: InputMessage[] = aiMessages.value
    .filter((m) => m.role === 'user' || (m.role === 'assistant' && !m.isStreaming))
    .map((m) => ({ role: m.role as 'user' | 'assistant', content: m.content }))

  const chatCtx: ChatContext = {
    base_branch: baseBranch.value,
    compare_branch: compareBranch.value,
    has_diff: diffData.value.length > 0,
  }

  aiApi.chat(
    chatHistory,
    chatCtx,
    (_name, display) => { toolStatus.value = display },
    (chunk) => {
      toolStatus.value = ''
      aiMsg.content += chunk
    },
    () => { aiMsg.isStreaming = false },
    (err) => {
      aiMsg.content = `请求失败: ${err.message}`
      aiMsg.isStreaming = false
      toolStatus.value = ''
    },
  )
}

const onBaseBranchChange = (branch: string) => { baseBranch.value = branch }
const onCompareBranchChange = (branch: string) => { compareBranch.value = branch }
const onCompare = () => { compareBranches(baseBranch.value, compareBranch.value) }
</script>

<template>
  <div class="app-layout">
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
      <div v-if="error" class="error-bar">
        <span>{{ error }}</span>
        <button class="btn btn-ghost btn-sm" @click="error = ''">✕</button>
      </div>
    </Transition>

    <!-- 主内容区 -->
    <div class="app-body">
      <Sidebar
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
        @file-select="() => {}"
        @fetch-commits="fetchCommits"
        @fetch-file-history="fetchFileHistory"
      />

      <AIPanel
        :messages="aiMessages"
        :has-diff="diffData.length > 0"
        :loading="loading"
        :tool-status="toolStatus"
        @send="handleChat"
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
</style>
