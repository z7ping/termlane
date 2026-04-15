// src/__tests__/secure-storage.test.js - 安全存储测试
import { secureStore, secureRetrieve, secureRemove, storePassword, getPassword, removePassword } from '../utils/secure-storage.js'

describe('安全存储', () => {
  beforeEach(() => {
    // 每个测试前清理测试数据
    localStorage.clear()
  })

  describe('基础存储功能', () => {
    test('secureStore 存储数据成功', () => {
      const result = secureStore('test_key', { data: 'test_value' })
      expect(result).toBe(true)
    })

    test('secureRetrieve 读取存储的数据', () => {
      const testData = { data: 'test_value', number: 123 }
      secureStore('test_key', testData)
      const retrieved = secureRetrieve('test_key')
      expect(retrieved).toEqual(testData)
    })

    test('secureRetrieve 返回null当键不存在', () => {
      const retrieved = secureRetrieve('non_existent_key')
      expect(retrieved).toBeNull()
    })

    test('secureRemove 删除存储的数据', () => {
      secureStore('test_key', { data: 'test' })
      secureRemove('test_key')
      const retrieved = secureRetrieve('test_key')
      expect(retrieved).toBeNull()
    })

    test('支持存储字符串', () => {
      secureStore('str_key', 'hello world')
      const retrieved = secureRetrieve('str_key')
      expect(retrieved).toBe('hello world')
    })

    test('支持存储数字', () => {
      secureStore('num_key', 42)
      const retrieved = secureRetrieve('num_key')
      expect(retrieved).toBe(42)
    })

    test('支持存储布尔值', () => {
      secureStore('bool_key', true)
      const retrieved = secureRetrieve('bool_key')
      expect(retrieved).toBe(true)
    })

    test('支持存储复杂嵌套对象', () => {
      const complexObj = {
        user: {
          name: 'test',
          permissions: ['read', 'write'],
          metadata: {
            created: Date.now()
          }
        }
      }
      secureStore('complex_key', complexObj)
      const retrieved = secureRetrieve('complex_key')
      expect(retrieved).toEqual(complexObj)
    })

    test('支持存储数组', () => {
      const arr = [1, 2, 3, 'four', { five: 5 }]
      secureStore('array_key', arr)
      const retrieved = secureRetrieve('array_key')
      expect(retrieved).toEqual(arr)
    })
  })

  describe('密码存储功能', () => {
    test('storePassword 存储密码成功', () => {
      const result = storePassword('conn_001', 'secret_password')
      expect(result).toBe(true)
    })

    test('getPassword 读取存储的密码', () => {
      storePassword('conn_001', 'secret_password')
      const retrieved = getPassword('conn_001')
      expect(retrieved).toBe('secret_password')
    })

    test('getPassword 返回null当连接ID不存在', () => {
      const retrieved = getPassword('non_existent_conn')
      expect(retrieved).toBeNull()
    })

    test('removePassword 删除存储的密码', () => {
      storePassword('conn_001', 'secret_password')
      removePassword('conn_001')
      const retrieved = getPassword('conn_001')
      expect(retrieved).toBeNull()
    })

    test('可以存储多个不同连接的密码', () => {
      storePassword('conn_001', 'password1')
      storePassword('conn_002', 'password2')
      storePassword('conn_003', 'password3')
      
      expect(getPassword('conn_001')).toBe('password1')
      expect(getPassword('conn_002')).toBe('password2')
      expect(getPassword('conn_003')).toBe('password3')
    })

    test('支持包含特殊字符的密码', () => {
      const specialPassword = 'P@ssw0rd!#$%^&*()_+-=[]{}|;:,.<>?'
      storePassword('conn_special', specialPassword)
      const retrieved = getPassword('conn_special')
      expect(retrieved).toBe(specialPassword)
    })

    test('支持空字符串密码', () => {
      storePassword('conn_empty', '')
      const retrieved = getPassword('conn_empty')
      expect(retrieved).toBe('')
    })
  })

  describe('错误处理', () => {
    test('secureStore 处理无法序列化的数据', () => {
      // circular reference cannot be JSON serialized
      const circularObj = {}
      circularObj.self = circularObj
      
      const result = secureStore('circular', circularObj)
      expect(result).toBe(false)
    })

    test('secureRetrieve 处理损坏的数据', () => {
      localStorage.setItem('xt_corrupted', 'invalid_json')
      const retrieved = secureRetrieve('corrupted')
      expect(retrieved).toBeNull()
    })
  })

  describe('前缀隔离', () => {
    test('不同键的数据互不影响', () => {
      secureStore('key1', 'value1')
      secureStore('key2', 'value2')
      
      expect(secureRetrieve('key1')).toBe('value1')
      expect(secureRetrieve('key2')).toBe('value2')
    })

    test('secureStorage键使用xt_前缀', () => {
      secureStore('test_key', 'test_value')
      const allKeys = Object.keys(localStorage)
      expect(allKeys).toContain('xt_test_key')
    })

    test('密码键使用pwd_前缀', () => {
      storePassword('conn_001', 'password')
      const allKeys = Object.keys(localStorage)
      expect(allKeys).toContain('xt_pwd_conn_001')
    })
  })
})
