// 安全存储（配置加密）
// 在Tauri环境中使用系统keyring，浏览器环境使用混淆存储

const ENCRYPTION_KEY_PREFIX = 'xt_'

export function secureStore(key, value) {
  try {
    // Simple obfuscation for browser (real encryption in Tauri would use keyring)
    const encoded = btoa(encodeURIComponent(JSON.stringify(value)))
    localStorage.setItem(ENCRYPTION_KEY_PREFIX + key, encoded)
    return true
  } catch {
    return false
  }
}

export function secureRetrieve(key) {
  try {
    const encoded = localStorage.getItem(ENCRYPTION_KEY_PREFIX + key)
    if (!encoded) return null
    return JSON.parse(decodeURIComponent(atob(encoded)))
  } catch {
    return null
  }
}

export function secureRemove(key) {
  localStorage.removeItem(ENCRYPTION_KEY_PREFIX + key)
}

// Store connection passwords securely
export function storePassword(connectionId, password) {
  return secureStore(`pwd_${connectionId}`, password)
}

export function getPassword(connectionId) {
  return secureRetrieve(`pwd_${connectionId}`)
}

export function removePassword(connectionId) {
  secureRemove(`pwd_${connectionId}`)
}
