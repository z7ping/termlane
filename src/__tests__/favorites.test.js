// src/__tests__/favorites.test.js - 连接收藏/置顶功能测试
import { getFavorites, toggleFavorite, isFavorite, sortWithFavorites } from '../utils/favorites.js'

describe('连接收藏/置顶功能', () => {
  beforeEach(() => {
    // 每个测试前清理测试数据
    localStorage.clear()
  })

  describe('getFavorites', () => {
    test('初始化时返回空数组', () => {
      const favs = getFavorites()
      expect(favs).toEqual([])
    })

    test('返回已存储的收藏列表', () => {
      const testFavs = ['conn1', 'conn2', 'conn3']
      localStorage.setItem('xterminal_favorites', JSON.stringify(testFavs))
      
      const favs = getFavorites()
      expect(favs).toEqual(testFavs)
    })

    test('JSON解析错误时返回空数组', () => {
      localStorage.setItem('xterminal_favorites', 'invalid json')
      const favs = getFavorites()
      expect(favs).toEqual([])
    })

    test('没有存储时返回空数组', () => {
      const favs = getFavorites()
      expect(favs).toEqual([])
    })
  })

  describe('toggleFavorite', () => {
    test('添加新收藏', () => {
      const result = toggleFavorite('conn1')
      expect(result).toContain('conn1')
      expect(result).toHaveLength(1)
    })

    test('移除已有收藏', () => {
      toggleFavorite('conn1')
      const result = toggleFavorite('conn1')
      expect(result).not.toContain('conn1')
      expect(result).toHaveLength(0)
    })

    test('多次切换', () => {
      expect(toggleFavorite('conn1')).toContain('conn1')
      expect(toggleFavorite('conn1')).not.toContain('conn1')
      expect(toggleFavorite('conn1')).toContain('conn1')
    })

    test('支持多个收藏', () => {
      toggleFavorite('conn1')
      toggleFavorite('conn2')
      toggleFavorite('conn3')
      
      const favs = getFavorites()
      expect(favs).toEqual(['conn1', 'conn2', 'conn3'])
    })

    test('移除其中一个收藏不影响其他', () => {
      toggleFavorite('conn1')
      toggleFavorite('conn2')
      toggleFavorite('conn3')
      
      toggleFavorite('conn2')
      
      const favs = getFavorites()
      expect(favs).toEqual(['conn1', 'conn3'])
      expect(favs).toHaveLength(2)
    })

    test('重复添加同一个收藏只保留一个', () => {
      toggleFavorite('conn1')
      toggleFavorite('conn1')
      toggleFavorite('conn1')
      
      const favs = getFavorites()
      expect(favs).toEqual(['conn1'])
      expect(favs).toHaveLength(1)
    })

    test('返回更新后的收藏列表', () => {
      const result1 = toggleFavorite('conn1')
      const result2 = toggleFavorite('conn2')
      
      expect(result1).toEqual(['conn1'])
      expect(result2).toEqual(['conn1', 'conn2'])
    })
  })

  describe('isFavorite', () => {
    test('未收藏的连接返回false', () => {
      expect(isFavorite('conn1')).toBe(false)
    })

    test('已收藏的连接返回true', () => {
      toggleFavorite('conn1')
      expect(isFavorite('conn1')).toBe(true)
    })

    test('取消收藏后返回false', () => {
      toggleFavorite('conn1')
      toggleFavorite('conn1')
      expect(isFavorite('conn1')).toBe(false)
    })

    test('检查多个收藏状态', () => {
      toggleFavorite('conn1')
      toggleFavorite('conn2')
      toggleFavorite('conn3')
      
      expect(isFavorite('conn1')).toBe(true)
      expect(isFavorite('conn2')).toBe(true)
      expect(isFavorite('conn3')).toBe(true)
      expect(isFavorite('conn4')).toBe(false)
    })
  })

  describe('sortWithFavorites', () => {
    test('空数组返回空数组', () => {
      const result = sortWithFavorites([])
      expect(result).toEqual([])
    })

    test('没有收藏时保持原顺序', () => {
      const connections = [
        { id: 'conn1', name: 'Connection 1' },
        { id: 'conn2', name: 'Connection 2' },
        { id: 'conn3', name: 'Connection 3' },
      ]
      
      const result = sortWithFavorites(connections)
      expect(result).toEqual(connections)
    })

    test('收藏的连接排在前面', () => {
      const connections = [
        { id: 'conn1', name: 'Connection 1' },
        { id: 'conn2', name: 'Connection 2' },
        { id: 'conn3', name: 'Connection 3' },
      ]
      
      toggleFavorite('conn2')
      toggleFavorite('conn1')
      
      const result = sortWithFavorites(connections)
      // 收藏的连接在前，但保持原始顺序
      expect(result[0].id).toBe('conn1')
      expect(result[1].id).toBe('conn2')
      expect(result[2].id).toBe('conn3')
    })

    test('保持收藏之间的原始顺序', () => {
      const connections = [
        { id: 'conn1', name: 'Connection 1' },
        { id: 'conn2', name: 'Connection 2' },
        { id: 'conn3', name: 'Connection 3' },
      ]
      
      toggleFavorite('conn1')
      toggleFavorite('conn3')
      
      const result = sortWithFavorites(connections)
      expect(result[0].id).toBe('conn1')
      expect(result[1].id).toBe('conn3')
      expect(result[2].id).toBe('conn2')
    })

    test('不修改原始数组', () => {
      const connections = [
        { id: 'conn1', name: 'Connection 1' },
        { id: 'conn2', name: 'Connection 2' },
        { id: 'conn3', name: 'Connection 3' },
      ]
      
      const originalOrder = connections.map(c => c.id)
      toggleFavorite('conn2')
      sortWithFavorites(connections)
      
      const currentOrder = connections.map(c => c.id)
      expect(originalOrder).toEqual(currentOrder)
    })

    test('返回新数组', () => {
      const connections = [
        { id: 'conn1', name: 'Connection 1' },
        { id: 'conn2', name: 'Connection 2' },
      ]
      
      toggleFavorite('conn1')
      const result = sortWithFavorites(connections)
      
      expect(result).not.toBe(connections)
      expect(result).toEqual(connections)
    })

    test('所有连接都被收藏时保持原始顺序', () => {
      const connections = [
        { id: 'conn1', name: 'Connection 1' },
        { id: 'conn2', name: 'Connection 2' },
        { id: 'conn3', name: 'Connection 3' },
      ]
      
      toggleFavorite('conn1')
      toggleFavorite('conn2')
      toggleFavorite('conn3')
      
      const result = sortWithFavorites(connections)
      expect(result).toEqual(connections)
    })

    test('复杂场景测试', () => {
      const connections = [
        { id: 'conn1', name: 'Connection 1' },
        { id: 'conn2', name: 'Connection 2' },
        { id: 'conn3', name: 'Connection 3' },
        { id: 'conn4', name: 'Connection 4' },
        { id: 'conn5', name: 'Connection 5' },
      ]
      
      // 收藏 conn1, conn3, conn5
      toggleFavorite('conn1')
      toggleFavorite('conn3')
      toggleFavorite('conn5')
      
      const result = sortWithFavorites(connections)
      // 收藏的连接按原始顺序在前，非收藏的在后
      expect(result[0].id).toBe('conn1')
      expect(result[1].id).toBe('conn3')
      expect(result[2].id).toBe('conn5')
      expect(result[3].id).toBe('conn2')
      expect(result[4].id).toBe('conn4')
    })
  })

  describe('集成测试', () => {
    test('完整的收藏工作流', () => {
      const connections = [
        { id: 'conn1', name: 'Server 1' },
        { id: 'conn2', name: 'Server 2' },
        { id: 'conn3', name: 'Server 3' },
      ]
      
      // 初始状态
      expect(getFavorites()).toEqual([])
      expect(isFavorite('conn1')).toBe(false)
      
      // 添加收藏
      toggleFavorite('conn1')
      expect(getFavorites()).toContain('conn1')
      expect(isFavorite('conn1')).toBe(true)
      
      // 添加更多收藏
      toggleFavorite('conn3')
      expect(getFavorites()).toHaveLength(2)
      
      // 排序测试
      const sorted = sortWithFavorites(connections)
      expect(sorted[0].id).toBe('conn1')
      expect(sorted[1].id).toBe('conn3')
      expect(sorted[2].id).toBe('conn2')
      
      // 取消收藏
      toggleFavorite('conn1')
      expect(getFavorites()).not.toContain('conn1')
      expect(isFavorite('conn1')).toBe(false)
      expect(getFavorites()).toHaveLength(1)
      
      // 再次排序
      const sorted2 = sortWithFavorites(connections)
      expect(sorted2[0].id).toBe('conn3')
    })
  })
})
