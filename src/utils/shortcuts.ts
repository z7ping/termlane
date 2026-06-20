// 快捷键工具函数

type ShortcutMap = Record<string, string>

const DEFAULT_SHORTCUTS: ShortcutMap = {
  '搜索': 'Ctrl+Shift+F',
  '清屏': 'Ctrl+L',
  '中断': 'Ctrl+C',
  '新建标签': 'Ctrl+T',
  '关闭标签': 'Ctrl+W',
  '分屏': 'Ctrl+Shift+D',
  '全屏': 'F11',
}

/**
 * 获取快捷键配置
 * @param actionName - 动作名称
 * @returns 快捷键字符串 (如 'Ctrl+Shift+F')
 */
export function getShortcut(actionName: string): string {
  return localStorage.getItem(`shortcut_${actionName}`) || DEFAULT_SHORTCUTS[actionName] || ''
}

/**
 * 解析快捷键字符串为按键检测函数
 * @param keyStr - 快捷键字符串 (如 'Ctrl+Shift+F')
 * @returns 按键检测函数
 */
export function parseShortcut(keyStr: string): (e: KeyboardEvent) => boolean {
  const parts = keyStr.toLowerCase().split('+')
  const ctrl = parts.includes('ctrl')
  const alt = parts.includes('alt')
  const shift = parts.includes('shift')
  const meta = parts.includes('meta')
  const key = parts.filter(p => !['ctrl', 'alt', 'shift', 'meta'].includes(p)).pop()?.toUpperCase() || ''
  return (e: KeyboardEvent) => {
    if (e.ctrlKey !== ctrl) return false
    if (e.altKey !== alt) return false
    if (e.shiftKey !== shift) return false
    if (e.metaKey !== meta) return false
    return e.key.toUpperCase() === key
  }
}
