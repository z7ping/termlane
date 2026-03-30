# XTerminal Pro

轻量级 SSH 终端 + SFTP + 多服务器管理工具，基于 Tauri 构建。

> 比 Xterminal 更轻量，比 Electron 更省内存（~30MB vs ~200MB）。

## 特性

- 🚀 内存占用 ~30MB，安装包 ~8MB
- 🔐 SSH 密码/密钥认证 + 代理跳板（跳板机中转）
- 📁 SFTP 双栏文件管理 + 传输队列
- 🖥️ 多服务器分组 + 连接监控 + 速度测试
- 💻 终端分屏 + 搜索 + 命令历史 + 会话录制
- ⚡ 批量命令执行 + 端口转发 + 快捷命令
- 🔍 自动更新 + 导入导出连接配置
- 🎨 暗色主题 + 多语言（中/英）

## 技术栈

| 层 | 技术 |
|---|------|
| 框架 | Tauri v2 |
| 前端 | Vue 3 + Vite + TailwindCSS |
| 终端 | xterm.js |
| 后端 | Rust (ssh2-rs + tokio) |

## 快速开始

```bash
# 安装前端依赖
npm install

# 浏览器开发模式（模拟SSH，无需安装Tauri）
npm run dev
```

浏览器打开 http://localhost:1420 即可体验完整 UI + 模拟终端。

## 打包发布

### 1. 安装系统依赖

**Linux (Debian/Ubuntu):**
```bash
sudo apt install -y pkg-config libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev
```

**macOS:**
```bash
xcode-select --install
```

**Windows（详细步骤）：**

```powershell
# 1. 安装 Visual Studio Build Tools
#    下载地址: https://visualstudio.microsoft.com/visual-cpp-build-tools/
#    运行安装器 → 选择 "Desktop development with C++" → 安装
#    （约 6GB，需要约 15 分钟）

# 2. 安装 Rust
#    方式一：winget
winget install Rustlang.Rustup

#    方式二：手动下载
#    访问 https://rustup.rs → 下载 rustup-init.exe → 运行安装
#    安装完成后重启终端

# 3. 验证 Rust 安装
rustc --version
cargo --version

# 4. 安装 Node.js
winget install OpenJS.NodeJS
# 或访问 https://nodejs.org 下载 LTS 版本安装

# 5. 安装 Tauri CLI
cargo install tauri-cli

# 6. 拉取代码
git clone https://gitea.7ping.site/ai-area/xterminal-pro.git
cd xterminal-pro

# 7. 安装前端依赖
npm install

# 8. 开发模式预览
cargo-tauri dev

# 9. 打包 exe
cargo-tauri build

# 输出: src-tauri\target\release\bundle\msi\XTerminal Pro_0.1.0_x64.msi
```

### 2. 安装 Rust + Tauri CLI

```bash
# 安装 Rust（如已安装跳过）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安装 Tauri CLI
cargo install tauri-cli
```

### 3. 构建

```bash
cd xterminal-pro

# 开发模式（实时预览 + 热重载）
cargo-tauri dev

# 打包发布
cargo-tauri build

# 或用构建脚本（Linux）
chmod +x build.sh && ./build.sh
```

### 4. 输出位置

| 平台 | 输出路径 |
|------|---------|
| Linux | `src-tauri/target/release/bundle/deb/*.deb` |
| macOS | `src-tauri/target/release/bundle/dmg/*.dmg` |
| Windows | `src-tauri/target/release/bundle/msi/*.msi` |

## 目录结构

```
├── src/                         # Vue 3 前端
│   ├── App.vue                  # 主入口
│   ├── utils/
│   │   ├── tauri.js             # Tauri IPC + 浏览器模拟
│   │   └── i18n.js              # 多语言
│   └── components/
│       ├── TerminalPanel.vue    # 终端（搜索/分屏/命令历史）
│       ├── SftpPanel.vue        # SFTP 文件管理
│       ├── BatchCommand.vue     # 批量命令执行
│       ├── QuickCommands.vue    # 快捷命令管理
│       ├── PortForward.vue      # 端口转发
│       ├── ConnectionMonitor.vue# 连接健康监控
│       ├── SpeedTest.vue        # 连接速度测试
│       ├── SessionRecorder.vue  # 会话录制
│       ├── ConnectionDialog.vue # 连接对话框（含跳板配置）
│       ├── ImportExport.vue     # 配置导入导出
│       ├── Settings.vue         # 设置面板
│       ├── ShortcutHelp.vue     # 快捷键帮助
│       ├── Onboarding.vue       # 新手引导
│       ├── Toast.vue            # 通知组件
│       ├── Tooltip.vue          # 提示气泡
│       ├── UpdateNotifier.vue   # 自动更新
│       ├── Sidebar.vue          # 连接列表
│       ├── TabBar.vue           # 标签页
│       ├── StatusBar.vue        # 状态栏
│       └── TitleBar.vue         # 标题栏
├── src-tauri/                   # Rust 后端
│   └── src/
│       ├── main.rs              # 10+ Tauri 命令
│       ├── ssh.rs               # SSH 连接（密码/密钥/跳板）
│       ├── config.rs            # 配置持久化
│       └── sftp.rs              # SFTP 文件操作
├── docs/
│   ├── SPEC.md                  # 设计文档
│   ├── PLAN.md                  # 实现计划
│   └── TODO.md                  # 待办清单
└── build.sh                     # 构建脚本
```

## License

MIT
