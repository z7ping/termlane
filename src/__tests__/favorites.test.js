// src/__tests__/favorites.test.js - 连接收藏/置顶功能测试
import { vi } from 'vitest'
import { getFavorites, toggleFavorite, isFavorite, sortWithFavorites } from '../utils/favorites.ts'

// Mock secureStore used by favorites.js
const mockStore = {}
vi.mock('../utils/secure-store-browser', () => ({
  secureStore: {
    get: vi.fn(async (key) => mockStore[key] ?? null),
    set: vi.fn(async (key, value) => { mockStore[key] = value }),
  },
}))

describe('连接收藏/置顶功能', () => {
  beforeEach(() => {
    // 每个测试前清理测试数据
    for (const k of Object.keys(mockStore)) delete mockStore[k]
  })

  describe('getFavorites', () => {
    test('初始化时返回空数组', async () => {
      const favs = await getFavorites()
      expect(favs).toEqual([])
    })

    test('返回已存储的收藏列表', async () => {
      const testFavs = ['conn1', 'conn2', 'conn3']
      mockStore['favorites'] = testFavs

      const favs = await getFavorites()
      expect(favs).toEqual(testFavs)
    })

    test('store异常时返回空数组', async () => {
      const { secureStore } = await import('../utils/secure-store-browser.ts')
      secureStore.get.mockRejectedValueOnce(new Error('corrupt'))

      const favs = await getFavorites()
      expect(favs).toEqual([])
    })

    test('没有存储时返回空数组', async () => {
      const favs = await getFavorites()
      expect(favs).toEqual([])
    })
  })

  describe('toggleFavorite', () => {
    test('添加新收藏', async () => {
      const result = await toggleFavorite('conn1')
      expect(result).toContain('conn1')
      expect(result).toHaveLength(1)
    })

    test('移除已有收藏', async () => {
      await toggleFavorite('conn1')
      const result = await toggleFavorite('conn1')
      expect(result).not.toContain('conn1')
      expect(result).toHaveLength(0)
    })

    test('多次切换', async () => {
      expect(await toggleFavorite('conn1')).toContain('conn1')
      expect(await toggleFavorite('conn1')).not.toContain('conn1')
      expect(await toggleFavorite('conn1')).toContain('conn1')
    })

    test('支持多个收藏', async () => {
      await toggleFavorite('conn1')
      await toggleFavorite('conn2')
      await toggleFavorite('conn3')

      const favs = await getFavorites()
      expect(favs).toEqual(['conn1', 'conn2', 'conn3'])
    })

    test('移除其中一个收藏不影响其他', async () => {
      await toggleFavorite('conn1')
      await toggleFavorite('conn2')
      await toggleFavorite('conn3')

      await toggleFavorite('conn2')

      const favs = await getFavorites()
      expect(favs).toEqual(['conn1', 'conn3'])
      expect(favs).toHaveLength(2)
    })

    test('重复添加同一个收藏只保留一个', async () => {
      await toggleFavorite('conn1')
      await toggleFavorite('conn1')
      await toggleFavorite('conn1')

      const favs = await getFavorites()
      expect(favs).toEqual(['conn1'])
      expect(favs).toHaveLength(1)
    })

    test('返回更新后的收藏列表', async () => {
      const result1 = await toggleFavorite('conn1')
      const result2 = await toggleFavorite('conn2')

      expect(result1).toEqual(['conn1'])
      expect(result2).toEqual(['conn1', 'conn2'])
    })
  })

  describe('isFavorite', () => {
    test('未收藏的连接返回false', async () => {
      expect(await isFavorite('conn1')).toBe(false)
    })

    test('已收藏的连接返回true', async () => {
      await toggleFavorite('conn1')
      expect(await isFavorite('conn1')).toBe(true)
    })

    test('取消收藏后返回false', async () => {
      await toggleFavorite('conn1')
      await toggleFavorite('conn1')
      expect(await isFavorite('conn1')).toBe(false)
    })

    test('检查多个收藏状态', async () => {
      await toggleFavorite('conn1')
      await toggleFavorite('conn2')
      await toggleFavorite('conn3')

      expect(await isFavorite('conn1')).toBe(true)
      expect(await isFavorite('conn2')).toBe(true)
      expect(await isFavorite('conn3')).toBe(true)
      expect(await isFavorite('conn4')).toBe(false)
    })
  })

  describe('sortWithFavorites', () => {
    test('空数组返回空数组', async () => {
      const result = await sortWithFavorites([])
      expect(result).toEqual([])
    })

    test('没有收藏时保持原顺序', async () => {
      const connections = [
        { id: 'conn1', name: 'Connection 1' },
        { id: 'conn2', name: 'Connection 2' },
        { id: 'conn3', name: 'Connection 3' },
      ]

      const result = await sortWithFavorites(connections)
      expect(result).toEqual(connections)
    })

    test('收藏的连接排在前面', async () => {
      const connections = [
        { id: 'conn1', name: 'Connection 1' },
        { id: 'conn2', name: 'Connection 2' },
        { id: 'conn3', name: 'Connection 3' },
      ]

      await toggleFavorite('conn2')
      await toggleFavorite('conn1')

      const result = await sortWithFavorites(connections)
      expect(result[0].id).toBe('conn1')
      expect(result[1].id).toBe('conn2')
      expect(result[2].id).toBe('conn3')
    })

    test('保持收藏之间的原始顺序', async () => {
      const connections = [
        { id: 'conn1', name: 'Connection 1' },
        { id: 'conn2', name: 'Connection 2' },
        { id: 'conn3', name: 'Connection 3' },
      ]

      await toggleFavorite('conn1')
      await toggleFavorite('conn3')

      const result = await sortWithFavorites(connections)
      expect(result[0].id).toBe('conn1')
      expect(result[1].id).toBe('conn3')
      expect(result[2].id).toBe('conn2')
    })

    test('不修改原始数组', async () => {
      const connections = [
        { id: 'conn1', name: 'Connection 1' },
        { id: 'conn2', name: 'Connection 2' },
        { id: 'conn3', name: 'Connection 3' },
      ]

      const originalOrder = connections.map(c => c.id)
      await toggleFavorite('conn2')
      await sortWithFavorites(connections)

      const currentOrder = connections.map(c => c.id)
      expect(originalOrder).toEqual(currentOrder)
    })

    test('返回新数组', async () => {
      const connections = [
        { id: 'conn1', name: 'Connection 1' },
        { id: 'conn2', name: 'Connection 2' },
      ]

      await toggleFavorite('conn1')
      const result = await sortWithFavorites(connections)

      expect(result).not.toBe(connections)
      expect(result).toEqual(connections)
    })

    test('所有连接都被收藏时保持原始顺序', async () => {
      const connections = [
        { id: 'conn1', name: 'Connection 1' },
        { id: 'conn2', name: 'Connection 2' },
        { id: 'conn3', name: 'Connection 3' },
      ]

      await toggleFavorite('conn1')
      await toggleFavorite('conn2')
      await toggleFavorite('conn3')

      const result = await sortWithFavorites(connections)
      expect(result).toEqual(connections)
    })

    test('复杂场景测试', async () => {
      const connections = [
        { id: 'conn1', name: 'Connection 1' },
        { id: 'conn2', name: 'Connection 2' },
        { id: 'conn3', name: 'Connection 3' },
        { id: 'conn4', name: 'Connection 4' },
        { id: 'conn5', name: 'Connection 5' },
      ]

      await toggleFavorite('conn1')
      await toggleFavorite('conn3')
      await toggleFavorite('conn5')

      const result = await sortWithFavorites(connections)
      expect(result[0].id).toBe('conn1')
      expect(result[1].id).toBe('conn3')
      expect(result[2].id).toBe('conn5')
      expect(result[3].id).toBe('conn2')
      expect(result[4].id).toBe('conn4')
    })
  })

  describe('集成测试', () => {
    test('完整的收藏工作流', async () => {
      const connections = [
        { id: 'conn1', name: 'Server 1' },
        { id: 'conn2', name: 'Server 2' },
        { id: 'conn3', name: 'Server 3' },
      ]

      expect(await getFavorites()).toEqual([])
      expect(await isFavorite('conn1')).toBe(false)

      await toggleFavorite('conn1')
      expect(await getFavorites()).toContain('conn1')
      expect(await isFavorite('conn1')).toBe(true)

      await toggleFavorite('conn3')
      expect(await getFavorites()).toHaveLength(2)

      const sorted = await sortWithFavorites(connections)
      expect(sorted[0].id).toBe('conn1')
      expect(sorted[1].id).toBe('conn3')
      expect(sorted[2].id).toBe('conn2')

      await toggleFavorite('conn1')
      expect(await getFavorites()).not.toContain('conn1')
      expect(await isFavorite('conn1')).toBe(false)
      expect(await getFavorites()).toHaveLength(1)

      const sorted2 = await sortWithFavorites(connections)
      expect(sorted2[0].id).toBe('conn3')
    })
  })
})
