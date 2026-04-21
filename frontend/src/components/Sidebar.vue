<script setup lang="ts">
import { ref } from 'vue'

// Detect Tauri environment
const isTauri = () => typeof window !== 'undefined' && '__TAURI__' in window

const props = defineProps<{
  repoPath: string
  branches: string[]
  currentBranch: string
  baseBranch: string
  compareBranch: string
  loading: boolean
}>()

const emit = defineEmits<{
  'open-repo': [path: string]
  'refresh-branches': []
  'update:base-branch': [branch: string]
  'update:compare-branch': [branch: string]
  'compare': []
}>()

const inputPath = ref('')
const isExpanded = ref(true)

// 当仓库打开成功时，同步更新输入框
import { watch } from 'vue'
watch(() => props.repoPath, (val) => {
  if (val && !inputPath.value) {
    inputPath.value = val
  }
})

const handleOpen = () => {
  if (inputPath.value.trim()) {
    emit('open-repo', inputPath.value.trim())
  }
}

const handleKeyDown = (e: KeyboardEvent) => {
  if (e.key === 'Enter') handleOpen()
}

const handleRefresh = () => {
  emit('refresh-branches')
}

const handleBrowse = async () => {
  if (!isTauri()) return
  try {
    const { open } = await import('@tauri-apps/plugin-dialog')
    const selected = await open({
      directory: true,
      multiple: false,
      title: '选择 Git 仓库',
    })
    if (selected) {
      inputPath.value = selected as string
      handleOpen()
    }
  } catch {
    // Fallback: do nothing
  }
}

const getBranchIcon = (branch: string) => {
  if (branch === props.currentBranch) return '⎇'
  return '├'
}
</script>

<template>
  <aside class="sidebar">
    <!-- 仓库路径 -->
    <div class="sidebar-section">
      <div class="section-header" @click="isExpanded = !isExpanded">
        <svg viewBox="0 0 16 16" width="14" height="14" fill="currentColor">
          <path d="M2 2.5A2.5 2.5 0 0 1 4.5 0h8.75a.75.75 0 0 1 .75.75v12.5a.75.75 0 0 1-.75.75h-2.5a.75.75 0 0 1 0-1.5h1.75v-2h-8a1 1 0 0 0-.714 1.7.75.75 0 1 1-1.072 1.05A2.495 2.495 0 0 1 2 11.5Zm10.5-1h-8a1 1 0 0 0-1 1v6.708A2.486 2.486 0 0 1 4.5 9h8Z"/>
        </svg>
        <span>仓库</span>
        <svg class="chevron" :class="{ rotated: !isExpanded }" viewBox="0 0 16 16" width="12" height="12" fill="currentColor">
          <path d="m4.427 7.427 3.396 3.396a.25.25 0 0 0 .354 0l3.396-3.396A.25.25 0 0 0 11.396 7H4.604a.25.25 0 0 0-.177.427Z"/>
        </svg>
      </div>
      <Transition name="collapse">
        <div v-show="isExpanded" class="section-content">
          <div class="repo-input-group">
            <input
              v-model="inputPath"
              type="text"
              placeholder="输入仓库路径..."
              :disabled="loading"
              @keydown="handleKeyDown"
            />
            <button
              v-if="isTauri()"
              class="btn btn-secondary btn-sm browse-btn"
              :disabled="loading"
              @click="handleBrowse"
              title="浏览文件夹"
            >
              ...
            </button>
            <button class="btn btn-primary btn-sm" :disabled="loading || !inputPath.trim()" @click="handleOpen">
              打开
            </button>
          </div>
          <div v-if="repoPath" class="repo-info">
            <span class="repo-path" :title="repoPath">{{ repoPath }}</span>
          </div>
        </div>
      </Transition>
    </div>

    <!-- 分支列表 -->
    <div v-if="branches.length > 0" class="sidebar-section branches-section">
      <div class="section-header">
        <svg viewBox="0 0 16 16" width="14" height="14" fill="currentColor">
          <path d="M11.75 2.5a.75.75 0 1 0 0 1.5.75.75 0 0 0 0-1.5Zm-2.25.75a2.25 2.25 0 1 1 3 2.122V6A2.5 2.5 0 0 1 10 8.5H6a1 1 0 0 0-1 1v1.128a2.251 2.251 0 1 1-1.5 0V5.372a2.25 2.25 0 1 1 1.5 0v1.836A2.493 2.493 0 0 1 6 7h4a1 1 0 0 0 1-1v-.628A2.25 2.25 0 0 1 9.5 3.25ZM4.25 12a.75.75 0 1 0 0 1.5.75.75 0 0 0 0-1.5ZM3.5 3.25a.75.75 0 1 1 1.5 0 .75.75 0 0 1-1.5 0Z"/>
        </svg>
        <span>分支</span>
        <button class="btn btn-ghost btn-icon refresh-btn" @click="handleRefresh" title="刷新">
          <svg viewBox="0 0 16 16" width="12" height="12" fill="currentColor">
            <path d="M8 2.5a5.487 5.487 0 0 0-4.131 1.869l1.204 1.204A.25.25 0 0 1 4.896 6H1.25A.25.25 0 0 1 1 5.75V2.104a.25.25 0 0 1 .427-.177l1.38 1.38A7.002 7.002 0 0 1 14.95 7.16a.75.75 0 1 1-1.49.178A5.5 5.5 0 0 0 8 2.5ZM1.705 8.005a.75.75 0 0 1 .834.656 5.5 5.5 0 0 0 9.592 2.97l-1.204-1.204a.25.25 0 0 1 .177-.427h3.646a.25.25 0 0 1 .25.25v3.646a.25.25 0 0 1-.427.177l-1.38-1.38A7.002 7.002 0 0 1 1.05 8.84a.75.75 0 0 1 .656-.834Z"/>
          </svg>
        </button>
      </div>
      <div class="branch-list">
        <div
          v-for="branch in branches"
          :key="branch"
          class="branch-item"
          :class="{ active: branch === currentBranch }"
          @click="emit('update:compare-branch', branch)"
        >
          <span class="branch-icon">{{ getBranchIcon(branch) }}</span>
          <span class="branch-name">{{ branch }}</span>
        </div>
      </div>
    </div>

    <!-- 分支比较 -->
    <div v-if="branches.length > 0" class="sidebar-section compare-section">
      <div class="section-header">
        <svg viewBox="0 0 16 16" width="14" height="14" fill="currentColor">
          <path d="M1 7.775V2.75C1 1.784 1.784 1 2.75 1h5.025c.464 0 .91.184 1.238.513l6.25 6.25a1.75 1.75 0 0 1 0 2.474l-5.026 5.026a1.75 1.75 0 0 1-2.474 0l-6.25-6.25A1.752 1.752 0 0 1 1 7.775Zm1.5 0c0 .066.026.13.073.177l6.25 6.25a.25.25 0 0 0 .354 0l5.025-5.025a.25.25 0 0 0 0-.354l-6.25-6.25a.25.25 0 0 0-.177-.073H2.75a.25.25 0 0 0-.25.25ZM6 5a1 1 0 1 1 0 2 1 1 0 0 1 0-2Z"/>
        </svg>
        <span>比较</span>
      </div>
      <div class="compare-content">
        <label class="compare-label">基准分支</label>
        <select :value="baseBranch" @change="emit('update:base-branch', ($event.target as HTMLSelectElement).value)">
          <option v-for="b in branches" :key="b" :value="b">{{ b }}</option>
        </select>
        <div class="compare-arrow">
          <svg viewBox="0 0 16 16" width="16" height="16" fill="currentColor">
            <path d="M8 1a.75.75 0 0 1 .75.75v10.69l2.72-2.72a.749.749 0 0 1 1.275.326.749.749 0 0 1-.215.734l-4 4a.75.75 0 0 1-1.06 0l-4-4a.749.749 0 0 1 .326-1.275.749.749 0 0 1 .734.215l2.72 2.72V1.75A.75.75 0 0 1 8 1Z"/>
          </svg>
        </div>
        <label class="compare-label">对比分支</label>
        <select :value="compareBranch" @change="emit('update:compare-branch', ($event.target as HTMLSelectElement).value)">
          <option v-for="b in branches" :key="b" :value="b">{{ b }}</option>
        </select>
        <button class="btn btn-primary compare-btn" :disabled="loading || !baseBranch || !compareBranch" @click="emit('compare')">
          比较分支
        </button>
      </div>
    </div>
  </aside>
</template>

<style scoped>
.sidebar {
  width: var(--sidebar-width);
  min-width: var(--sidebar-width);
  background-color: var(--bg-secondary);
  border-right: 1px solid var(--border-default);
  display: flex;
  flex-direction: column;
  overflow-y: auto;
  flex-shrink: 0;
}

.sidebar-section {
  border-bottom: 1px solid var(--border-muted);
}

.section-header {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  padding: var(--space-sm) var(--space-md);
  font-size: var(--text-xs);
  font-weight: 600;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  cursor: pointer;
  user-select: none;
}

.section-header:hover {
  color: var(--text-primary);
}

.chevron {
  margin-left: auto;
  transition: transform var(--transition-fast);
}

.chevron.rotated {
  transform: rotate(-90deg);
}

.section-content {
  padding: 0 var(--space-md) var(--space-md);
}

/* 折叠动画 */
.collapse-enter-active,
.collapse-leave-active {
  transition: all var(--transition-normal);
  overflow: hidden;
}

.collapse-enter-from,
.collapse-leave-to {
  opacity: 0;
  max-height: 0;
  padding-top: 0;
  padding-bottom: 0;
}

/* 仓库输入 */
.repo-input-group {
  display: flex;
  gap: var(--space-sm);
}

.repo-input-group input {
  flex: 1;
  font-size: var(--text-sm);
  padding: var(--space-xs) var(--space-sm);
}

.browse-btn {
  padding: 2px var(--space-sm);
  font-family: var(--font-mono);
  min-width: 32px;
  justify-content: center;
}

.repo-info {
  margin-top: var(--space-sm);
}

.repo-path {
  display: block;
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  font-family: var(--font-mono);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* 分支列表 */
.branches-section {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.branches-section .section-header {
  cursor: default;
  flex-shrink: 0;
}

.refresh-btn {
  margin-left: auto;
  padding: 2px;
}

.branch-list {
  flex: 1;
  overflow-y: auto;
  padding: 0 var(--space-xs);
}

.branch-item {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  padding: var(--space-xs) var(--space-sm);
  border-radius: var(--radius-sm);
  cursor: pointer;
  font-size: var(--text-sm);
  color: var(--text-secondary);
  transition: all var(--transition-fast);
}

.branch-item:hover {
  background-color: var(--bg-tertiary);
  color: var(--text-primary);
}

.branch-item.active {
  background-color: var(--accent-subtle);
  color: var(--accent-default);
}

.branch-icon {
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  width: 16px;
  text-align: center;
  flex-shrink: 0;
}

.branch-name {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* 分支比较 */
.compare-section {
  flex-shrink: 0;
}

.compare-section .section-header {
  cursor: default;
}

.compare-content {
  padding: 0 var(--space-md) var(--space-md);
  display: flex;
  flex-direction: column;
  gap: var(--space-xs);
}

.compare-label {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  margin-top: var(--space-xs);
}

.compare-content select {
  width: 100%;
  font-size: var(--text-sm);
  padding: var(--space-xs) var(--space-sm);
}

.compare-arrow {
  display: flex;
  justify-content: center;
  color: var(--text-tertiary);
  padding: var(--space-xs) 0;
}

.compare-btn {
  margin-top: var(--space-xs);
  width: 100%;
}
</style>
