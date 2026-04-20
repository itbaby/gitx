// Git 相关类型定义

export interface BranchInfo {
  name: string
  isCurrent: boolean
}

export interface CommitInfo {
  hash: string
  shortHash: string
  message: string
  author: string
  email: string
  timestamp: string
}

export interface DiffInfo {
  file: string
  patch: string
  added: number
  deleted: number
}

export interface DiffStats {
  totalFiles: number
  totalAdded: number
  totalDeleted: number
}

// AI 相关类型定义

export interface ChatMessage {
  id: string
  role: 'user' | 'assistant' | 'system'
  content: string
  timestamp: Date
  isStreaming?: boolean
}

export interface InputMessage {
  role: 'user' | 'assistant'
  content: string
}

export interface ChatContext {
  base_branch: string
  compare_branch: string
  has_diff: boolean
}
