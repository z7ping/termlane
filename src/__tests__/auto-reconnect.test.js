// src/__tests__/auto-reconnect.test.js - 自动重连功能测试
import { AutoReconnectManager, createReconnectManager } from '../utils/auto-reconnect.ts'

describe('AutoReconnectManager 自动重连', () => {
  test('AutoReconnectManager 类可导入', () => {
    // 测试 AutoReconnectManager 类可导入
    expect(typeof AutoReconnectManager).toBe('function')
  })

  test('createReconnectManager 函数存在', () => {
    // 测试 createReconnectManager 函数存在
    expect(typeof createReconnectManager).toBe('function')
  })

  test('AutoReconnectManager 可实例化', () => {
    // 测试 AutoReconnectManager 可实例化
    const manager = new AutoReconnectManager()
    expect(manager instanceof AutoReconnectManager).toBe(true)
  })

  test('AutoReconnectManager 实例有正确的方法', () => {
    // 测试 AutoReconnectManager 实例有正确的方法
    const manager = new AutoReconnectManager()
    
    expect(typeof manager.register).toBe('function')
    expect(typeof manager.unregister).toBe('function')
    expect(typeof manager.onConnect).toBe('function')
    expect(typeof manager.onDisconnect).toBe('function')
  })

  test('createReconnectManager 工厂函数返回正确实例', () => {
    // 测试 createReconnectManager 工厂函数返回正确实例
    const manager = createReconnectManager(5, 1000)
    expect(manager instanceof AutoReconnectManager).toBe(true)
    expect(manager.maxRetries).toBe(5)
    expect(manager.retryDelay).toBe(1000)
  })

  test('默认参数正确', () => {
    // 测试默认参数正确
    const manager = new AutoReconnectManager()
    expect(manager.maxRetries).toBe(3)
    expect(manager.retryDelay).toBe(5000)
  })
})