// 快捷键工具函数
import { STORAGE_KEYS } from './storage-keys'

type ShortcutMap = Record<string, string>

const DEFAULT_SHORTCUTS: ShortcutMap = {
  '搜索': 'Ctrl+Shift+F',
  '清屏': 'Ctrl+L',
  '中断': 'Ctrl+C',
  '新建标签': 'Ctrl+T',
  '关闭标签': 'Ctrl+W',
  '设置': 'Ctrl+,',
  '全屏': 'F11',
}

/** 获取快捷键配置。 */
export function getShortcut(actionName: string): string {
  return localStorage.getItem(`${STORAGE_KEYS.SHORTCUT_PREFIX}${actionName}`)
    || DEFAULT_SHORTCUTS[actionName]
    || ''
}

/** 解析快捷键字符串为按键检测函数。 */
export function parseShortcut(keyStr: string): (event: KeyboardEvent) => boolean {
  const parts = keyStr.toLowerCase().split('+')
  const ctrl = parts.includes('ctrl')
  const alt = parts.includes('alt')
  const shift = parts.includes('shift')
  const meta = parts.includes('meta')
  const key = parts.filter(part => !['ctrl', 'alt', 'shift', 'meta'].includes(part)).pop()?.toUpperCase() || ''

  return (event: KeyboardEvent) => {
    if (event.ctrlKey !== ctrl) return false
    if (event.altKey !== alt) return false
    if (event.shiftKey !== shift) return false
    if (event.metaKey !== meta) return false
    return event.key.toUpperCase() === key
  }
}
