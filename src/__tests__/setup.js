import { vi } from 'vitest'

// 测试环境设置
// Mock localStorage 用于 jsdom 环境

if (typeof globalThis.localStorage === 'undefined' || !globalThis.localStorage) {
  const store = {}
  globalThis.localStorage = {
    getItem: (k) => store[k] ?? null,
    setItem: (k, v) => { store[k] = String(v); },
    removeItem: (k) => { delete store[k]; },
    clear: () => { Object.keys(store).forEach(k => delete store[k]); },
    get length() { return Object.keys(store).length; },
    key: (i) => Object.keys(store)[i],
  }
}

// Mock sessionStorage
if (typeof globalThis.sessionStorage === 'undefined' || !globalThis.sessionStorage) {
  const store = {}
  globalThis.sessionStorage = {
    getItem: (k) => store[k] ?? null,
    setItem: (k, v) => { store[k] = String(v); },
    removeItem: (k) => { delete store[k]; },
    clear: () => { Object.keys(store).forEach(k => delete store[k]); },
    get length() { return Object.keys(store).length; },
    key: (i) => Object.keys(store)[i],
  }
}

// Mock Web Crypto API
if (typeof globalThis.crypto === 'undefined') {
  globalThis.crypto = {
    subtle: {
      generateKey: async () => ({
        exportKey: async () => new ArrayBuffer(32)
      }),
      encrypt: async () => new ArrayBuffer(32),
      decrypt: async () => new ArrayBuffer(32),
      importKey: async () => ({}),
      deriveKey: async () => ({}),
    },
    getRandomValues: (arr) => {
      for (let i = 0; i < arr.length; i++) {
        arr[i] = Math.floor(Math.random() * 256)
      }
      return arr
    }
  }
}

// Mock @tauri-apps/plugin-secure-storage
vi.mock('@tauri-apps/plugin-secure-storage', () => ({
  getPassword: vi.fn().mockResolvedValue(null),
  setPassword: vi.fn().mockResolvedValue(true),
  deletePassword: vi.fn().mockResolvedValue(true),
}))

// Mock Tauri API
globalThis.__TAURI__ = {
  invoke: async () => ({}),
  listen: async () => ({}),
  core: {
    path: { resolve: async (p) => p },
  },
}
