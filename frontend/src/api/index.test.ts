import { describe, it, expect, beforeEach, vi } from 'vitest'

// ============================================================
// Mock the Tauri runtime APIs so we can test the event wiring in isolation.
// ============================================================

type Handler = (event: { payload: unknown }) => void

// Captured event handlers per event name, so tests can dispatch events and
// assert cleanup (listeners are removed on done/error).
const handlers = new Map<string, Set<Handler>>()

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(async (event: string, handler: Handler) => {
    if (!handlers.has(event)) handlers.set(event, new Set())
    handlers.get(event)!.add(handler)
    // Return the unlisten function Tauri's real `listen` resolves with.
    return () => {
      handlers.get(event)?.delete(handler)
    }
  }),
}))

// Invoke calls are held in a queue so each test can resolve or reject them,
// simulating the backend accepting the stream request.
const invokeQueue: Array<{
  resolve: (v: unknown) => void
  reject: (e: unknown) => void
}> = []
const invokeCalls: { cmd: string; args?: unknown }[] = []

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn((cmd: string, args?: unknown) => {
    invokeCalls.push({ cmd, args })
    return new Promise((resolve, reject) => {
      invokeQueue.push({ resolve, reject })
    })
  }),
}))

// --- helpers ---

/** Dispatch an event to every currently-registered handler for `event`. */
const emit = (event: string, payload: unknown = undefined) => {
  // Spread into an array first: a `done` handler calls cleanup, which mutates
  // the set during iteration. Snapshotting avoids skipping handlers.
  for (const h of [...(handlers.get(event) ?? [])]) h({ payload })
}
const handlerCount = (event: string) => handlers.get(event)?.size ?? 0

// Import the module under test AFTER the mocks are registered.
import { aiApi } from './index'

beforeEach(() => {
  handlers.clear()
  invokeQueue.length = 0
  invokeCalls.length = 0
})

// ============================================================
// analyzeStream
// ============================================================

describe('aiApi.analyzeStream', () => {
  it('delivers chunk events to onChunk and cleans up on done', async () => {
    const onChunk = vi.fn()
    const onDone = vi.fn()
    const onError = vi.fn()

    const p = aiApi.analyzeStream([], 'explain this', onChunk, onDone, onError)
    // Wait for the three listeners to attach (listen is awaited in-order).
    await vi.waitFor(() => {
      expect(handlerCount('ai-analyze-chunk')).toBe(1)
      expect(handlerCount('ai-analyze-done')).toBe(1)
      expect(handlerCount('ai-error')).toBe(1)
    })

    emit('ai-analyze-chunk', 'hello')
    emit('ai-analyze-chunk', ' world')
    expect(onChunk).toHaveBeenCalledTimes(2)
    expect(onChunk).toHaveBeenLastCalledWith(' world')

    // done event must trigger onDone and remove all listeners (cleanup).
    emit('ai-analyze-done')
    expect(onDone).toHaveBeenCalledTimes(1)
    expect(handlerCount('ai-analyze-chunk')).toBe(0)
    expect(handlerCount('ai-analyze-done')).toBe(0)
    expect(onError).not.toHaveBeenCalled()

    // Let the underlying invoke settle so the promise resolves.
    invokeQueue[0].resolve(undefined)
    await p
  })

  it('ignores events from a stale request after a newer one starts', async () => {
    const onChunk1 = vi.fn()
    const onChunk2 = vi.fn()

    aiApi.analyzeStream([], 'first', onChunk1, vi.fn(), vi.fn())
    await vi.waitFor(() => expect(handlerCount('ai-analyze-chunk')).toBe(1))

    aiApi.analyzeStream([], 'second', onChunk2, vi.fn(), vi.fn())
    await vi.waitFor(() => expect(handlerCount('ai-analyze-chunk')).toBe(2))

    // A chunk arrives now: only the most recent request's callback fires —
    // request 1's closure sees a bumped module counter and bails.
    emit('ai-analyze-chunk', 'late')
    expect(onChunk2).toHaveBeenCalledWith('late')
    expect(onChunk1).not.toHaveBeenCalled()
  })

  it('reports the error and cleans up when invoke rejects', async () => {
    const onError = vi.fn()
    const onDone = vi.fn()

    const p = aiApi.analyzeStream([], 'q', vi.fn(), onDone, onError)
    // Wait for invoke to be called (happens after all 3 listeners attach),
    // so the rejectable promise is sitting in the queue.
    await vi.waitFor(() => expect(invokeQueue.length).toBe(1))

    invokeQueue[0].reject(new Error('backend down'))
    await p

    expect(onError).toHaveBeenCalledTimes(1)
    expect((onError.mock.calls[0][0] as Error).message).toBe('backend down')
    // Rejection path must tear down listeners and must not call onDone.
    expect(handlerCount('ai-analyze-chunk')).toBe(0)
    expect(onDone).not.toHaveBeenCalled()
  })

  it('forwards the diff and prompt to the backend command', async () => {
    const p = aiApi.analyzeStream(
      [{ file: 'a.rs', patch: '+x', added: 1, deleted: 0 }],
      'review',
      vi.fn(),
      vi.fn(),
      vi.fn(),
    )
    await vi.waitFor(() => expect(invokeCalls.length).toBe(1))
    expect(invokeCalls[0].cmd).toBe('ai_analyze_stream')
    expect(invokeCalls[0].args).toEqual({
      request: {
        diff: [{ file: 'a.rs', patch: '+x', added: 1, deleted: 0 }],
        prompt: 'review',
      },
    })
    invokeQueue[0].resolve(undefined)
    await p
  })
})

// ============================================================
// chat
// ============================================================

describe('aiApi.chat', () => {
  it('routes tool, tool-result, chunk, and done events to the right callbacks', async () => {
    const onTool = vi.fn()
    const onToolResult = vi.fn()
    const onChunk = vi.fn()
    const onDone = vi.fn()

    const p = aiApi.chat(
      [{ role: 'user', content: 'hi' }],
      { base_branch: 'a', compare_branch: 'b', has_diff: false },
      onTool,
      onToolResult,
      onChunk,
      onDone,
      vi.fn(),
    )
    await vi.waitFor(() => {
      expect(handlerCount('ai-tool')).toBe(1)
      expect(handlerCount('ai-chat-chunk')).toBe(1)
    })

    emit('ai-tool', { name: 'get_branches', display: 'fetching...' })
    expect(onTool).toHaveBeenCalledWith('get_branches', 'fetching...')

    emit('ai-tool-result', { name: 'get_branches', result: 'main\ndev' })
    expect(onToolResult).toHaveBeenCalledWith('get_branches', 'main\ndev')

    // A chat chunk must clear the tool status (onTool('','')) then deliver text.
    emit('ai-chat-chunk', 'answer')
    expect(onTool).toHaveBeenCalledWith('', '')
    expect(onChunk).toHaveBeenCalledWith('answer')

    emit('ai-chat-done')
    expect(onDone).toHaveBeenCalledTimes(1)
    expect(handlerCount('ai-tool')).toBe(0)

    invokeQueue[0].resolve(undefined)
    await p
  })

  it('ignores stale events from a previous chat session', async () => {
    const onChunk1 = vi.fn()
    const onChunk2 = vi.fn()
    aiApi.chat([], { base_branch: '', compare_branch: '', has_diff: false }, vi.fn(), vi.fn(), onChunk1, vi.fn(), vi.fn())
    await vi.waitFor(() => expect(handlerCount('ai-chat-chunk')).toBe(1))
    aiApi.chat([], { base_branch: '', compare_branch: '', has_diff: false }, vi.fn(), vi.fn(), onChunk2, vi.fn(), vi.fn())
    await vi.waitFor(() => expect(handlerCount('ai-chat-chunk')).toBe(2))

    emit('ai-chat-chunk', 'x')
    expect(onChunk2).toHaveBeenCalledWith('x')
    expect(onChunk1).not.toHaveBeenCalled()
  })

  it('calls onError when invoke rejects', async () => {
    const onError = vi.fn()
    const p = aiApi.chat([], { base_branch: '', compare_branch: '', has_diff: false }, vi.fn(), vi.fn(), vi.fn(), vi.fn(), onError)
    await vi.waitFor(() => expect(invokeCalls.length).toBe(1))
    invokeQueue[0].reject(new Error('nope'))
    await p
    expect(onError).toHaveBeenCalledTimes(1)
    expect((onError.mock.calls[0][0] as Error).message).toBe('nope')
  })
})

// ============================================================
// cancel
// ============================================================

describe('aiApi.cancel', () => {
  it('invokes the ai_cancel command', async () => {
    const p = aiApi.cancel()
    await vi.waitFor(() => expect(invokeCalls.length).toBe(1))
    expect(invokeCalls[0].cmd).toBe('ai_cancel')
    invokeQueue[0].resolve(undefined)
    await expect(p).resolves.toBeUndefined()
  })

  it('swallows a backend rejection (cancel is best-effort)', async () => {
    const p = aiApi.cancel()
    await vi.waitFor(() => expect(invokeQueue.length).toBe(1))
    const warn = vi.spyOn(console, 'warn').mockImplementation(() => {})
    invokeQueue[0].reject(new Error('nothing to cancel'))
    await expect(p).resolves.toBeUndefined()
    expect(warn).toHaveBeenCalled()
    warn.mockRestore()
  })
})
