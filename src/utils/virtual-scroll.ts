const ITEM_HEIGHT = 40 // 每行高度(px)

export interface VirtualItem<T> {
  _virtualIndex: number
  _style: Record<string, string>
}

// 虚拟滚动工具（大列表优化）
export function calculateVisibleRange(
  scrollTop: number,
  itemHeight: number,
  containerHeight: number,
  totalItems: number
): { start: number; end: number } {
  const start = Math.floor(scrollTop / itemHeight)
  const visibleCount = Math.ceil(containerHeight / itemHeight)
  const end = Math.min(start + visibleCount + 1, totalItems)
  return { start: Math.max(0, start - 1), end }
}

export function getVisibleItems<T extends Record<string, unknown>>(
  items: T[],
  start: number,
  end: number
): (T & VirtualItem<T>)[] {
  return items.slice(start, end).map((item, i) => ({
    ...item,
    _virtualIndex: start + i,
    _style: { position: 'absolute', top: (start + i) * ITEM_HEIGHT + 'px', width: '100%' }
  }))
}
