// 懒加载工具（非活跃标签不渲染）

export type RecentlyActive = Record<string, number>

export function shouldRender(tabId: string, activeTabId: string, recentlyActive: RecentlyActive): boolean {
  // Always render active tab
  if (tabId === activeTabId) return true
  // Render recently active tabs (within last 5 minutes)
  const lastActive = recentlyActive[tabId]
  if (lastActive && Date.now() - lastActive < 5 * 60 * 1000) return true
  // Don't render old inactive tabs
  return false
}

export function markActive(tabId: string, recentlyActive: RecentlyActive): RecentlyActive {
  recentlyActive[tabId] = Date.now()
  return { ...recentlyActive }
}
