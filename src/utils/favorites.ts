// 连接收藏/置顶 — 使用加密存储
import { secureStore } from './secure-store-browser'

const STORAGE_KEY = 'favorites'

export interface Connection {
  id: string
  [key: string]: unknown
}

export async function getFavorites(): Promise<string[]> {
  try {
    const stored = await secureStore.get<string[]>(STORAGE_KEY)
    return stored ? [...stored] : []
  } catch {
    return []
  }
}

export async function toggleFavorite(connectionId: string): Promise<string[]> {
  const current = await getFavorites()
  const exists = current.includes(connectionId)
  const next = exists
    ? current.filter(id => id !== connectionId)
    : [...current, connectionId]

  await secureStore.set(STORAGE_KEY, next)
  return [...next]
}

export async function isFavorite(connectionId: string): Promise<boolean> {
  const favs = await getFavorites()
  return favs.includes(connectionId)
}

export async function sortWithFavorites<T extends Connection>(connections: T[]): Promise<T[]> {
  const favs = new Set(await getFavorites())
  return [...connections].sort((a, b) => {
    const aFav = favs.has(a.id) ? 0 : 1
    const bFav = favs.has(b.id) ? 0 : 1
    return aFav - bFav
  })
}
