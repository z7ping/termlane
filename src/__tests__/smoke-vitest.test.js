import { describe, expect, test } from 'vitest'

describe('Termlane browser IPC smoke', () => {
  test('uses browser mock outside Tauri', async () => {
    const tauri = await import('../utils/tauri.ts')
    expect(tauri.isTauri).toBe(false)
    expect(typeof tauri.invoke).toBe('function')
    expect(typeof tauri.listen).toBe('function')
  })

  test('loads the local connection fixture', async () => {
    const { invoke } = await import('../utils/tauri.ts')
    const connections = await invoke('load_connections')
    expect(connections).toEqual(expect.arrayContaining([
      expect.objectContaining({ id: 'local', host: 'localhost', authType: 'local' }),
    ]))
  })

  test('keeps browser mode explicit about real SSH limitations', async () => {
    const { invoke } = await import('../utils/tauri.ts')
    await expect(invoke('ssh_connect', {
      host: 'example.com',
      username: 'user',
      password: 'demo',
    })).rejects.toThrow('浏览器演示模式不支持真实 SSH 连接')
  })

  test('supports active SFTP UI mocks', async () => {
    const { invoke } = await import('../utils/tauri.ts')
    const localFiles = await invoke('sftp_list_local', { path: '/' })
    const remoteFiles = await invoke('sftp_list_remote', { path: '/' })
    expect(Array.isArray(localFiles)).toBe(true)
    expect(Array.isArray(remoteFiles)).toBe(true)
  })

  test('recording mock matches the native command name', async () => {
    const { invoke } = await import('../utils/tauri.ts')
    await expect(invoke('save_recording', {
      filename: 'demo.cast',
      content: '',
      meta: {},
    })).resolves.toBeNull()
  })

  test('removed monitoring surface is not mocked as a product capability', async () => {
    const { invoke } = await import('../utils/tauri.ts')
    await expect(invoke('ssh_monitor', { sessionId: 'demo' })).rejects.toThrow('Unknown command')
  })

  test('reports current build metadata', async () => {
    const { invoke } = await import('../utils/tauri.ts')
    await expect(invoke('get_app_version')).resolves.toBe('0.1.0')
  })
})
