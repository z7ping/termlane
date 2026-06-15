// 连接收藏/置顶 — 使用加密存储
import { secureStore } from './secure-store-browser.js'

const STORAGE_KEY = 'favorites'

export async function getFavorites() {
  try {
    return await secureStore.get(STORAGE_KEY) || []
  } catch { return [] }
}

export async function toggleFavorite(connectionId) {
  const favs = await getFavorites()
  const idx = favs.indexOf(connectionId)
  if (idx >= 0) favs.splice(idx, 1)
  else favs.push(connectionId)
  await secureStore.set(STORAGE_KEY, favs)
  return favs
}

export async function isFavorite(connectionId) {
  const favs = await getFavorites()
  return favs.includes(connectionId)
}

export async function sortWithFavorites(connections) {
  const favs = new Set(await getFavorites())
  return [...connections].sort((a, b) => {
    const aFav = favs.has(a.id) ? 0 : 1
    const bFav = favs.has(b.id) ? 0 : 1
    return aFav - bFav
  })
}
