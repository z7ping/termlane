// localStorage 键名常量 — 避免 magic string 散布
export const STORAGE_KEYS = {
  // 核心状态
  TABS: 'termlane_tabs',
  ACTIVE_TAB: 'termlane_active_tab',
  CONNECTIONS: 'termlane_connections',
  // 外观
  THEME: 'termlane-theme',
  FONT_SIZE: 'termlane-fontSize',
  SCROLLBACK: 'termlane-scrollback',
  // 代理（历史配置保留，1.0 当前不暴露代理入口）
  PROXY: 'termlane_proxy',
  // 快捷键前缀（历史上未带产品名，保持兼容）
  SHORTCUT_PREFIX: 'shortcut_',
  // 工具模块
  SCHEDULED_TASKS: 'termlane_scheduled_tasks',
  CONNECTION_TAGS: 'termlane_connection_tags',
  MACROS: 'termlane_macros',
  BOOKMARKS: 'termlane_bookmarks',
  QUICK_COMMANDS: 'termlane_quick_commands',
  NOTES_PREFIX: 'termlane_notes_',
  // 其他
  ONBOARDED: 'termlane_onboarded',
  LOCALE: 'termlane_locale',
  SKIPPED_VERSION: 'skipped_version',
  RECORDING_DIR: 'termlane_recordingDir',
} as const

const LEGACY_STORAGE_KEYS = {
  TABS: 'xterminal_tabs',
  ACTIVE_TAB: 'xterminal_active_tab',
  CONNECTIONS: 'xterminal_connections',
  THEME: 'xterminal-theme',
  FONT_SIZE: 'xterminal-fontSize',
  SCROLLBACK: 'xterminal-scrollback',
  PROXY: 'xterminal_proxy',
  SCHEDULED_TASKS: 'xterminal_scheduled_tasks',
  CONNECTION_TAGS: 'xterminal_connection_tags',
  MACROS: 'xterminal_macros',
  BOOKMARKS: 'xterminal_bookmarks',
  QUICK_COMMANDS: 'xterminal_quick_commands',
  ONBOARDED: 'xterminal_onboarded',
  LOCALE: 'xterminal_locale',
  RECORDING_DIR: 'xterminal_recordingDir',
} as const

const LEGACY_NOTES_PREFIX = 'xterminal_notes_'

/**
 * 将 XTerminal Pro 时期的普通 localStorage 数据迁移到 Termlane 命名空间。
 * 凭证不在这里处理：历史 `xterminal-pwd_*` 由 credentials.ts 迁移到安全存储。
 */
export function migrateLegacyStorageNamespace(): void {
  if (typeof localStorage === 'undefined') return

  for (const key of Object.keys(LEGACY_STORAGE_KEYS) as Array<keyof typeof LEGACY_STORAGE_KEYS>) {
    const legacyKey = LEGACY_STORAGE_KEYS[key]
    const currentKey = STORAGE_KEYS[key]
    const legacyValue = localStorage.getItem(legacyKey)

    if (legacyValue === null) continue
    if (localStorage.getItem(currentKey) === null) {
      localStorage.setItem(currentKey, legacyValue)
    }
    localStorage.removeItem(legacyKey)
  }

  const noteKeys: string[] = []
  for (let i = 0; i < localStorage.length; i++) {
    const key = localStorage.key(i)
    if (key?.startsWith(LEGACY_NOTES_PREFIX)) noteKeys.push(key)
  }

  for (const legacyKey of noteKeys) {
    const value = localStorage.getItem(legacyKey)
    if (value === null) continue
    const currentKey = STORAGE_KEYS.NOTES_PREFIX + legacyKey.slice(LEGACY_NOTES_PREFIX.length)
    if (localStorage.getItem(currentKey) === null) {
      localStorage.setItem(currentKey, value)
    }
    localStorage.removeItem(legacyKey)
  }
}

// 在任何读取 STORAGE_KEYS 的模块初始化前完成一次同步迁移。
migrateLegacyStorageNamespace()
