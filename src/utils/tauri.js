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

export default { invoke, isTauri };
