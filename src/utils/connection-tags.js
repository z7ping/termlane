// 连接标签管理
const STORAGE_KEY = 'xterminal_connection_tags'

export function getTags() {
  try { return JSON.parse(localStorage.getItem(STORAGE_KEY)) || [
    { id: 'prod', name: '生产', color: 'red' },
    { id: 'test', name: '测试', color: 'yellow' },
    { id: 'dev', name: '开发', color: 'green' },
    { id: 'db', name: '数据库', color: 'blue' },
  ]} catch { return [] }
}

export function addTag(name, color) {
  const tags = getTags()
  tags.push({ id: `tag_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`, name, color })
  localStorage.setItem(STORAGE_KEY, JSON.stringify(tags))
  return tags
}

export function removeTag(id) {
  const tags = getTags().filter(t => t.id !== id)
  localStorage.setItem(STORAGE_KEY, JSON.stringify(tags))
  return tags
}

export function getTagColor(tagId) {
  const tag = getTags().find(t => t.id === tagId)
  return tag?.color || 'none'
}
