// utils/tauri.js - Tauri IPC wrapper
// 当在 Tauri 环境中运行时使用原生 IPC，否则使用模拟数据

const isTauri = typeof window.__TAURI__ !== 'undefined';

export const invoke = isTauri
  ? window.__TAURI__.core.invoke
  : async (cmd, args = {}) => {
      // 浏览器环境下的模拟实现
      console.log(`[Mock] invoke: ${cmd}`, args);

      switch (cmd) {
        case 'ssh_connect':
          return `mock_session_${Date.now()}`;
        case 'ssh_connect_key':
          return `mock_session_key_${Date.now()}`;
        case 'ssh_connect_jump':
          return `mock_session_jump_${Date.now()}`;
        case 'ssh_execute':
          return mockExecute(args.command);
        case 'ssh_disconnect':
          return null;
        case 'ssh_list_sessions':
          return [];
        case 'load_connections':
          return [
            { id: 'local', name: '本地终端', host: 'localhost', port: 22, username: 'user', authType: 'local', group: '本地', icon: '💻' },
          ];
        case 'save_connection':
          return null;
        case 'delete_connection':
          return null;
        case 'get_app_version':
          return '0.1.0';
        case 'sftp_list_local':
          return mockLocalFiles(args.path);
        case 'sftp_list_remote':
          return mockRemoteFiles(args.path);
        case 'sftp_upload':
          return `上传完成: ${args.local_path} -> ${args.remote_path}`;
        case 'sftp_download':
          return `下载完成: ${args.remote_path} -> ${args.local_path}`;
        case 'sftp_rename':
          return `已重命名: ${args.old_path} → ${args.new_path}`;
        case 'sftp_delete':
          return `已删除: ${args.path}`;
        case 'sftp_mkdir':
          return `已创建: ${args.path}`;
        case 'sftp_chmod':
          return `已修改权限: ${args.path} → ${args.mode}`;
        case 'sftp_read_file':
          return `// 文件内容预览\n// 路径: ${args.path}\n// 此功能需要连接远程服务器`;
        case 'sftp_write_file':
          return `已保存: ${args.path}`;
        case 'ssh_start_shell':
          return `mock_shell_${Date.now()}`;
        case 'ssh_shell_input':
          return null;
        case 'ssh_shell_resize':
          return null;
        case 'ssh_close_shell':
          return null;
        case 'ssh_list_shells':
          return [];
        case 'ssh_monitor':
          return {
            cpu_usage: 25.3,
            memory_total: 16777216000,
            memory_used: 6442450944,
            memory_percent: 38.4,
            disk_total: 107374182400,
            disk_used: 45097156608,
            disk_percent: 42.0,
            load_1: 0.45,
            load_5: 0.38,
            load_15: 0.32,
            uptime_seconds: 278400,
          };
        default:
          throw new Error(`Unknown command: ${cmd}`);
      }
    };

function mockExecute(command) {
  const parts = command.trim().split(/\s+/);
  const bin = parts[0];

  const responses = {
    help: '可用命令: help, clear, echo, date, whoami, pwd, ls, uname, df, free, top, exit',
    clear: '\x1b[2J\x1b[H',
    date: new Date().toString(),
    whoami: 'user',
    pwd: '/home/user',
    hostname: 'xterminal-pro',
    uname: 'Linux xterminal-pro 6.6.0-generic x86_64 GNU/Linux',
    'uname -a': 'Linux xterminal-pro 6.6.0-generic #1 SMP x86_64 GNU/Linux',
    ls: 'Desktop  Documents  Downloads  Music  Pictures  Videos',
    'ls -la': `total 28
drwxr-xr-x 7 user user 4096 Mar 30 16:00 .
drwxr-xr-x 3 root root 4096 Mar 30 16:00 ..
drwxr-xr-x 2 user user 4096 Mar 30 16:00 Desktop
drwxr-xr-x 2 user user 4096 Mar 30 16:00 Documents
drwxr-xr-x 2 user user 4096 Mar 30 16:00 Downloads`,
    'df -h': `Filesystem      Size  Used Avail Use% Mounted on
/dev/sda1       100G   42G   58G  42% /
tmpfs           7.8G     0  7.8G   0% /dev/shm`,
    'free -h': `              total        used        free      shared  buff/cache   available
Mem:           15Gi       4.2Gi       8.1Gi       256Mi       3.1Gi        10Gi
Swap:          2.0Gi          0B       2.0Gi`,
    uptime: ` 16:32:00 up 3 days,  4:20,  1 user,  load average: 0.15, 0.20, 0.18`,
    id: `uid=1000(user) gid=1000(user) groups=1000(user),4(adm),27(sudo)`,
    cat: parts[1] ? `cat: ${parts[1]}: No such file or directory` : 'cat: missing operand',
    echo: parts.slice(1).join(' '),
  };

  if (command === 'clear') return '\x1b[2J\x1b[H';
  if (bin === 'echo') return parts.slice(1).join(' ');
  if (responses[command]) return responses[command];
  if (responses[bin]) return responses[bin];

  return `\x1b[31m${bin}: command not found\x1b[0m\n提示: 这是浏览器演示模式，连接服务器后可执行完整命令`;
}

function mockLocalFiles(path) {
  return [
    { name: '..', path: '/home/user', size: 0, is_dir: true, modified: '2026-03-30 10:00', permissions: 'drwxr-xr-x' },
    { name: 'Documents', path: '/home/user/Documents', size: 4096, is_dir: true, modified: '2026-03-30 14:00', permissions: 'drwxr-xr-x' },
    { name: 'Downloads', path: '/home/user/Downloads', size: 4096, is_dir: true, modified: '2026-03-30 12:00', permissions: 'drwxr-xr-x' },
    { name: 'Desktop', path: '/home/user/Desktop', size: 4096, is_dir: true, modified: '2026-03-30 08:00', permissions: 'drwxr-xr-x' },
    { name: 'Pictures', path: '/home/user/Pictures', size: 4096, is_dir: true, modified: '2026-03-29 18:00', permissions: 'drwxr-xr-x' },
    { name: '.bashrc', path: '/home/user/.bashrc', size: 3526, is_dir: false, modified: '2026-03-25 10:00', permissions: '-rw-r--r--' },
    { name: '.gitconfig', path: '/home/user/.gitconfig', size: 280, is_dir: false, modified: '2026-03-28 16:00', permissions: '-rw-r--r--' },
    { name: 'README.md', path: '/home/user/README.md', size: 1520, is_dir: false, modified: '2026-03-30 15:30', permissions: '-rw-r--r--' },
    { name: 'project.tar.gz', path: '/home/user/project.tar.gz', size: 15728640, is_dir: false, modified: '2026-03-29 22:00', permissions: '-rw-r--r--' },
  ];
}

function mockRemoteFiles(path) {
  return [
    { name: '..', path: '/', size: 0, is_dir: true, modified: '2026-03-01 00:00', permissions: 'drwxr-xr-x' },
    { name: 'etc', path: '/etc', size: 4096, is_dir: true, modified: '2026-03-30 08:00', permissions: 'drwxr-xr-x' },
    { name: 'home', path: '/home', size: 4096, is_dir: true, modified: '2026-01-15 10:00', permissions: 'drwxr-xr-x' },
    { name: 'var', path: '/var', size: 4096, is_dir: true, modified: '2026-03-30 16:00', permissions: 'drwxr-xr-x' },
    { name: 'tmp', path: '/tmp', size: 4096, is_dir: true, modified: '2026-03-30 17:00', permissions: 'drwxrwxrwt' },
    { name: 'opt', path: '/opt', size: 4096, is_dir: true, modified: '2026-02-20 14:00', permissions: 'drwxr-xr-x' },
    { name: 'usr', path: '/usr', size: 4096, is_dir: true, modified: '2026-03-01 00:00', permissions: 'drwxr-xr-x' },
  ];
}

export const listen = isTauri
  ? window.__TAURI__.event.listen
  : async (event, callback) => {
      console.log(`[Mock] listen: ${event}`);
      // Return a no-op unlisten function
      return () => {};
    };

export default { invoke, listen, isTauri };
