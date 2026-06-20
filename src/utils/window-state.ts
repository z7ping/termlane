// 窗口状态持久化
import { STORAGE_KEYS } from './storage-keys'

const STORAGE_KEY = STORAGE_KEYS.WINDOW_STATE

export interface WindowState {
  width: number
  height: number
  x: number
  y: number
  maximized: boolean
  savedAt: number
}

export function saveWindowState(): void {
  try {
    // In Tauri, would use window.__TAURI__.window
    // For browser, use localStorage
    const state: WindowState = {
      width: window.innerWidth,
      height: window.innerHeight,
      x: window.screenX,
      y: window.screenY,
      maximized: document.fullscreenElement !== null,
      savedAt: Date.now(),
    }
    localStorage.setItem(STORAGE_KEY, JSON.stringify(state))
  } catch {}
}

export function loadWindowState(): WindowState | null {
  try {
    return JSON.parse(localStorage.getItem(STORAGE_KEY) || '') || null
  } catch {
    return null
  }
}

export function clearWindowState(): void {
  localStorage.removeItem(STORAGE_KEY)
}

// Auto-save on resize
let saveTimer: ReturnType<typeof setTimeout> | null = null
export function startAutoSave(): void {
  window.addEventListener('resize', () => {
    if (saveTimer) clearTimeout(saveTimer)
    saveTimer = setTimeout(saveWindowState, 1000)
  })
}
