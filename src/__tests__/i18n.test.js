// src/__tests__/i18n.test.js - 国际化功能测试
const i18n = require('../utils/i18n.ts')

// Mock localStorage 用于 Node.js 环境
if (typeof globalThis.localStorage === 'undefined') {
  const store = {}
  globalThis.localStorage = {
    getItem: (k) => store[k] ?? null,
    setItem: (k, v) => { store[k] = String(v) },
    removeItem: (k) => { delete store[k] },
  }
}

describe('i18n 国际化功能', () => {
  // RED - 先写失败的测试
  test('t() 函数存在且可调用', () => {
    // 测试 t() 函数存在
    expect(typeof i18n.t).toBe('function')
  })

  test('setLocale() 函数存在且可调用', () => {
    // 测试 setLocale() 函数存在
    expect(typeof i18n.setLocale).toBe('function')
  })

  test('getLocale() 函数存在且可调用', () => {
    // 测试 getLocale() 函数存在
    expect(typeof i18n.getLocale).toBe('function')
  })

  test('getLocales() 函数存在且返回数组', () => {
    // 测试 getLocales() 函数存在且返回数组
    expect(typeof i18n.getLocales).toBe('function')
    const locales = i18n.getLocales()
    expect(Array.isArray(locales)).toBe(true)
    expect(locales.length).toBeGreaterThan(0)
  })

  test('支持中文 (zh)', () => {
    // 测试支持中文
    const locales = i18n.getLocales()
    expect(locales.includes('zh')).toBe(true)
  })

  test('支持英文 (en)', () => {
    // 测试支持英文
    const locales = i18n.getLocales()
    expect(locales.includes('en')).toBe(true)
  })

  test('t("app.name") 返回正确值', () => {
    // 测试 t("app.name") 返回正确值
    const appName = i18n.t('app.name')
    expect(appName).toBe('Termlane')
  })

  test('设置语言后可以切换', () => {
    // 测试设置语言后可以切换
    const originalLocale = i18n.getLocale()

    // 切换到英文
    i18n.setLocale('en')
    expect(i18n.getLocale()).toBe('en')

    // 切换到中文
    i18n.setLocale('zh')
    expect(i18n.getLocale()).toBe('zh')

    // 恢复原始语言
    i18n.setLocale(originalLocale)
  })
})