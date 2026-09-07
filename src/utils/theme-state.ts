import { STORAGE_KEYS } from './storage-keys'

export const THEME_VALUES = ['dark', 'light', 'nord'] as const
export type ThemeValue = typeof THEME_VALUES[number]

export function getStoredTheme(): ThemeValue {
  const stored = localStorage.getItem(STORAGE_KEYS.THEME)
  return THEME_VALUES.includes(stored as ThemeValue) ? stored as ThemeValue : 'dark'
}

export function applyTheme(theme: ThemeValue): void {
  document.documentElement.setAttribute('data-theme', theme)
  localStorage.setItem(STORAGE_KEYS.THEME, theme)
  window.dispatchEvent(new CustomEvent('xterminal-theme-changed', { detail: theme }))
}

export function nextTheme(theme: ThemeValue): ThemeValue {
  const index = THEME_VALUES.indexOf(theme)
  return THEME_VALUES[(index + 1) % THEME_VALUES.length]
}
