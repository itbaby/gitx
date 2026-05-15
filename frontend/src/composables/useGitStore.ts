import { ref, computed } from 'vue'
import { gitApi } from '../api/index'
import type { DiffInfo, CommitInfo } from '../types/index'
import { friendlyError } from '../utils/error'

// ---- Shared reactive state (singleton) ----

const repoPath = ref('')
const branches = ref<string[]>([])
const currentBranch = ref('')
const baseBranch = ref('')
const compareBranch = ref('')
const diffData = ref<DiffInfo[]>([])
const commits = ref<CommitInfo[]>([])
const fileHistory = ref<CommitInfo[]>([])
const loading = ref(false)
const activeTab = ref<'diff' | 'history' | 'commits'>('diff')
const error = ref('')

// ---- Computed ----

const diffStats = computed(() => {
  const stats = { totalFiles: 0, totalAdded: 0, totalDeleted: 0 }
  for (const d of diffData.value) {
    stats.totalFiles++
    stats.totalAdded += d.added
    stats.totalDeleted += d.deleted
  }
  return stats
})

const hasRepo = computed(() => !!repoPath.value)

// ---- Git operations ----

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
  } catch (e: unknown) {
    error.value = friendlyError(e)
  } finally {
    loading.value = false
  }
}

const fetchBranches = async () => {
  try {
    const res = await gitApi.getBranches()
    branches.value = res.branches
  } catch (e: unknown) {
    error.value = friendlyError(e)
  }
}

const compareBranches = async (b1: string, b2: string) => {
  loading.value = true
  error.value = ''
  activeTab.value = 'diff'
  try {
    // Fetch diff only; commits are loaded lazily via fetchCommits
    const res = await gitApi.getBranchDiff(b1, b2)
    diffData.value = res.diff
  } catch (e: unknown) {
    error.value = friendlyError(e)
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
  } catch (e: unknown) {
    error.value = friendlyError(e)
    commits.value = []
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
  } catch (e: unknown) {
    error.value = friendlyError(e)
    fileHistory.value = []
  } finally {
    loading.value = false
  }
}

const fetchCommitDiff = async (hash: string) => {
  loading.value = true
  error.value = ''
  activeTab.value = 'diff'
  try {
    const res = await gitApi.getCommitDiff(hash)
    diffData.value = res.diff
  } catch (e: unknown) {
    error.value = friendlyError(e)
    diffData.value = []
  } finally {
    loading.value = false
  }
}

// ---- Handlers ----

const onBaseBranchChange = (branch: string) => { baseBranch.value = branch }
const onCompareBranchChange = (branch: string) => { compareBranch.value = branch }
const onCompare = () => { compareBranches(baseBranch.value, compareBranch.value) }

export function useGitStore() {
  return {
    // State
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
    // Computed
    diffStats,
    hasRepo,
    // Functions
    openRepo,
    fetchBranches,
    compareBranches,
    fetchCommits,
    fetchFileHistory,
    fetchCommitDiff,
    // Handlers
    onBaseBranchChange,
    onCompareBranchChange,
    onCompare,
    // Utility
    friendlyError,
  }
}
