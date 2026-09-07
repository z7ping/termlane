/**
 * Termlane 浏览器版本安全存储模块
 * 纯浏览器环境，使用 AES-GCM 加密 + localStorage。
 * 不依赖任何 Tauri 相关 API；桌面凭证不经过本模块。
 */

const ENCRYPTION_KEY_NAME = 'xt_secure_key'

/**
 * 生成或获取浏览器回退模式的加密密钥。
 *
 * SECURITY NOTE: AES key 以可导出的 JWK 放在 sessionStorage，任何同源脚本
 * 都可能在发生 XSS 时读取它。因此这里只能作为 browser mock/fallback，不能
 * 与桌面安全等级等同。Tauri 桌面模式通过 Rust `keyring` 使用 OS Keyring，
 * password / passphrase 不应进入此浏览器存储层。
 */
async function getEncryptionKey(): Promise<CryptoKey> {
  try {
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

    const key = await crypto.subtle.generateKey(
      {
        name: 'AES-GCM',
        length: 256,
      },
      true,
      ['encrypt', 'decrypt']
    )

    const exportedKey = await crypto.subtle.exportKey('jwk', key)
    sessionStorage.setItem(ENCRYPTION_KEY_NAME, JSON.stringify(exportedKey))

    return key
  } catch (error) {
    console.error('Failed to get encryption key:', error)
    throw new Error('Encryption key generation failed')
  }
}

interface EncryptedData {
  ciphertext: string
  iv: string
}

async function encryptData(data: string): Promise<EncryptedData> {
  try {
    const key = await getEncryptionKey()
    const encoder = new TextEncoder()
    const encoded = encoder.encode(data)
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

async function decryptData(ciphertext: string, iv: string): Promise<string | null> {
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

class BrowserSecureStorage {
  readonly PREFIX = 'xt_secure_'

  async set(key: string, value: unknown): Promise<boolean> {
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

  async get<T = unknown>(key: string): Promise<T | null> {
    try {
      const raw = localStorage.getItem(this.PREFIX + key)
      if (!raw) return null

      const encrypted: EncryptedData = JSON.parse(raw)
      const decrypted = await decryptData(encrypted.ciphertext, encrypted.iv)
      if (!decrypted) return null

      return JSON.parse(decrypted) as T
    } catch (error) {
      console.error(`Failed to retrieve ${key}:`, error)
      return null
    }
  }

  async remove(key: string): Promise<void> {
    localStorage.removeItem(this.PREFIX + key)
  }

  async clear(): Promise<void> {
    const keys: string[] = []
    for (let i = 0; i < localStorage.length; i++) {
      const key = localStorage.key(i)
      if (key?.startsWith(this.PREFIX)) {
        keys.push(key)
      }
    }
    keys.forEach(key => localStorage.removeItem(key))
  }

  async keys(): Promise<string[]> {
    const result: string[] = []
    for (let i = 0; i < localStorage.length; i++) {
      const key = localStorage.key(i)
      if (key?.startsWith(this.PREFIX)) {
        result.push(key.substring(this.PREFIX.length))
      }
    }
    return result
  }

  async has(key: string): Promise<boolean> {
    return localStorage.getItem(this.PREFIX + key) !== null
  }
}

function createSecureStorage(): BrowserSecureStorage {
  return new BrowserSecureStorage()
}

export const secureStore = createSecureStorage()

export async function storePassword(connectionId: string, password: string): Promise<boolean> {
  return secureStore.set(`pwd_${connectionId}`, password)
}

export async function getPassword(connectionId: string): Promise<string | null> {
  return secureStore.get<string>(`pwd_${connectionId}`)
}

export async function removePassword(connectionId: string): Promise<void> {
  return secureStore.remove(`pwd_${connectionId}`)
}

export async function storeBookmarks(bookmarks: unknown): Promise<boolean> {
  return secureStore.set('bookmarks', bookmarks)
}

export async function getBookmarks<T = unknown>(): Promise<T | null> {
  return secureStore.get<T>('bookmarks')
}

export async function storeFavorites(favorites: unknown): Promise<boolean> {
  return secureStore.set('favorites', favorites)
}

export async function getFavorites<T = unknown>(): Promise<T | null> {
  return secureStore.get<T>('favorites')
}

export async function storeSchedulerTasks(tasks: unknown): Promise<boolean> {
  return secureStore.set('scheduler_tasks', tasks)
}

export async function getSchedulerTasks<T = unknown>(): Promise<T | null> {
  return secureStore.get<T>('scheduler_tasks')
}

export function checkBrowserSupport(): boolean {
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

interface StorageInfo {
  keyCount: number
  totalSize: number
  details: Record<string, number>
  availableSpace: number
}

export async function getStorageInfo(): Promise<StorageInfo> {
  const store = createSecureStorage()
  const keys = await store.keys()

  let totalSize = 0
  const details: Record<string, number> = {}

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
    availableSpace: localStorage.length,
  }
}
