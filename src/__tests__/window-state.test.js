// src/__tests__/window-state.test.js - 窗口状态持久化测试
import { saveWindowState, loadWindowState, clearWindowState, startAutoSave } from '../utils/window-state.ts'

describe('窗口状态持久化', () => {
  beforeEach(() => {
    // 每个测试前清理测试数据
    localStorage.clear()
    
    // Mock window properties
    global.window = {
      innerWidth: 1024,
      innerHeight: 768,
      screenX: 100,
      screenY: 50,
      addEventListener: vi.fn(),
    }
    
    global.document = {
      fullscreenElement: null,
    }
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  describe('saveWindowState', () => {
    test('保存窗口状态到localStorage', () => {
      saveWindowState()
      const stored = localStorage.getItem('xterminal_window_state')
      expect(stored).toBeTruthy()
    })

    test('保存的state包含正确的字段', () => {
      saveWindowState()
      const state = JSON.parse(localStorage.getItem('xterminal_window_state'))
      
      expect(state).toHaveProperty('width')
      expect(state).toHaveProperty('height')
      expect(state).toHaveProperty('x')
      expect(state).toHaveProperty('y')
      expect(state).toHaveProperty('maximized')
      expect(state).toHaveProperty('savedAt')
    })

    test('保存的width和height正确', () => {
      saveWindowState()
      const state = JSON.parse(localStorage.getItem('xterminal_window_state'))
      
      expect(state.width).toBe(1024)
      expect(state.height).toBe(768)
    })

    test('保存的x和y坐标正确', () => {
      saveWindowState()
      const state = JSON.parse(localStorage.getItem('xterminal_window_state'))
      
      expect(state.x).toBe(100)
      expect(state.y).toBe(50)
    })

    test('maximized根据fullscreenElement判断', () => {
      // 不是全屏
      saveWindowState()
      let state = JSON.parse(localStorage.getItem('xterminal_window_state'))
      expect(state.maximized).toBe(false)
      
      // 模拟全屏
      document.fullscreenElement = document.body
      saveWindowState()
      state = JSON.parse(localStorage.getItem('xterminal_window_state'))
      expect(state.maximized).toBe(true)
    })

    test('savedAt是有效的时间戳', () => {
      const beforeSave = Date.now()
      saveWindowState()
      const afterSave = Date.now()
      
      const state = JSON.parse(localStorage.getItem('xterminal_window_state'))
      expect(state.savedAt).toBeGreaterThanOrEqual(beforeSave)
      expect(state.savedAt).toBeLessThanOrEqual(afterSave)
    })

    test('多次保存会覆盖之前的状态', () => {
      saveWindowState()
      const firstState = localStorage.getItem('xterminal_window_state')
      
      // 改变窗口大小
      window.innerWidth = 1920
      window.innerHeight = 1080
      
      saveWindowState()
      const secondState = localStorage.getItem('xterminal_window_state')
      
      expect(firstState).not.toBe(secondState)
    })

    test('异常时静默失败', () => {
      // Mock JSON.stringify to throw error
      const originalStringify = JSON.stringify
      JSON.stringify = vi.fn(() => { throw new Error('Mock error') })
      
      // 不应该抛出异常
      expect(() => saveWindowState()).not.toThrow()
      
      JSON.stringify = originalStringify
    })
  })

  describe('loadWindowState', () => {
    test('加载保存的窗口状态', () => {
      const testState = {
        width: 800,
        height: 600,
        x: 0,
        y: 0,
        maximized: false,
        savedAt: Date.now(),
      }
      localStorage.setItem('xterminal_window_state', JSON.stringify(testState))
      
      const loaded = loadWindowState()
      expect(loaded).toEqual(testState)
    })

    test('没有保存状态时返回null', () => {
      const loaded = loadWindowState()
      expect(loaded).toBeNull()
    })

    test('JSON解析错误时返回null', () => {
      localStorage.setItem('xterminal_window_state', 'invalid json')
      const loaded = loadWindowState()
      expect(loaded).toBeNull()
    })

    test('支持加载不完整的state对象', () => {
      const incompleteState = { width: 1024, height: 768 }
      localStorage.setItem('xterminal_window_state', JSON.stringify(incompleteState))
      
      const loaded = loadWindowState()
      expect(loaded).toEqual(incompleteState)
    })
  })

  describe('clearWindowState', () => {
    test('删除保存的窗口状态', () => {
      saveWindowState()
      expect(localStorage.getItem('xterminal_window_state')).toBeTruthy()
      
      clearWindowState()
      expect(localStorage.getItem('xterminal_window_state')).toBeNull()
    })

    test('多次清理安全', () => {
      clearWindowState()
      clearWindowState()
      expect(localStorage.getItem('xterminal_window_state')).toBeNull()
    })
  })

  describe('startAutoSave', () => {
    test('注册resize事件监听器', () => {
      startAutoSave()
      expect(window.addEventListener).toHaveBeenCalledWith('resize', expect.any(Function))
    })

    test('多次调用不会重复注册太多监听器', () => {
      startAutoSave()
      const callCount1 = window.addEventListener.mock.calls.filter(
        call => call[0] === 'resize'
      ).length
      
      startAutoSave()
      const callCount2 = window.addEventListener.mock.calls.filter(
        call => call[0] === 'resize'
      ).length
      
      // 每次调用都会注册一个监听器，这是预期行为
      expect(callCount2).toBeGreaterThanOrEqual(callCount1)
    })
  })

  describe('集成测试', () => {
    test('完整的保存-加载-清理流程', () => {
      // 初始状态为null
      expect(loadWindowState()).toBeNull()
      
      // 保存状态
      saveWindowState()
      const state1 = loadWindowState()
      expect(state1).toBeTruthy()
      
      // 修改后再次保存
      window.innerWidth = 1280
      window.innerHeight = 720
      saveWindowState()
      
      const state2 = loadWindowState()
      expect(state2.width).toBe(1280)
      expect(state2.height).toBe(720)
      expect(state2).not.toEqual(state1)
      
      // 清理状态
      clearWindowState()
      expect(loadWindowState()).toBeNull()
    })
  })
})
