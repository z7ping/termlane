// 快捷键工具函数
import { STORAGE_KEYS } from './storage-keys'

// 这里只列真正由应用监听、修改后能够生效的快捷键。
// Ctrl+C / Ctrl+L 属于终端控制键，不伪装成可配置应用快捷键。
export const SHORTCUT_ACTIONS = [
  { name: '搜索', defaultKey: 'Ctrl+Shift+F', group: '终端' },
  { name: '新建标签', defaultKey: 'Ctrl+T', group: '工作区' },
  { name: '关闭标签', defaultKey: 'Ctrl+W', group: '工作区' },
  { name: '切换侧边栏', defaultKey: 'Ctrl+B', group: '工作区' },
  { name: '设置', defaultKey: 'Ctrl+,', group: '工作区' },
  { name: '全屏', defaultKey: 'F11', group: '工作区' },
  { name: '帮助', defaultKey: '?', group: '工作区' },
] as const

type ShortcutMap = Record<string, string>
const DEFAULT_SHORTCUTS: ShortcutMap = Object.fromEntries(
  SHORTCUT_ACTIONS.map(action => [action.name, action.defaultKey]),
)

/** 获取快捷键配置。 */
export function getShortcut(actionName: string): string {
  return localStorage.getItem(`${STORAGE_KEYS.SHORTCUT_PREFIX}${actionName}`)
    || DEFAULT_SHORTCUTS[actionName]
    || ''
}

/** 保存一个快捷键配置。 */
export function setShortcut(actionName: string, key: string): void {
  localStorage.setItem(`${STORAGE_KEYS.SHORTCUT_PREFIX}${actionName}`, key)
  window.dispatchEvent(new CustomEvent('shortcut-changed', {
    detail: { name: actionName, key },
  }))
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
    const eventKey = event.key.toUpperCase()
    if (eventKey !== key) return false
    if (event.ctrlKey !== ctrl) return false
    if (event.altKey !== alt) return false
    if (event.metaKey !== meta) return false

    // `?`, `:`, `+` 等符号通常物理上需要 Shift 才能输入，但 event.key
    // 已经是最终符号。只有快捷键显式写了 Shift 时才强制匹配 Shift。
    const shiftedPrintableSymbol = !shift
      && event.shiftKey
      && key.length === 1
      && !/[A-Z0-9]/.test(key)
    if (!shiftedPrintableSymbol && event.shiftKey !== shift) return false

    return true
  }
}
