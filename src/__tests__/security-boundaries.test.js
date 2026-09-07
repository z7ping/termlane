import { describe, expect, test } from 'vitest'
import { stripConnectionSecrets } from '../utils/credentials.ts'
import { hostKeyTarget, parseHostKeyError } from '../utils/ssh-host-key.ts'

describe('credential persistence boundary', () => {
  test('ordinary connection persistence removes password and passphrase', () => {
    const safe = stripConnectionSecrets({
      id: 'prod',
      host: '10.0.0.1',
      username: 'root',
      password: 'secret-password',
      passphrase: 'secret-passphrase',
    })

    expect(safe).toEqual({
      id: 'prod',
      host: '10.0.0.1',
      username: 'root',
    })
    expect('password' in safe).toBe(false)
    expect('passphrase' in safe).toBe(false)
  })
})

describe('SSH host key error boundary', () => {
  test('parses first-use host key details', () => {
    const detail = parseHostKeyError(
      'HOST_KEY_UNKNOWN:{"host":"example.com","port":2222,"algorithm":"ssh-ed25519","fingerprint":"SHA256:01:02"}',
    )

    expect(detail).toEqual({
      code: 'HOST_KEY_UNKNOWN',
      host: 'example.com',
      port: 2222,
      algorithm: 'ssh-ed25519',
      fingerprint: 'SHA256:01:02',
    })
    expect(hostKeyTarget(detail)).toBe('example.com:2222')
  })

  test('parses mismatch as a hard-failure code', () => {
    const detail = parseHostKeyError(
      new Error('HOST_KEY_MISMATCH:{"host":"example.com","port":22,"algorithm":"ssh-rsa","fingerprint":"SHA256:aa:bb"}'),
    )

    expect(detail?.code).toBe('HOST_KEY_MISMATCH')
    expect(hostKeyTarget(detail)).toBe('example.com')
  })

  test('ignores malformed or unrelated errors', () => {
    expect(parseHostKeyError('connection timeout')).toBeNull()
    expect(parseHostKeyError('HOST_KEY_UNKNOWN:not-json')).toBeNull()
  })
})
