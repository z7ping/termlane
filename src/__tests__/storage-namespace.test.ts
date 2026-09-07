import { beforeEach, describe, expect, test } from 'vitest'
import { STORAGE_KEYS, migrateLegacyStorageNamespace } from '../utils/storage-keys'

describe('Termlane storage namespace migration', () => {
  beforeEach(() => {
    localStorage.clear()
  })

  test('migrates legacy application keys to Termlane', () => {
    localStorage.setItem('xterminal-theme', 'nord')
    localStorage.setItem('xterminal_tabs', '[{"id":"tab-1"}]')

    migrateLegacyStorageNamespace()

    expect(localStorage.getItem(STORAGE_KEYS.THEME)).toBe('nord')
    expect(localStorage.getItem(STORAGE_KEYS.TABS)).toBe('[{"id":"tab-1"}]')
    expect(localStorage.getItem('xterminal-theme')).toBeNull()
    expect(localStorage.getItem('xterminal_tabs')).toBeNull()
  })

  test('does not overwrite an existing Termlane value', () => {
    localStorage.setItem(STORAGE_KEYS.THEME, 'dark')
    localStorage.setItem('xterminal-theme', 'light')

    migrateLegacyStorageNamespace()

    expect(localStorage.getItem(STORAGE_KEYS.THEME)).toBe('dark')
    expect(localStorage.getItem('xterminal-theme')).toBeNull()
  })

  test('migrates connection-scoped note keys', () => {
    localStorage.setItem('xterminal_notes_server-1', 'legacy note')

    migrateLegacyStorageNamespace()

    expect(localStorage.getItem(`${STORAGE_KEYS.NOTES_PREFIX}server-1`)).toBe('legacy note')
    expect(localStorage.getItem('xterminal_notes_server-1')).toBeNull()
  })

  test('leaves legacy plaintext credential keys for secure credential migration', () => {
    localStorage.setItem('xterminal-pwd_server-1', 'secret')

    migrateLegacyStorageNamespace()

    expect(localStorage.getItem('xterminal-pwd_server-1')).toBe('secret')
  })
})
