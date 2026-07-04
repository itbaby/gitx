/**
 * Read a clamped number from localStorage, falling back to `fallback` if the
 * key is absent, unparseable, or storage is unavailable. The returned value is
 * clamped to [min, max] so a stale stored value (e.g. from an older build with
 * different bounds) can never exceed the current limits.
 */
export const loadClampedNumber = (
  key: string,
  fallback: number,
  min: number,
  max: number,
): number => {
  try {
    const raw = localStorage.getItem(key)
    if (raw === null) return fallback
    const val = Number(raw)
    if (!Number.isFinite(val)) return fallback
    return Math.min(max, Math.max(min, val))
  } catch {
    // localStorage may be unavailable (e.g. disabled, or running outside a
    // browser in tests). Fall back to the default.
    return fallback
  }
}

/**
 * Persist a number to localStorage. Silently no-ops if storage is unavailable;
 * persistence is best-effort and must never break the UI.
 */
export const saveNumber = (key: string, value: number): void => {
  try {
    localStorage.setItem(key, String(value))
  } catch {
    // Ignore — best-effort.
  }
}
