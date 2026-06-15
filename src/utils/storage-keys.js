// localStorage 键名常量 — 避免 magic string 散布
export const STORAGE_KEYS = {
  // 核心状态
  TABS: 'xterminal_tabs',
  ACTIVE_TAB: 'xterminal_active_tab',
  CONNECTIONS: 'xterminal_connections',
  // 外观
  THEME: 'xterminal-theme',
  FONT_SIZE: 'xterminal-fontSize',
  SCROLLBACK: 'xterminal-scrollback',
  CURSOR_BLINK: 'xterminal-cursorBlink',
  SSH_TIMEOUT: 'xterminal-sshTimeout',
  // 代理
  PROXY: 'xterminal_proxy',
  // 快捷键前缀
  SHORTCUT_PREFIX: 'shortcut_',
  // 其他
  ONBOARDED: 'xterminal_onboarded',
  LOCALE: 'xterminal_locale',
  SKIPPED_VERSION: 'skipped_version',
  RECORDING_DIR: 'xterminal_recordingDir',
}
