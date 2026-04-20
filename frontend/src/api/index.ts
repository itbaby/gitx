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
}

// ============================================================
// AI API (Tauri IPC + event listeners)
// ============================================================

interface ToolEvent {
  name: string
  display: string
}

export const aiApi = {
  async chat(
    messages: InputMessage[],
    context: ChatContext,
    onTool: (name: string, display: string) => void,
    onChunk: (text: string) => void,
    onDone: () => void,
    onError: (err: Error) => void,
  ): Promise<void> {
    const unlisteners: UnlistenFn[] = []

    try {
      unlisteners.push(
        await listen<ToolEvent>('ai-tool', (event) => {
          onTool(event.payload.name, event.payload.display)
        }),
      )

      unlisteners.push(
        await listen<string>('ai-chat-chunk', (event) => {
          onTool('', '')
          onChunk(event.payload)
        }),
      )

      unlisteners.push(
        await listen<void>('ai-chat-done', () => {
          onDone()
          unlisteners.forEach((u) => u())
        }),
      )

      unlisteners.push(
        await listen<string>('ai-error', (event) => {
          onError(new Error(event.payload))
          unlisteners.forEach((u) => u())
        }),
      )

      await invoke('ai_chat', { request: { messages, context } })
    } catch (err) {
      unlisteners.forEach((u) => u())
      onError(err instanceof Error ? err : new Error(String(err)))
    }
  },

  async analyze(diff: DiffInfo[], prompt: string): Promise<{ analysis: string }> {
    const analysis = await invoke<string>('ai_analyze', { request: { diff, prompt } })
    return { analysis }
  },

  async analyzeStream(
    diff: DiffInfo[],
    prompt: string,
    onChunk: (text: string) => void,
    onDone: () => void,
    onError: (err: Error) => void,
  ): Promise<void> {
    const unlisteners: UnlistenFn[] = []

    try {
      unlisteners.push(
        await listen<string>('ai-analyze-chunk', (event) => {
          onChunk(event.payload)
        }),
      )

      unlisteners.push(
        await listen<void>('ai-analyze-done', () => {
          onDone()
          unlisteners.forEach((u) => u())
        }),
      )

      unlisteners.push(
        await listen<string>('ai-error', (event) => {
          onError(new Error(event.payload))
          unlisteners.forEach((u) => u())
        }),
      )

      await invoke('ai_analyze_stream', { request: { diff, prompt } })
    } catch (err) {
      unlisteners.forEach((u) => u())
      onError(err instanceof Error ? err : new Error(String(err)))
    }
  },

  async parseIntent(input: string): Promise<{ intent: unknown; action: string }> {
    const intent = await invoke('parse_intent', { request: { input } })
    return { intent, action: '解析完成' }
  },
}
