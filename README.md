# XTerminal Pro

轻量级 SSH 终端 + SFTP + 多服务器管理工具，基于 Tauri 构建。

## 特性

- 🚀 内存占用仅 ~30MB（vs Electron ~200MB）
- 🔐 SSH 密码/密钥/证书认证 + 代理跳板
- 📁 SFTP 双栏文件管理（拖拽上传下载）
- 🖥️ 多服务器分组管理 + 加密存储
- 💻 终端 split pane + 搜索 + 会话录制
- 🎨 暗色/亮色主题

## 技术栈

- **前端**: Vue 3 + xterm.js + TailwindCSS
- **后端**: Rust (Tauri) + ssh2-rs + tokio
- **通信**: Tauri IPC + WebSocket
- **存储**: AES 加密配置文件

## 开发

```bash
# 安装依赖
npm install
cargo install tauri-cli

# 开发模式
cargo tauri dev

# 构建
cargo tauri build
```

## License

MIT
