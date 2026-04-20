import type { DiffInfo, CommitInfo, InputMessage, ChatContext } from '../types'

const BASE_URL = ''
const SSE_URL = '/api/ai/chat'

async function request<T>(url: string, options?: RequestInit): Promise<T> {
  const response = await fetch(BASE_URL + url, {
    headers: {
      'Content-Type': 'application/json',
    },
    ...options,
  })

  if (!response.ok) {
    const error = await response.json().catch(() => ({ error: '请求失败' }))
    throw new Error(error.error || `HTTP ${response.status}`)
  }

  return response.json()
}

// Git API

export const gitApi = {
  /** 打开仓库 */
  async openRepo(path: string): Promise<{ message: string; path: string }> {
    return request('/api/git/open', {
      method: 'POST',
      body: JSON.stringify({ path }),
    })
  },

  /** 获取分支列表 */
  async getBranches(): Promise<{ branches: string[] }> {
    return request('/api/git/branches')
  },

  /** 获取当前分支 */
  async getCurrentBranch(): Promise<{ current_branch: string }> {
    return request('/api/git/branches/current')
  },

  /** 获取提交历史 */
  async getCommits(branch?: string, limit = 20): Promise<{ commits: CommitInfo[] }> {
    const params = new URLSearchParams()
    if (branch) params.set('branch', branch)
    params.set('limit', String(limit))
    return request(`/api/git/commits?${params}`)
  },

  /** 获取提交间差异 */
  async getDiff(from: string, to: string): Promise<{ diff: DiffInfo[] }> {
    return request('/api/git/diff', {
      method: 'POST',
      body: JSON.stringify({ from, to }),
    })
  },

  /** 获取分支间差异 */
  async getBranchDiff(branch1: string, branch2: string): Promise<{ diff: DiffInfo[] }> {
    return request('/api/git/branch-diff', {
      method: 'POST',
      body: JSON.stringify({ branch1, branch2 }),
    })
  },

  /** 获取文件历史 */
  async getFileHistory(file: string, timeRange = '3d'): Promise<{ commits: CommitInfo[] }> {
    return request(`/api/git/file-history?file=${encodeURIComponent(file)}&timeRange=${encodeURIComponent(timeRange)}`)
  },
}

// AI API

export const aiApi = {
  /** Agent 对话（SSE 流式，统一入口） */
  chat(
    messages: InputMessage[],
    context: ChatContext,
    onTool: (name: string, display: string) => void,
    onChunk: (text: string) => void,
    onDone: () => void,
    onError: (err: Error) => void,
  ): void {
    console.log('[SSE] fetch start, url:', SSE_URL)
    fetch(SSE_URL, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ messages, context }),
    })
      .then((response) => {
        console.log('[SSE] response received, status:', response.status, 'type:', response.headers.get('content-type'))
        if (!response.ok) {
          throw new Error(`HTTP ${response.status}`)
        }
        const reader = response.body?.getReader()
        const decoder = new TextDecoder()
        let buffer = ''
        let currentEvent = ''
        let dataLines: string[] = []

        function flushData() {
          if (currentEvent === 'message' && dataLines.length > 0) {
            const text = dataLines.join('\n')
            if (text) onChunk(text)
          }
          dataLines = []
        }

        function read() {
          if (!reader) return
          reader.read().then(({ done, value }) => {
            if (done) {
              console.log('[SSE] stream done')
              flushData()
              onDone()
              return
            }

            buffer += decoder.decode(value, { stream: true })

            const lines = buffer.split('\n')
            buffer = lines.pop() || ''

            for (const line of lines) {
              if (line.startsWith('event:')) {
                flushData()
                currentEvent = line.slice(6).trim()
              } else if (line.startsWith('data:')) {
                const data = line.slice(5)
                if (currentEvent === 'done') {
                  flushData()
                  onDone()
                  return
                }
                if (currentEvent === 'error') {
                  onError(new Error(data.trim()))
                  return
                }
                if (currentEvent === 'tool') {
                  try {
                    const tool = JSON.parse(data)
                    onTool(tool.name, tool.display)
                  } catch { /* ignore */ }
                  continue
                }
                if (currentEvent === 'message') {
                  dataLines.push(data)
                }
              } else {
                flushData()
              }
            }

            read()
          }).catch((err) => {
            console.error('[SSE] read error:', err)
            onError(err)
          })
        }

        read()
      }).catch((err) => {
        console.error('[SSE] fetch error:', err)
        onError(err)
      })
  },

  /** 分析差异（非流式，保留兼容） */
  async analyze(diff: DiffInfo[], prompt: string): Promise<{ analysis: string }> {
    return request('/api/ai/analyze', {
      method: 'POST',
      body: JSON.stringify({ diff, prompt }),
    })
  },

  /** 流式分析差异（SSE） */
  analyzeStream(diff: DiffInfo[], prompt: string, onChunk: (text: string) => void, onDone: () => void, onError: (err: Error) => void): void {
    fetch('/api/ai/analyze-stream', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ diff, prompt }),
    })
      .then((response) => {
        if (!response.ok) {
          throw new Error(`HTTP ${response.status}`)
        }
        const reader = response.body?.getReader()
        const decoder = new TextDecoder()
        let buffer = ''

        function read() {
          if (!reader) return
          reader.read().then(({ done, value }) => {
            if (done) {
              onDone()
              return
            }

            buffer += decoder.decode(value, { stream: true })

            // 解析 SSE 事件
            const lines = buffer.split('\n')
            buffer = lines.pop() || ''

            for (const line of lines) {
              if (line.startsWith('data:')) {
                const data = line.slice(5).trim()
                if (data === '"[DONE]"' || data === '[DONE]') {
                  onDone()
                  return
                }
                // 解析 JSON 字符串
                try {
                  const parsed = JSON.parse(data)
                  if (typeof parsed === 'string') {
                    onChunk(parsed)
                  }
                } catch {
                  // 可能是纯文本
                  onChunk(data)
                }
              }
            }

            read()
          }).catch(onError)
        }

        read()
      })
      .catch(onError)
  },

  /** 解析自然语言意图（保留兼容） */
  async parseIntent(input: string): Promise<{ intent: any; action: string }> {
    return request('/api/ai/parse-intent', {
      method: 'POST',
      body: JSON.stringify({ input }),
    })
  },
}
