// src/__tests__/connection-tags.test.js - 连接标签功能测试
import { getTags, addTag, removeTag, getTagColor } from '../utils/connection-tags.ts'

describe('连接标签功能', () => {
  beforeEach(() => {
    // 每个测试前清理测试数据
    localStorage.clear()
  })

  describe('getTags', () => {
    test('初始化时返回默认标签', () => {
      const tags = getTags()
      expect(tags).toEqual([
        { id: 'prod', name: '生产', color: 'red' },
        { id: 'test', name: '测试', color: 'yellow' },
        { id: 'dev', name: '开发', color: 'green' },
        { id: 'db', name: '数据库', color: 'blue' }
      ])
    })

    test('返回已存储的标签列表', () => {
      const customTags = [
        { id: 'custom1', name: '自定义标签1', color: 'red' },
        { id: 'custom2', name: '自定义标签2', color: 'blue' },
      ]
      localStorage.setItem('xterminal_connection_tags', JSON.stringify(customTags))
      
      const tags = getTags()
      expect(tags).toEqual(customTags)
    })

    test('JSON解析错误时返回空数组', () => {
      localStorage.setItem('xterminal_connection_tags', 'invalid json')
      const tags = getTags()
      expect(tags).toEqual([])
    })

    test('没有存储时返回默认标签', () => {
      const tags = getTags()
      expect(tags).toHaveLength(4)
      expect(tags[0].id).toBe('prod')
      expect(tags[1].id).toBe('test')
    })
  })

  describe('addTag', () => {
    test('添加新标签', () => {
      const tags = addTag('新标签', 'purple')
      
      expect(tags).toHaveLength(5)
      const newTag = tags.find(t => t.name === '新标签')
      expect(newTag).toBeTruthy()
      expect(newTag.color).toBe('purple')
      expect(newTag.id).toMatch(/^tag_\d+_[a-z0-9]+$/)
    })

    test('添加多个标签', () => {
      addTag('标签1', 'orange')
      addTag('标签2', 'pink')
      
      const tags = getTags()
      expect(tags).toHaveLength(6)
      
      const tag1 = tags.find(t => t.name === '标签1')
      const tag2 = tags.find(t => t.name === '标签2')
      
      expect(tag1.color).toBe('orange')
      expect(tag2.color).toBe('pink')
    })

    test('返回更新后的标签列表', () => {
      const initialLength = getTags().length
      
      const result = addTag('测试标签', 'brown')
      
      expect(result).toHaveLength(initialLength + 1)
      expect(result).toEqual(getTags())
    })

    test('生成的ID是唯一的', async () => {
      // 添加小延迟确保不同时间戳
      const tag1 = addTag('标签1', 'color1')
      await new Promise(resolve => setTimeout(resolve, 1))
      const tag2 = addTag('标签2', 'color2')
      
      const newTag1 = tag1.find(t => t.name === '标签1')
      const newTag2 = tag2.find(t => t.name === '标签2')
      
      expect(newTag1.id).not.toBe(newTag2.id)
    })
  })

  describe('removeTag', () => {
    test('删除指定ID的标签', () => {
      const initialTags = getTags()
      const tagToRemove = initialTags[0]
      
      const result = removeTag(tagToRemove.id)
      
      expect(result).toHaveLength(3)
      expect(result).not.toContain(tagToRemove)
      
      const finalTags = getTags()
      expect(finalTags).toHaveLength(3)
    })

    test('删除不存在的标签不影响列表', () => {
      const initialLength = getTags().length
      
      const result = removeTag('non_existent_id')
      
      expect(result).toHaveLength(initialLength)
      expect(result).toEqual(getTags())
    })

    test('删除自定义标签', () => {
      // 先添加一个自定义标签
      addTag('自定义', 'custom')
      const tagsWithCustom = getTags()
      const customTag = tagsWithCustom.find(t => t.name === '自定义')
      
      // 删除自定义标签
      const result = removeTag(customTag.id)
      
      expect(result).toHaveLength(4) // 回到默认的4个标签
      expect(result).not.toContain(customTag)
    })

    test('返回更新后的标签列表', () => {
      const initialTags = getTags()
      const idToRemove = initialTags[0].id
      
      const result = removeTag(idToRemove)
      
      expect(result).toHaveLength(3)
      expect(result).toEqual(getTags())
    })
  })

  describe('getTagColor', () => {
    test('返回指定ID标签的颜色', () => {
      expect(getTagColor('prod')).toBe('red')
      expect(getTagColor('test')).toBe('yellow')
      expect(getTagColor('dev')).toBe('green')
      expect(getTagColor('db')).toBe('blue')
    })

    test('不存在的标签ID返回none', () => {
      expect(getTagColor('non_existent')).toBe('none')
    })

    test('自定义标签的颜色', () => {
      addTag('自定义标签', 'magenta')
      const tags = getTags()
      const customTag = tags.find(t => t.name === '自定义标签')
      
      expect(getTagColor(customTag.id)).toBe('magenta')
    })

    test('空ID返回none', () => {
      expect(getTagColor('')).toBe('none')
      expect(getTagColor(null)).toBe('none')
      expect(getTagColor(undefined)).toBe('none')
    })
  })

  describe('集成测试', () => {
    test('完整的标签管理流程', () => {
      // 初始状态
      let tags = getTags()
      expect(tags).toHaveLength(4)
      
      // 添加自定义标签
      tags = addTag('项目A', '#ff6b6b')
      expect(tags).toHaveLength(5)
      
      // 获取颜色
      expect(getTagColor('prod')).toBe('red')
      expect(getTagColor('test')).toBe('yellow')
      
      // 验证自定义标签
      const customTag = tags.find(t => t.name === '项目A')
      expect(customTag).toBeTruthy()
      expect(getTagColor(customTag.id)).toBe('#ff6b6b')
      
      // 删除标签
      tags = removeTag(customTag.id)
      expect(tags).toHaveLength(4)
      expect(getTagColor(customTag.id)).toBe('none')
    })

    test('批量标签操作', () => {
      // 批量添加
      addTag('开发环境', '#10b981')
      addTag('预发布环境', '#f59e0b')
      addTag('生产环境', '#ef4444')
      
      let tags = getTags()
      expect(tags).toHaveLength(7) // 4个默认 + 3个新增
      
      // 验证所有新增标签都存在
      const envDev = tags.find(t => t.name === '开发环境')
      const envStaging = tags.find(t => t.name === '预发布环境')
      const envProd = tags.find(t => t.name === '生产环境')
      
      expect(envDev).toBeTruthy()
      expect(envStaging).toBeTruthy()
      expect(envProd).toBeTruthy()
      
      // 验证颜色，使用find定位的标签对象
      expect(envDev.color).toBe('#10b981')
      expect(envStaging.color).toBe('#f59e0b')
      expect(envProd.color).toBe('#ef4444')
      
      // 验证getTagColor也能正确返回
      expect(getTagColor(envDev.id)).toBe('#10b981')
      expect(getTagColor(envStaging.id)).toBe('#f59e0b')
      expect(getTagColor(envProd.id)).toBe('#ef4444')
      
      // 批量删除
      removeTag(envDev.id)
      removeTag(envStaging.id)
      removeTag(envProd.id)
      
      tags = getTags()
      expect(tags).toHaveLength(4) // 只剩默认标签
      
      // 验证颜色查询返回none
      expect(getTagColor(envDev.id)).toBe('none')
      expect(getTagColor(envStaging.id)).toBe('none')
      expect(getTagColor(envProd.id)).toBe('none')
    })
  })
})
