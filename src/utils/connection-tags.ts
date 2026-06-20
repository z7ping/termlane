// 连接标签管理
import { STORAGE_KEYS } from './storage-keys'

export interface Tag {
  id: string
  name: string
  color: string
}

const STORAGE_KEY = STORAGE_KEYS.CONNECTION_TAGS

const DEFAULT_TAGS: Tag[] = [
  { id: 'prod', name: '生产', color: 'red' },
  { id: 'test', name: '测试', color: 'yellow' },
  { id: 'dev', name: '开发', color: 'green' },
  { id: 'db', name: '数据库', color: 'blue' },
]

export function getTags(): Tag[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (!raw) return [...DEFAULT_TAGS]
    return JSON.parse(raw) || [...DEFAULT_TAGS]
  } catch { return [] }
}

export function addTag(name: string, color: string): Tag[] {
  const tags = getTags()
  tags.push({ id: `tag_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`, name, color })
  localStorage.setItem(STORAGE_KEY, JSON.stringify(tags))
  return tags
}

export function removeTag(id: string): Tag[] {
  const tags = getTags().filter(t => t.id !== id)
  localStorage.setItem(STORAGE_KEY, JSON.stringify(tags))
  return tags
}

export function getTagColor(tagId: string): string {
  const tag = getTags().find(t => t.id === tagId)
  return tag?.color || 'none'
}
