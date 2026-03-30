// 虚拟滚动工具（大列表优化）
export function calculateVisibleRange(scrollTop, itemHeight, containerHeight, totalItems) {
  const start = Math.floor(scrollTop / itemHeight)
  const visibleCount = Math.ceil(containerHeight / itemHeight)
  const end = Math.min(start + visibleCount + 1, totalItems)
  return { start: Math.max(0, start - 1), end }
}

export function getVisibleItems(items, start, end) {
  return items.slice(start, end).map((item, i) => ({
    ...item,
    _virtualIndex: start + i,
    _style: { position: 'absolute', top: (start + i) * 40 + 'px', width: '100%' }
  }))
}
