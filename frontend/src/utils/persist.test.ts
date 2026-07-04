import { describe, it, expect, beforeEach, vi } from 'vitest'
import { loadClampedNumber, saveNumber } from './persist'

// Minimal localStorage mock. The vitest suite runs in the node environment
// (no global localStorage), so we stub one for these tests.
function makeStorage() {
  const store = new Map<string, string>()
  return {
    getItem: (k: string) => (store.has(k) ? store.get(k)! : null),
    setItem: (k: string, v: string) => { store.set(k, String(v)) },
    removeItem: (k: string) => { store.delete(k) },
    clear: () => { store.clear() },
    _store: store,
  }
}

let storage: ReturnType<typeof makeStorage>

beforeEach(() => {
  storage = makeStorage()
  vi.stubGlobal('localStorage', storage)
})

describe('loadClampedNumber', () => {
  it('returns fallback when the key is absent', () => {
    expect(loadClampedNumber('w', 260, 180, 500)).toBe(260)
  })

  it('returns the stored value when within bounds', () => {
    storage.setItem('w', '300')
    expect(loadClampedNumber('w', 260, 180, 500)).toBe(300)
  })

  it('clamps a stale value that exceeds the current max', () => {
    // Simulate an older build that allowed 700; current max is 500.
    storage.setItem('w', '700')
    expect(loadClampedNumber('w', 260, 180, 500)).toBe(500)
  })

  it('clamps a stale value below the current min', () => {
    storage.setItem('w', '50')
    expect(loadClampedNumber('w', 260, 180, 500)).toBe(180)
  })

  it('falls back on a non-numeric stored value', () => {
    storage.setItem('w', 'not-a-number')
    expect(loadClampedNumber('w', 260, 180, 500)).toBe(260)
  })

  it('falls back when localStorage.getItem throws', () => {
    vi.stubGlobal('localStorage', {
      getItem: () => { throw new Error('denied') },
      setItem: () => {},
    })
    expect(loadClampedNumber('w', 260, 180, 500)).toBe(260)
  })
})

describe('saveNumber', () => {
  it('writes the value as a string', () => {
    saveNumber('w', 340)
    expect(storage._store.get('w')).toBe('340')
  })

  it('does not throw when localStorage.setItem throws', () => {
    vi.stubGlobal('localStorage', {
      getItem: () => null,
      setItem: () => { throw new Error('denied') },
    })
    expect(() => saveNumber('w', 340)).not.toThrow()
  })
})
