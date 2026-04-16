// utils/tauri.js - Tauri IPC wrapper
// Tauri 环境使用原生 IPC，浏览器使用 mock

let _invoke, _listen, _isTauri;

try {
  // Tauri v2: withGlobalTauri 注入 window.__TAURI__
  if (typeof window !== 'undefined' && window.__TAURI__ && window.__TAURI__.core) {
    _invoke = window.__TAURI__.core.invoke;
    _listen = window.__TAURI__.event.listen;
    _isTauri = true;
  } else {
    throw new Error('Not in Tauri');
  }
} catch {
  _isTauri = false;
  _invoke = null;
  _listen = null;
}

export const isTauri = _isTauri;

export const invoke = _isTauri
  ? _invoke
  : async (cmd, args = {}) => {
      // 浏览器 mock
      switch (cmd) {
        case 'ssh_connect': {
          if (!args.password) throw new Error('认证失败: 密码不能为空');
          // mock 无法真正认证，统一返回失败提示
          throw new Error('认证失败: 用户名或密码错误（浏览器演示模式不支持真实连接）');
        }
        case 'ssh_connect_key': return `mock_session_key_${Date.now()}`;
        case 'ssh_connect_jump': return `mock_session_jump_${Date.now()}`;
        case 'ssh_execute': return mockExecute(args.command);
        case 'ssh_disconnect': return null;
        case 'ssh_list_sessions': return [];
        case 'ssh_start_shell': return `mock_shell_${Date.now()}`;
        case 'ssh_shell_input': case 'ssh_shell_resize': case 'ssh_close_shell': return null;
        case 'ssh_list_shells': return [];
        case 'ssh_monitor': return { cpu_usage: 25.3, memory_total: 16777216000, memory_used: 6442450944, memory_percent: 38.4, disk_total: 107374182400, disk_used: 45097156608, disk_percent: 42.0, load_1: 0.45, load_5: 0.38, load_15: 0.32, uptime_seconds: 278400 };
        case 'tcp_ping': return Math.floor(Math.random() * 80) + 5;
        case 'load_connections': return [{ id: 'local', name: '本地终端', host: 'localhost', port: 22, username: 'local', authType: 'local', group: '本地', icon: '💻' }];
        case 'save_connection': case 'delete_connection': return null;
        case 'get_app_version': return '0.1.0';
        case 'sftp_list_local': return mockLocalFiles(args.path);
        case 'sftp_list_remote': return mockRemoteFiles(args.path);
        case 'sftp_upload': return `上传: ${args.local} -> ${args.remote}`;
        case 'sftp_download': return `下载: ${args.remote} -> ${args.local}`;
        case 'sftp_rename': return `已重命名`;
        case 'sftp_delete': return `已删除`;
        case 'sftp_mkdir': return `已创建`;
        case 'sftp_chmod': return `已修改权限`;
        case 'sftp_read_file': return '// 预览\n// 需要连接远程服务器';
        case 'sftp_write_file': return '已保存';
        case 'keyring_save_password': case 'keyring_load_password': case 'keyring_delete_password': return '***';
        case 'save_window_state': case 'load_window_state': return { x: 100, y: 100, width: 1200, height: 800, maximized: false };
        case 'list_recordings': return [];
        case 'save_recording_meta': case 'delete_recording': case 'save_recording_file': case 'read_recording_file': case 'get_recording_dir': return '';
        case 'local_start_shell': return `mock_local_${Date.now()}`;
        case 'local_shell_input': case 'local_shell_resize': case 'local_close_shell': return null;
        case 'local_list_shells': return [];
        default: throw new Error(`Unknown command: ${cmd}`);
      }
    };

export const listen = _isTauri
  ? _listen
  : async (event, callback) => {
      // Mock listen - no-op
      return () => {};
    };

function mockExecute(command) {
  const bin = command.trim().split(/\s+/)[0];
  const responses = {
    help: '可用命令: help, clear, echo, date, whoami, pwd, ls, uname, df, free, exit',
    clear: '\x1b[2J\x1b[H',
    date: new Date().toString(),
    whoami: 'user',
    pwd: '/home/user',
    hostname: 'xterminal-pro',
    uname: 'Linux xterminal-pro 6.6.0-generic x86_64 GNU/Linux',
    ls: 'Desktop  Documents  Downloads',
    'ls -la': 'total 28\ndrwxr-xr-x 7 user user 4096 Mar 30 16:00 .\ndrwxr-xr-x 3 root root 4096 Mar 30 16:00 ..',
    'df -h': 'Filesystem Size Used Avail Use% Mounted on\n/dev/sda1 100G 42G 58G 42% /',
    'free -h': 'total used free shared buff/cache available\nMem: 15Gi 4.2Gi 8.1Gi 256Mi 3.1Gi 10Gi',
    uptime: '16:32:00 up 3 days, 4:20, 1 user, load average: 0.15, 0.20, 0.18',
    echo: command.slice(5),
  };
  if (command === 'clear') return '\x1b[2J\x1b[H';
  if (bin === 'echo') return command.slice(5);
  return responses[command] || responses[bin] || `\x1b[31m${bin}: command not found\x1b[0m\n提示: 浏览器演示模式`;
}

function mockLocalFiles(path) {
  return [
    { name: '..', path: '/home/user', size: 0, is_dir: true, modified: '2026-03-30', permissions: 'drwxr-xr-x' },
    { name: 'Documents', path: '/home/user/Documents', size: 4096, is_dir: true, modified: '2026-03-30', permissions: 'drwxr-xr-x' },
    { name: '.bashrc', path: '/home/user/.bashrc', size: 3526, is_dir: false, modified: '2026-03-25', permissions: '-rw-r--r--' },
  ];
}

function mockRemoteFiles(path) {
  return [
    { name: '..', path: '/', size: 0, is_dir: true, modified: '2026-03-01', permissions: 'drwxr-xr-x' },
    { name: 'etc', path: '/etc', size: 4096, is_dir: true, modified: '2026-03-30', permissions: 'drwxr-xr-x' },
    { name: 'home', path: '/home', size: 4096, is_dir: true, modified: '2026-01-15', permissions: 'drwxr-xr-x' },
  ];
}

export default { invoke, listen, isTauri };
