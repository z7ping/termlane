// 连接收藏/置顶
const STORAGE_KEY = 'xterminal_favorites'

export function getFavorites() {
  try { return JSON.parse(localStorage.getItem(STORAGE_KEY)) || [] }
  catch { return [] }
}

export function toggleFavorite(connectionId) {
  const favs = getFavorites()
  const idx = favs.indexOf(connectionId)
  if (idx >= 0) favs.splice(idx, 1)
  else favs.push(connectionId)
  localStorage.setItem(STORAGE_KEY, JSON.stringify(favs))
  return favs
}

export function isFavorite(connectionId) {
  return getFavorites().includes(connectionId)
}

export function sortWithFavorites(connections) {
  const favs = new Set(getFavorites())
  return [...connections].sort((a, b) => {
    const aFav = favs.has(a.id) ? 0 : 1
    const bFav = favs.has(b.id) ? 0 : 1
    return aFav - bFav
  })
}
