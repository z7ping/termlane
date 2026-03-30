// 懒加载工具（非活跃标签不渲染）
export function shouldRender(tabId, activeTabId, recentlyActive) {
  // Always render active tab
  if (tabId === activeTabId) return true
  // Render recently active tabs (within last 5 minutes)
  const lastActive = recentlyActive[tabId]
  if (lastActive && Date.now() - lastActive < 300000) return true
  // Don't render old inactive tabs
  return false
}

export function markActive(tabId, recentlyActive) {
  recentlyActive[tabId] = Date.now()
  return { ...recentlyActive }
}
