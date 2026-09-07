// i18n.ts - 国际化支持

interface TranslationMessages {
  [key: string]: string | TranslationMessages
}

type TranslationValue = string | TranslationMessages

const messages: Record<string, TranslationMessages> = {
  zh: {
    app: { name: 'XTerminal Pro', welcome: '欢迎使用 XTerminal Pro' },
    sidebar: { connections: '连接', add: '新建连接', search: '搜索连接...', noResults: '没有匹配的连接', empty: '点击 + 添加第一个连接', quickCmd: '快捷命令' },
    tab: { new: '新终端', close: '关闭', closeOthers: '关闭其他', closeAll: '关闭全部' },
    terminal: { searching: '搜索...', help: '输入 help 查看可用命令', connected: '已连接', disconnected: '会话已断开', connecting: '正在连接...', failed: '连接失败', copied: '已复制', local: '本地演示', split: '分屏' },
    sftp: { title: '文件', local: '本地', remote: '远程', upload: '上传 →', download: '← 下载', emptyDir: '空目录', notConnected: '未连接', transferQueue: '传输队列' },
    batch: { title: '批量命令', execute: '执行', executing: '执行中...', selectServers: '选择服务器', selectAll: '全选', history: '历史' },
    connection: { name: '名称', host: '主机', port: '端口', username: '用户名', password: '密码', authType: '认证方式', keyPath: '密钥路径', passphrase: '密钥密码', group: '分组', test: '测试连接', save: '保存', cancel: '取消', testing: '测试中...', success: '连接成功', failed: '连接失败', passwordAuth: '密码', keyAuth: 'SSH 密钥' },
    settings: { title: '设置', appearance: '外观', darkMode: '暗色主题', darkModeDesc: '切换暗色/亮色主题', fontSize: '字体大小', cursor: '光标样式', cursorDesc: '终端光标闪烁', terminal: '终端', scrollback: '滚动缓冲区', timeout: 'SSH 连接超时', shortcuts: '快捷键', about: '关于' },
    shortcuts: { search: '搜索', clear: '清屏', interrupt: '中断', newTab: '新建标签', closeTab: '关闭标签', split: '分屏', fullscreen: '全屏', help: '快捷键帮助', sidebar: '切换侧边栏', settings: '设置', newConn: '新连接' },
    toast: { saved: '已保存', deleted: '已删除', copied: '已复制', success: '成功', error: '失败', disconnected: '已断开连接', connected: '连接成功' },
    onboarding: { step1Title: '欢迎使用 XTerminal Pro', step1Desc: '轻量级 SSH 终端工具，基于 Tauri 构建。', step2Title: '连接你的服务器', step2Desc: '点击左侧 + 添加 SSH 连接，支持密码和密钥认证。', next: '下一步', prev: '上一步', start: '开始使用 🚀' },
  },
  en: {
    app: { name: 'XTerminal Pro', welcome: 'Welcome to XTerminal Pro' },
    sidebar: { connections: 'Connections', add: 'New Connection', search: 'Search connections...', noResults: 'No matching connections', empty: 'Click + to add your first connection', quickCmd: 'Quick Commands' },
    tab: { new: 'New Terminal', close: 'Close', closeOthers: 'Close Others', closeAll: 'Close All' },
    terminal: { searching: 'Searching...', help: 'Type help for available commands', connected: 'Connected', disconnected: 'Session disconnected', connecting: 'Connecting...', failed: 'Connection failed', copied: 'Copied', local: 'Local Demo', split: 'Split' },
    sftp: { title: 'Files', local: 'Local', remote: 'Remote', upload: 'Upload →', download: '← Download', emptyDir: 'Empty directory', notConnected: 'Not connected', transferQueue: 'Transfer Queue' },
    batch: { title: 'Batch Commands', execute: 'Execute', executing: 'Executing...', selectServers: 'Select Servers', selectAll: 'Select All', history: 'History' },
    connection: { name: 'Name', host: 'Host', port: 'Port', username: 'Username', password: 'Password', authType: 'Auth Type', keyPath: 'Key Path', passphrase: 'Passphrase', group: 'Group', test: 'Test Connection', save: 'Save', cancel: 'Cancel', testing: 'Testing...', success: 'Connection successful', failed: 'Connection failed', passwordAuth: 'Password', keyAuth: 'SSH Key' },
    settings: { title: 'Settings', appearance: 'Appearance', darkMode: 'Dark Mode', darkModeDesc: 'Toggle dark/light theme', fontSize: 'Font Size', cursor: 'Cursor', cursorDesc: 'Terminal cursor blink', terminal: 'Terminal', scrollback: 'Scrollback Buffer', timeout: 'SSH Timeout', shortcuts: 'Shortcuts', about: 'About' },
    shortcuts: { search: 'Search', clear: 'Clear', interrupt: 'Interrupt', newTab: 'New Tab', closeTab: 'Close Tab', split: 'Split', fullscreen: 'Fullscreen', help: 'Shortcuts Help', sidebar: 'Toggle Sidebar', settings: 'Settings', newConn: 'New Connection' },
    toast: { saved: 'Saved', deleted: 'Deleted', copied: 'Copied', success: 'Success', error: 'Error', disconnected: 'Disconnected', connected: 'Connected' },
    onboarding: { step1Title: 'Welcome to XTerminal Pro', step1Desc: 'Lightweight SSH terminal built with Tauri.', step2Title: 'Connect your servers', step2Desc: 'Click + on the left to add SSH connections.', next: 'Next', prev: 'Previous', start: 'Get Started 🚀' },
  },
}

let currentLocale: string = localStorage.getItem('xterminal_locale') || 'zh'

export function t(key: string): string {
  const keys = key.split('.')
  let result: TranslationValue | undefined = messages[currentLocale]
  for (const k of keys) {
    if (result && typeof result === 'object') {
      result = result[k]
    } else {
      return key
    }
  }
  return (typeof result === 'string' ? result : key) || key
}

export function setLocale(locale: string): void {
  currentLocale = locale
  localStorage.setItem('xterminal_locale', locale)
}

export function getLocale(): string {
  return currentLocale
}

export function getLocales(): string[] {
  return Object.keys(messages)
}

export default { t, setLocale, getLocale, getLocales }
