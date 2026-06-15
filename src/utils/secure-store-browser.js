/**
 * XTerminal Pro 浏览器版本安全存储模块
 * 纯浏览器环境，使用AES-GCM加密 + localStorage
 * 不依赖任何Tauri相关API
 */

// 加密密钥管理
const ENCRYPTION_KEY_NAME = 'xt_secure_key'

/**
 * 生成或获取加密密钥
 * 使用 Web Crypto API 生成随机密钥
 *
 * SECURITY NOTE: The AES key is stored as JWK in sessionStorage, which is
 * accessible to any same-origin script. This is the browser-only fallback —
 * in the Tauri desktop app, the Rust backend uses the OS keyring via the
 * tauri-plugin-stronghold secure-store backend, so the key never touches
 * the browser storage layer. For the browser build, sessionStorage is
 * acceptable since there is no cross-origin data to exfiltrate, but an XSS
 * vulnerability could extract the key. This is an inherent trade-off of
 * pure browser storage; if stronger guarantees are needed, use the desktop
 * build or integrate a server-side secret manager.
 */
async function getEncryptionKey() {
  try {
    // 尝试从sessionStorage获取已存在的密钥
    const storedKey = sessionStorage.getItem(ENCRYPTION_KEY_NAME)
    if (storedKey) {
      const keyData = JSON.parse(storedKey)
      return await crypto.subtle.importKey(
        'jwk',
        keyData,
        { name: 'AES-GCM' },
        true,
        ['encrypt', 'decrypt']
      )
    }

    // 生成新密钥
    const key = await crypto.subtle.generateKey(
      {
        name: 'AES-GCM',
        length: 256,
      },
      true,
      ['encrypt', 'decrypt']
    )

    // 导出并存储密钥到sessionStorage（会话结束时清除）
    const exportedKey = await crypto.subtle.exportKey('jwk', key)
    sessionStorage.setItem(ENCRYPTION_KEY_NAME, JSON.stringify(exportedKey))

    return key
  } catch (error) {
    console.error('Failed to get encryption key:', error)
    throw new Error('Encryption key generation failed')
  }
}

/**
 * AES-GCM 加密
 */
async function encryptData(data) {
  try {
    const key = await getEncryptionKey()
    const encoder = new TextEncoder()
    const encoded = encoder.encode(data)

    // 生成随机IV
    const iv = crypto.getRandomValues(new Uint8Array(12))

    const ciphertext = await crypto.subtle.encrypt(
      {
        name: 'AES-GCM',
        iv: iv,
      },
      key,
      encoded
    )

    return {
      ciphertext: btoa(String.fromCharCode(...new Uint8Array(ciphertext))),
      iv: btoa(String.fromCharCode(...iv)),
    }
  } catch (error) {
    console.error('Encryption failed:', error)
    throw new Error('Data encryption failed')
  }
}

/**
 * AES-GCM 解密
 */
async function decryptData(ciphertext, iv) {
  try {
    const key = await getEncryptionKey()
    const encryptedData = Uint8Array.from(atob(ciphertext), c => c.charCodeAt(0))
    const ivData = Uint8Array.from(atob(iv), c => c.charCodeAt(0))

    const decrypted = await crypto.subtle.decrypt(
      {
        name: 'AES-GCM',
        iv: ivData,
      },
      key,
      encryptedData
    )

    const decoder = new TextDecoder()
    return decoder.decode(decrypted)
  } catch (error) {
    console.error('Decryption failed:', error)
    return null
  }
}

/**
 * 浏览器环境安全存储实现
 * 使用AES-GCM加密 + localStorage
 */
class BrowserSecureStorage {
  constructor() {
    this.PREFIX = 'xt_secure_'
  }

  async set(key, value) {
    try {
      const json = JSON.stringify(value)
      const encrypted = await encryptData(json)
      localStorage.setItem(
        this.PREFIX + key,
        JSON.stringify(encrypted)
      )
      return true
    } catch (error) {
      console.error(`Failed to store ${key}:`, error)
      return false
    }
  }

  async get(key) {
    try {
      const raw = localStorage.getItem(this.PREFIX + key)
      if (!raw) return null

      const encrypted = JSON.parse(raw)
      const decrypted = await decryptData(encrypted.ciphertext, encrypted.iv)
      if (!decrypted) return null

      return JSON.parse(decrypted)
    } catch (error) {
      console.error(`Failed to retrieve ${key}:`, error)
      return null
    }
  }

  async remove(key) {
    localStorage.removeItem(this.PREFIX + key)
  }

  async clear() {
    // 清除所有以 PREFIX 开头的键
    const keys = []
    for (let i = 0; i < localStorage.length; i++) {
      const key = localStorage.key(i)
      if (key?.startsWith(this.PREFIX)) {
        keys.push(key)
      }
    }
    keys.forEach(key => localStorage.removeItem(key))
  }

  /**
   * 列出所有存储的键（不包含前缀）
   */
  async keys() {
    const result = []
    for (let i = 0; i < localStorage.length; i++) {
      const key = localStorage.key(i)
      if (key?.startsWith(this.PREFIX)) {
        result.push(key.substring(this.PREFIX.length))
      }
    }
    return result
  }

  /**
   * 检查键是否存在
   */
  async has(key) {
    return localStorage.getItem(this.PREFIX + key) !== null
  }
}

/**
 * 创建安全存储实例
 */
function createSecureStorage() {
  return new BrowserSecureStorage()
}

// 默认实例
export const secureStore = createSecureStorage()

// ============ 辅助函数 ============

/**
 * 密码存储辅助函数
 */
export async function storePassword(connectionId, password) {
  return secureStore.set(`pwd_${connectionId}`, password)
}

export async function getPassword(connectionId) {
  return secureStore.get(`pwd_${connectionId}`)
}

export async function removePassword(connectionId) {
  return secureStore.remove(`pwd_${connectionId}`)
}

/**
 * 书签存储辅助函数
 */
export async function storeBookmarks(bookmarks) {
  return secureStore.set('bookmarks', bookmarks)
}

export async function getBookmarks() {
  return secureStore.get('bookmarks')
}

/**
 * 收藏夹存储辅助函数
 */
export async function storeFavorites(favorites) {
  return secureStore.set('favorites', favorites)
}

export async function getFavorites() {
  return secureStore.get('favorites')
}

/**
 * 定时任务存储辅助函数
 */
export async function storeSchedulerTasks(tasks) {
  return secureStore.set('scheduler_tasks', tasks)
}

export async function getSchedulerTasks() {
  return secureStore.get('scheduler_tasks')
}

/**
 * 浏览器兼容性检查
 */
export function checkBrowserSupport() {
  const features = {
    crypto: 'crypto' in window && 'subtle' in window.crypto,
    localStorage: typeof Storage !== 'undefined',
    sessionStorage: typeof Storage !== 'undefined',
    btoa: typeof btoa === 'function',
    atob: typeof atob === 'function',
    json: typeof JSON === 'object',
  }

  const supported = Object.values(features).every(Boolean)
  
  if (!supported) {
    console.error('Browser support check failed:', features)
    return false
  }

  return true
}

/**
 * 导出存储统计信息
 */
export async function getStorageInfo() {
  const store = createSecureStorage()
  const keys = await store.keys()
  
  let totalSize = 0
  const details = {}
  
  for (const key of keys) {
    const raw = localStorage.getItem(store.PREFIX + key)
    const size = raw ? new Blob([raw]).size : 0
    totalSize += size
    details[key] = size
  }
  
  return {
    keyCount: keys.length,
    totalSize,
    details,
    availableSpace: localStorage.length, // 粗略估计
  }
}