// src/__tests__/secure-storage.test.js - 安全存储测试
import { secureStore, storePassword, getPassword, removePassword } from '../utils/secure-store-browser.js'

describe('安全存储', () => {
  beforeEach(async () => {
    // 每个测试前清理测试数据
    await secureStore.clear()
  })

  describe('基础存储功能', () => {
    test('secureStore 存储数据成功', async () => {
      const result = await secureStore.set('test_key', { data: 'test_value' })
      expect(result).toBe(true)
    })

    test('secureStore 读取存储的数据', async () => {
      const testData = { data: 'test_value', number: 123 }
      await secureStore.set('test_key', testData)
      const retrieved = await secureStore.get('test_key')
      expect(retrieved).toEqual(testData)
    })

    test('secureStore 返回null当键不存在', async () => {
      const retrieved = await secureStore.get('non_existent_key')
      expect(retrieved).toBeNull()
    })

    test('secureStore 删除存储的数据', async () => {
      await secureStore.set('test_key', { data: 'test' })
      await secureStore.remove('test_key')
      const retrieved = await secureStore.get('test_key')
      expect(retrieved).toBeNull()
    })

    test('支持存储字符串', async () => {
      await secureStore.set('str_key', 'hello world')
      const retrieved = await secureStore.get('str_key')
      expect(retrieved).toBe('hello world')
    })

    test('支持存储数字', async () => {
      await secureStore.set('num_key', 42)
      const retrieved = await secureStore.get('num_key')
      expect(retrieved).toBe(42)
    })

    test('支持存储布尔值', async () => {
      await secureStore.set('bool_key', true)
      const retrieved = await secureStore.get('bool_key')
      expect(retrieved).toBe(true)
    })

    test('支持存储复杂嵌套对象', async () => {
      const complexObj = {
        user: {
          name: 'test',
          permissions: ['read', 'write'],
          metadata: {
            created: Date.now()
          }
        }
      }
      await secureStore.set('complex_key', complexObj)
      const retrieved = await secureStore.get('complex_key')
      expect(retrieved).toEqual(complexObj)
    })

    test('支持存储数组', async () => {
      const arr = [1, 2, 3, 'four', { five: 5 }]
      await secureStore.set('array_key', arr)
      const retrieved = await secureStore.get('array_key')
      expect(retrieved).toEqual(arr)
    })
  })

  describe('密码存储功能', () => {
    test('storePassword 存储密码成功', async () => {
      const result = await storePassword('conn_001', 'secret_password')
      expect(result).toBe(true)
    })

    test('getPassword 读取存储的密码', async () => {
      await storePassword('conn_001', 'secret_password')
      const retrieved = await getPassword('conn_001')
      expect(retrieved).toBe('secret_password')
    })

    test('getPassword 返回null当连接ID不存在', async () => {
      const retrieved = await getPassword('non_existent_conn')
      expect(retrieved).toBeNull()
    })

    test('removePassword 删除存储的密码', async () => {
      await storePassword('conn_001', 'secret_password')
      await removePassword('conn_001')
      const retrieved = await getPassword('conn_001')
      expect(retrieved).toBeNull()
    })

    test('可以存储多个不同连接的密码', async () => {
      await storePassword('conn_001', 'password1')
      await storePassword('conn_002', 'password2')
      await storePassword('conn_003', 'password3')

      expect(await getPassword('conn_001')).toBe('password1')
      expect(await getPassword('conn_002')).toBe('password2')
      expect(await getPassword('conn_003')).toBe('password3')
    })

    test('支持包含特殊字符的密码', async () => {
      const specialPassword = 'P@ssw0rd!#$%^&*()_+-=[]{}|;:,.<>?'
      await storePassword('conn_special', specialPassword)
      const retrieved = await getPassword('conn_special')
      expect(retrieved).toBe(specialPassword)
    })

    test('支持空字符串密码', async () => {
      await storePassword('conn_empty', '')
      const retrieved = await getPassword('conn_empty')
      expect(retrieved).toBe('')
    })
  })

  describe('错误处理', () => {
    test('secureStore 处理无法序列化的数据', async () => {
      // circular reference cannot be JSON serialized
      const circularObj = {}
      circularObj.self = circularObj

      const result = await secureStore.set('circular', circularObj)
      expect(result).toBe(false)
    })

    test('secureStore 处理损坏的数据', async () => {
      localStorage.setItem('xt_secure_corrupted', 'invalid_json')
      const retrieved = await secureStore.get('corrupted')
      expect(retrieved).toBeNull()
    })
  })

  describe('前缀隔离', () => {
    test('不同键的数据互不影响', async () => {
      await secureStore.set('key1', 'value1')
      await secureStore.set('key2', 'value2')

      expect(await secureStore.get('key1')).toBe('value1')
      expect(await secureStore.get('key2')).toBe('value2')
    })

    test('secureStorage键使用xt_secure_前缀', async () => {
      await secureStore.set('test_key', 'test_value')
      const allKeys = Object.keys(localStorage)
      expect(allKeys).toContain('xt_secure_test_key')
    })

    test('密码键使用pwd_前缀', async () => {
      await storePassword('conn_001', 'password')
      const allKeys = Object.keys(localStorage)
      expect(allKeys).toContain('xt_secure_pwd_conn_001')
    })
  })
})
