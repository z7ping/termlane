// 窗口状态持久化
const STORAGE_KEY = 'xterminal_window_state'

export function saveWindowState() {
  try {
    // In Tauri, would use window.__TAURI__.window
    // For browser, use localStorage
    const state = {
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

export function loadWindowState() {
  try {
    return JSON.parse(localStorage.getItem(STORAGE_KEY))
  } catch {
    return null
  }
}

export function clearWindowState() {
  localStorage.removeItem(STORAGE_KEY)
}

// Auto-save on resize
let saveTimer = null
export function startAutoSave() {
  window.addEventListener('resize', () => {
    clearTimeout(saveTimer)
    saveTimer = setTimeout(saveWindowState, 1000)
  })
}
