// SFTP 增强功能工具

export type FileType = 'image' | 'text' | 'code' | 'archive' | 'document' | 'unknown'

export interface FileItem {
  name: string
  path: string
  size: number
  isDir: boolean
  modified: string
  permissions: string
}

export interface ContextMenuItem {
  id: string
  label?: string
  icon?: string
  divider?: boolean
  needsFile?: boolean
  needsPreviewable?: boolean
  danger?: boolean
}

// 文件 MIME 类型检测
export function getFileType(filename: string): FileType {
  const ext = filename.split('.').pop()?.toLowerCase()
  const types: Record<string, FileType> = {
    // Images
    png: 'image', jpg: 'image', jpeg: 'image', gif: 'image', svg: 'image', webp: 'image',
    // Text
    txt: 'text', md: 'text', log: 'text', json: 'text', yaml: 'text', yml: 'text', xml: 'text',
    js: 'code', ts: 'code', py: 'code', rs: 'code', go: 'code', java: 'code', c: 'code', cpp: 'code',
    html: 'code', css: 'code', sh: 'code', bat: 'code',
    // Archives
    zip: 'archive', tar: 'archive', gz: 'archive', '7z': 'archive', rar: 'archive',
    // Documents
    pdf: 'document', doc: 'document', docx: 'document', xls: 'document', xlsx: 'document',
  }
  return types[ext || ''] || 'unknown'
}

export function isPreviewable(filename: string): boolean {
  const type = getFileType(filename)
  return ['image', 'text', 'code'].includes(type)
}

// 批量选择管理
export class SelectionManager {
  private selected: Set<string>
  private lastSelected: string | null

  constructor() {
    this.selected = new Set()
    this.lastSelected = null
  }

  toggle(path: string): string[] {
    if (this.selected.has(path)) this.selected.delete(path)
    else this.selected.add(path)
    this.lastSelected = path
    return this.getSelected()
  }

  rangeSelect(startPath: string, endPath: string, items: FileItem[]): string[] {
    const startIdx = items.findIndex(i => i.path === startPath)
    const endIdx = items.findIndex(i => i.path === endPath)
    if (startIdx < 0 || endIdx < 0) return this.getSelected()
    const [from, to] = startIdx < endIdx ? [startIdx, endIdx] : [endIdx, startIdx]
    for (let i = from; i <= to; i++) {
      this.selected.add(items[i].path)
    }
    this.lastSelected = endPath
    return this.getSelected()
  }

  selectAll(items: FileItem[]): string[] {
    items.forEach(i => this.selected.add(i.path))
    return this.getSelected()
  }

  clear(): string[] {
    this.selected.clear()
    this.lastSelected = null
    return []
  }

  getSelected(): string[] {
    return [...this.selected]
  }

  isSelected(path: string): boolean {
    return this.selected.has(path)
  }

  count(): number {
    return this.selected.size
  }
}

// 右键菜单定义
export const fileContextMenu: ContextMenuItem[] = [
  { id: 'open', label: '打开', icon: '📂', needsFile: true },
  { id: 'preview', label: '预览', icon: '👁️', needsFile: true, needsPreviewable: true },
  { id: 'download', label: '下载', icon: '⬇️', needsFile: true },
  { id: 'divider1', divider: true },
  { id: 'rename', label: '重命名', icon: '✏️', needsFile: true },
  { id: 'delete', label: '删除', icon: '🗑️', needsFile: true, danger: true },
  { id: 'divider2', divider: true },
  { id: 'permissions', label: '权限', icon: '🔒', needsFile: true },
  { id: 'details', label: '详情', icon: 'ℹ️', needsFile: true },
]
