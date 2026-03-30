# XTerminal Pro

轻量级 SSH 终端 + SFTP + 多服务器管理工具，基于 Tauri 构建。

> 比 Xterminal 更轻量，比 Electron 更省内存。

## 特性

- 🚀 内存占用 ~30MB（Electron ~200MB）
- 🔐 SSH 密码/密钥认证 + 代理跳板
- 📁 SFTP 双栏文件管理（拖拽上传下载）
- 🖥️ 多服务器分组管理 + 加密存储
- 💻 终端 split pane + 搜索 + 会话录制
- ⚡ 快捷命令管理 + 端口转发
- 🎨 暗色主题

## 技术栈

| 层 | 技术 |
|---|------|
| 框架 | Tauri v2 |
| 前端 | Vue 3 + Vite + TailwindCSS |
| 终端 | xterm.js |
| 后端 | Rust (ssh2-rs + tokio) |

## 快速开始

```bash
# 安装依赖
npm install

# 浏览器开发模式（模拟SSH）
npm run dev

# Tauri 开发模式
npm run tauri:dev

# 构建
npm run tauri:build
```

## 目录结构

```
├── src/                    # Vue 3 前端
│   ├── App.vue
│   ├── components/
│   │   ├── TerminalPanel.vue   # 终端（搜索/分屏）
│   │   ├── SftpPanel.vue       # SFTP文件管理
│   │   ├── QuickCommands.vue   # 快捷命令
│   │   ├── PortForward.vue     # 端口转发
│   │   ├── Sidebar.vue         # 连接列表
│   │   ├── TabBar.vue          # 标签页
│   │   ├── StatusBar.vue       # 状态栏
│   │   ├── TitleBar.vue        # 标题栏
│   │   └── ConnectionDialog.vue # 连接对话框
│   └── utils/tauri.js          # Tauri IPC + 浏览器模拟
├── src-tauri/              # Rust 后端
│   └── src/
│       ├── main.rs         # Tauri命令入口
│       ├── ssh.rs          # SSH连接管理
│       ├── config.rs       # 配置持久化
│       └── sftp.rs         # SFTP文件操作
└── docs/
    ├── SPEC.md             # 设计文档
    └── PLAN.md             # 实现计划
```

## 已实现功能

| 功能 | 状态 |
|------|------|
| SSH 密码认证 | ✅ |
| SSH 密钥认证 | ✅ |
| 终端模拟器 | ✅ |
| Split Pane | ✅ |
| 终端搜索 | ✅ |
| 多标签页 | ✅ |
| 连接分组 | ✅ |
| 测试连接 | ✅ |
| SFTP 文件管理 | ✅ |
| 快捷命令 | ✅ |
| 端口转发 UI | ✅ |
| 浏览器模拟模式 | ✅ |
| 暗色主题 | ✅ |

## License

MIT
