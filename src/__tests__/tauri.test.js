// src/__tests__/tauri.test.js - Tauri 工具函数测试
import { invoke, listen, isTauri } from '../utils/tauri.js'

describe('tauri.js 工具函数', () => {
  // RED - 先写失败的测试
  test('invoke 函数存在且可调用', () => {
    // 测试 invoke 函数存在
    expect(typeof invoke).toBe('function')
  })

  test('listen 函数存在且可调用', () => {
    // 测试 listen 函数存在
    expect(typeof listen).toBe('function')
  })

  test('isTauri 返回布尔值', () => {
    // 测试 isTauri 返回布尔值
    expect(typeof isTauri).toBe('boolean')
    // 在 Node.js 环境中，isTauri 应该为 false
    expect(isTauri).toBe(false)
  })

  test('invoke 返回 Promise', async () => {
    // 测试 invoke 返回 Promise
    const result = invoke('test_command')
    expect(result).toBeInstanceOf(Promise)
    
    // 测试 mock 调用
    try {
      await result
    } catch (error) {
      // 在 Node.js 环境中，invoke 应该抛出错误
      expect(error).toBeDefined()
    }
  })
})