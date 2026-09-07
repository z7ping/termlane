// Tauri IPC wrapper: native desktop calls use official ESM APIs; browser mode uses a focused UI mock.

import { invoke as nativeInvoke, isTauri as detectTauri } from '@tauri-apps/api/core'
import { listen as nativeListen } from '@tauri-apps/api/event'

type InvokeFn = (cmd: string, args?: Record<string, unknown>) => Promise<unknown>
type ListenFn = (event: string, handler: (event: { payload: unknown }) => void) => Promise<() => void>

export const isTauri: boolean = typeof window !== 'undefined' && detectTauri()

export const invoke: InvokeFn = isTauri
  ? nativeInvoke
  : async (cmd: string, args: Record<string, unknown> = {}) => {
      switch (cmd) {
        case 'ssh_connect': {
          if (!args.password) throw new Error('认证失败: 密码不能为空')
          throw new Error('浏览器演示模式不支持真实 SSH 连接')
        }
        case 'ssh_connect_key': throw new Error('浏览器演示模式不支持真实 SSH 连接')
        case 'ssh_execute': return mockExecute(args.command as string)
        case 'ssh_disconnect': return null
        case 'ssh_list_sessions': return []
        case 'ssh_start_shell': return `mock_shell_${Date.now()}`
        case 'ssh_shell_input':
        case 'ssh_shell_resize':
        case 'ssh_close_shell': return null
        case 'ssh_list_shells': return []
        case 'tcp_ping': return Math.floor(Math.random() * 80) + 5
        case 'load_connections': return [
          { id: 'local', name: '本地终端', host: 'localhost', port: 22, username: 'local', authType: 'local', group: '本地' },
        ]
        case 'save_connection':
        case 'delete_connection': return null
        case 'get_app_version': return '0.1.0'
        case 'sftp_list_local': return mockLocalFiles(args.path as string)
        case 'sftp_list_remote': return mockRemoteFiles(args.path as string)
        case 'sftp_upload': return `上传: ${args.local} -> ${args.remote}`
        case 'sftp_download': return `下载: ${args.remote} -> ${args.local}`
        case 'sftp_rename': return '已重命名'
        case 'sftp_delete': return '已删除'
        case 'sftp_mkdir': return '已创建'
        case 'sftp_chmod': return '已修改权限'
        case 'sftp_read_file': return '// 浏览器演示模式：需要桌面应用连接真实服务器'
        case 'sftp_write_file': return '已保存'
        case 'keyring_save_password':
        case 'keyring_delete_password': return null
        case 'keyring_load_password': return ''
        case 'list_recordings': return []
        case 'save_recording_meta':
        case 'delete_recording':
        case 'save_recording': return null
        case 'read_recording_file': return ''
        case 'get_recording_dir': return ''
        case 'local_start_shell': return `mock_local_${Date.now()}`
        case 'local_input':
        case 'local_resize':
        case 'local_close_shell': return null
        case 'local_list_shells': return []
        case 'check_update': return null
        default: throw new Error(`Unknown command: ${cmd}`)
      }
    }

export const listen: ListenFn = isTauri
  ? (nativeListen as unknown as ListenFn)
  : async (_event: string, _callback: (event: { payload: unknown }) => void) => () => {}

function mockExecute(command = ''): string {
  const trimmed = command.trim()
  const bin = trimmed.split(/\s+/)[0]
  const responses: Record<string, string> = {
    help: '可用命令: help, clear, echo, date, whoami, pwd, ls, uname, df, free, exit',
    clear: '\x1b[2J\x1b[H',
    date: new Date().toString(),
    whoami: 'user',
    pwd: '/home/user',
    hostname: 'termlane',
    uname: 'Linux termlane 6.6.0-generic x86_64 GNU/Linux',
    ls: 'Desktop  Documents  Downloads',
    'ls -la': 'total 28\ndrwxr-xr-x 7 user user 4096 .\ndrwxr-xr-x 3 root root 4096 ..',
    'df -h': 'Filesystem Size Used Avail Use% Mounted on\n/dev/sda1 100G 42G 58G 42% /',
    'free -h': 'total used free shared buff/cache available\nMem: 15Gi 4.2Gi 8.1Gi 256Mi 3.1Gi 10Gi',
    uptime: 'up 3 days, load average: 0.15, 0.20, 0.18',
  }
  if (bin === 'echo') return trimmed.slice(5)
  return responses[trimmed] || responses[bin] || `${bin}: command not found\n提示: 浏览器演示模式`
}

function mockLocalFiles(_path: string): FileItem[] {
  return [
    { name: '..', path: '/home/user', size: 0, isDir: true, modified: '2026-03-30', permissions: 'drwxr-xr-x' },
    { name: 'Documents', path: '/home/user/Documents', size: 4096, isDir: true, modified: '2026-03-30', permissions: 'drwxr-xr-x' },
    { name: '.bashrc', path: '/home/user/.bashrc', size: 3526, isDir: false, modified: '2026-03-25', permissions: '-rw-r--r--' },
  ]
}

function mockRemoteFiles(_path: string): FileItem[] {
  return [
    { name: '..', path: '/', size: 0, isDir: true, modified: '2026-03-01', permissions: 'drwxr-xr-x' },
    { name: 'etc', path: '/etc', size: 4096, isDir: true, modified: '2026-03-30', permissions: 'drwxr-xr-x' },
    { name: 'home', path: '/home', size: 4096, isDir: true, modified: '2026-01-15', permissions: 'drwxr-xr-x' },
  ]
}

interface FileItem {
  name: string
  path: string
  size: number
  isDir: boolean
  modified: string
  permissions: string
}

export default { invoke, listen, isTauri }
