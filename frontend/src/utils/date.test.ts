import { describe, it, expect } from 'vitest'
import { formatTimestamp, formatFullDate } from './date'

describe('formatTimestamp', () => {
  it('returns 未知时间 for non-numeric input', () => {
    expect(formatTimestamp('not-a-number')).toBe('未知时间')
  })

  it('returns 未知时间 for NaN timestamp', () => {
    expect(formatTimestamp('NaN')).toBe('未知时间')
  })

  it('treats future timestamps (clock skew) as 刚刚', () => {
    const future = Math.floor(Date.now() / 1000) + 60
    expect(formatTimestamp(String(future))).toBe('刚刚')
  })

  it('returns 刚刚 for the current time', () => {
    const now = Math.floor(Date.now() / 1000)
    expect(formatTimestamp(String(now))).toBe('刚刚')
  })

  it('formats minutes-ago bucket', () => {
    const fiveMinAgo = Math.floor(Date.now() / 1000) - 5 * 60
    expect(formatTimestamp(String(fiveMinAgo))).toBe('5 分钟前')
  })

  it('formats hours-ago bucket', () => {
    const threeHrsAgo = Math.floor(Date.now() / 1000) - 3 * 3600
    expect(formatTimestamp(String(threeHrsAgo))).toBe('3 小时前')
  })

  it('formats days-ago bucket', () => {
    const twoDaysAgo = Math.floor(Date.now() / 1000) - 2 * 86400
    expect(formatTimestamp(String(twoDaysAgo))).toBe('2 天前')
  })
})

describe('formatFullDate', () => {
  it('returns 未知时间 for invalid input', () => {
    expect(formatFullDate('garbage')).toBe('未知时间')
  })

  it('returns a localized date string for valid input', () => {
    const ts = Math.floor(Date.now() / 1000) - 86400
    const out = formatFullDate(String(ts))
    // Should contain a year (20xx) and not the error fallback.
    expect(out).not.toBe('未知时间')
    expect(out).toMatch(/20\d{2}/)
  })
})
