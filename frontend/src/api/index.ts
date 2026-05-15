import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { DiffInfo, CommitInfo, InputMessage, ChatContext } from '../types'

// ============================================================
// Git API (Tauri IPC only)
// ============================================================

export const gitApi = {
  async openRepo(path: string): Promise<{ message: string; path: string }> {
    const result = await invoke<string>('open_repo', { path })
    return { message: result, path }
  },

  async getBranches(): Promise<{ branches: string[] }> {
    const branches = await invoke<string[]>('get_branches')
    return { branches }
  },

  async getCurrentBranch(): Promise<{ current_branch: string }> {
    const branch = await invoke<string>('get_current_branch')
    return { current_branch: branch }
  },

  async getCommits(branch?: string, limit = 20): Promise<{ commits: CommitInfo[] }> {
    const commits = await invoke<CommitInfo[]>('get_commits', {
      branch: branch || null,
      limit,
    })
    return { commits }
  },

  async getDiff(from: string, to: string): Promise<{ diff: DiffInfo[] }> {
    const diff = await invoke<DiffInfo[]>('get_diff', { from, to })
    return { diff }
  },

  async getBranchDiff(branch1: string, branch2: string): Promise<{ diff: DiffInfo[] }> {
    const diff = await invoke<DiffInfo[]>('get_branch_diff', { branch1, branch2 })
    return { diff }
  },

  async getFileHistory(file: string, timeRange = '3d'): Promise<{ commits: CommitInfo[] }> {
    const commits = await invoke<CommitInfo[]>('get_file_history', { file, timeRange })
    return { commits }
  },

  async getCommitDiff(hash: string): Promise<{ diff: DiffInfo[] }> {
    const diff = await invoke<DiffInfo[]>('get_commit_diff', { hash })
    return { diff }
  },
}

// ============================================================
// AI API (Tauri IPC + event listeners)
// ============================================================

interface ToolEvent {
  name: string
  display: string
}

// Monotonic request counters to disambiguate concurrent streaming sessions.
let chatRequestId = 0
let analyzeRequestId = 0

export const aiApi = {
  /**
   * Stream a diff analysis directly (no agent loop, no tool calling).
   * Faster and cheaper for "analyze these changes" type queries.
   */
  async analyzeStream(
    diff: DiffInfo[],
    prompt: string,
    onChunk: (text: string) => void,
    onDone: () => void,
    onError: (err: Error) => void,
  ): Promise<void> {
    const requestId = ++analyzeRequestId
    const unlisteners: UnlistenFn[] = []
    const cleanup = () => unlisteners.forEach((u) => u())
    const isCurrentRequest = () => requestId === analyzeRequestId

    try {
      unlisteners.push(
        await listen<string>('ai-analyze-chunk', (event) => {
          if (!isCurrentRequest()) return
          onChunk(event.payload)
        }),
      )
      unlisteners.push(
        await listen<void>('ai-analyze-done', () => {
          if (!isCurrentRequest()) return
          cleanup()
          onDone()
        }),
      )
      unlisteners.push(
        await listen<string>('ai-error', (event) => {
          if (!isCurrentRequest()) return
          cleanup()
          onError(new Error(event.payload))
        }),
      )

      await invoke('ai_analyze_stream', { request: { diff, prompt } })
    } catch (err) {
      cleanup()
      onError(err instanceof Error ? err : new Error(String(err)))
    }
  },

  async chat(
    messages: InputMessage[],
    context: ChatContext,
    onTool: (name: string, display: string) => void,
    onToolResult: (name: string, result: string) => void,
    onChunk: (text: string) => void,
    onDone: () => void,
    onError: (err: Error) => void,
  ): Promise<void> {
    // Increment request ID so stale events from previous requests are ignored.
    const requestId = ++chatRequestId
    const unlisteners: UnlistenFn[] = []

    const cleanup = () => {
      unlisteners.forEach((u) => u())
    }

    const isCurrentRequest = () => requestId === chatRequestId

    try {
      unlisteners.push(
        await listen<ToolEvent>('ai-tool', (event) => {
          if (!isCurrentRequest()) return
          onTool(event.payload.name, event.payload.display)
        }),
      )

      unlisteners.push(
        await listen<{ name: string; result: string }>('ai-tool-result', (event) => {
          if (!isCurrentRequest()) return
          onToolResult(event.payload.name, event.payload.result)
        }),
      )

      unlisteners.push(
        await listen<string>('ai-chat-chunk', (event) => {
          if (!isCurrentRequest()) return
          onTool('', '')
          onChunk(event.payload)
        }),
      )

      unlisteners.push(
        await listen<void>('ai-chat-done', () => {
          if (!isCurrentRequest()) return
          cleanup()
          onDone()
        }),
      )

      unlisteners.push(
        await listen<string>('ai-error', (event) => {
          if (!isCurrentRequest()) return
          cleanup()
          onError(new Error(event.payload))
        }),
      )

      await invoke('ai_chat', { request: { messages, context } })
    } catch (err) {
      cleanup()
      onError(err instanceof Error ? err : new Error(String(err)))
    }
  },
}
