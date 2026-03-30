// command-completion.js - 命令自动补全

const commonCommands = [
  // 文件操作
  'ls', 'ls -la', 'ls -lh', 'cd', 'pwd', 'mkdir', 'rmdir', 'rm', 'rm -rf', 'cp', 'mv', 'touch', 'cat', 'less', 'more', 'head', 'tail', 'tail -f', 'find', 'locate', 'which', 'whereis',
  // 文本处理
  'grep', 'grep -r', 'sed', 'awk', 'sort', 'uniq', 'wc', 'diff', 'cut', 'tr', 'tee',
  // 系统信息
  'uname -a', 'hostname', 'whoami', 'id', 'uptime', 'date', 'cal', 'df', 'df -h', 'du', 'du -sh', 'free', 'free -h', 'top', 'htop', 'ps', 'ps aux', 'ps aux --sort=-%mem', 'kill', 'kill -9',
  // 网络
  'ping', 'ping -c 4', 'curl', 'wget', 'ssh', 'scp', 'rsync', 'netstat', 'ss', 'ss -tlnp', 'ip addr', 'ifconfig', 'traceroute', 'nslookup', 'dig',
  // 压缩
  'tar', 'tar -czf', 'tar -xzf', 'zip', 'unzip', 'gzip', 'gunzip',
  // 权限
  'chmod', 'chown', 'chgrp', 'sudo', 'su',
  // 包管理
  'apt update', 'apt upgrade', 'apt install', 'apt remove', 'yum install', 'dnf install', 'pip install', 'npm install', 'npm start', 'npm run',
  // Git
  'git status', 'git add .', 'git commit -m', 'git push', 'git pull', 'git clone', 'git log', 'git log --oneline', 'git diff', 'git branch', 'git checkout', 'git merge', 'git stash',
  // Docker
  'docker ps', 'docker ps -a', 'docker images', 'docker pull', 'docker run', 'docker exec -it', 'docker stop', 'docker rm', 'docker logs', 'docker compose up', 'docker compose down',
  // 进程管理
  'systemctl status', 'systemctl start', 'systemctl stop', 'systemctl restart', 'systemctl enable', 'journalctl', 'nohup',
  // 编辑器
  'vim', 'nano', 'vi',
]

export function getCompletions(input) {
  if (!input || input.length < 2) return []

  const lower = input.toLowerCase()
  const matches = commonCommands
    .filter(cmd => cmd.toLowerCase().startsWith(lower) && cmd.toLowerCase() !== lower)
    .slice(0, 8)

  return matches
}

export function getSmartCompletions(input, history = []) {
  // First check common commands
  let completions = getCompletions(input)

  // Then check command history
  if (history.length > 0) {
    const historyMatches = history
      .filter(cmd => cmd.toLowerCase().startsWith(input.toLowerCase()) && cmd.toLowerCase() !== input.toLowerCase())
      .reverse()
      .slice(0, 3)

    // Merge, history first
    completions = [...new Set([...historyMatches, ...completions])].slice(0, 8)
  }

  return completions
}

export default { getCompletions, getSmartCompletions }
