import { describe, it, expect } from 'vitest'
import { friendlyError } from './error'

describe('friendlyError', () => {
  it('stringifies non-Error inputs', () => {
    expect(friendlyError('boom')).toBe('boom')
    expect(friendlyError(42)).toBe('42')
  })

  it('maps uninitialized-client error to a config hint', () => {
    expect(friendlyError(new Error('AI 客户端未初始化'))).toMatch(/OPENAI_API_KEY/)
  })

  it('maps 401 to invalid key message', () => {
    expect(friendlyError(new Error('AI API 错误 (401): Unauthorized'))).toBe(
      'API Key 无效，请检查配置',
    )
  })

  it('maps 429 to rate-limit message', () => {
    expect(friendlyError(new Error('AI API 错误 (429): Too Many Requests'))).toBe(
      '请求过于频繁，请稍后再试',
    )
  })

  it('maps other AI API errors to generic service message', () => {
    expect(friendlyError(new Error('AI API 错误 (500): Internal'))).toBe(
      'AI 服务暂时不可用，请稍后再试',
    )
  })

  it('maps connection failures to network message', () => {
    expect(friendlyError(new Error('AI API 请求失败: connection reset'))).toBe(
      '无法连接到 AI 服务，请检查网络',
    )
  })

  it('returns the original message when no pattern matches', () => {
    expect(friendlyError(new Error('something unusual'))).toBe('something unusual')
  })
})
