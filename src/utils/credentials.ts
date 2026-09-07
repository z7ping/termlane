import { invoke, isTauri } from './tauri'
import { getPassword, removePassword, storePassword } from './secure-store-browser'

const LEGACY_PASSWORD_PREFIX = 'xterminal-pwd_'

export type ConnectionLike = Record<string, unknown> & { id?: string }

/**
 * Remove transient secrets before a connection object is persisted in ordinary storage.
 */
export function stripConnectionSecrets<T extends ConnectionLike>(connection: T): Omit<T, 'password' | 'passphrase'> {
  const { password: _password, passphrase: _passphrase, ...safe } = connection
  return safe as Omit<T, 'password' | 'passphrase'>
}

/**
 * Desktop credentials live only in the OS keyring. Browser demo credentials use
 * the existing AES-GCM fallback because no native keyring exists there.
 */
export async function saveCredential(connectionId: string, password: string): Promise<void> {
  if (!connectionId || !password) return

  if (isTauri) {
    await invoke('keyring_save_password', { connId: connectionId, password })
    return
  }

  const stored = await storePassword(connectionId, password)
  if (!stored) throw new Error('浏览器安全存储写入失败')
}

export async function loadCredential(connectionId: string): Promise<string> {
  if (!connectionId) return ''

  if (isTauri) {
    try {
      const value = await invoke('keyring_load_password', { connId: connectionId })
      return typeof value === 'string' ? value : ''
    } catch {
      return ''
    }
  }

  return (await getPassword(connectionId)) || ''
}

export async function deleteCredential(connectionId: string): Promise<void> {
  if (!connectionId) return

  if (isTauri) {
    try {
      await invoke('keyring_delete_password', { connId: connectionId })
    } catch {
      // delete_connection also removes the keyring item; missing credentials are harmless
    }
    return
  }

  await removePassword(connectionId)
}

/**
 * Migrate the historical plaintext localStorage password into the current secure
 * backend and always remove the plaintext key once a safe copy already exists or
 * the migration succeeds.
 */
export async function migrateLegacyPlaintextCredential(connectionId: string): Promise<boolean> {
  if (!connectionId || typeof localStorage === 'undefined') return false

  const legacyKey = `${LEGACY_PASSWORD_PREFIX}${connectionId}`
  const legacyPassword = localStorage.getItem(legacyKey)
  if (!legacyPassword) return false

  const existing = await loadCredential(connectionId)
  if (!existing) {
    await saveCredential(connectionId, legacyPassword)
  }

  localStorage.removeItem(legacyKey)
  return true
}
