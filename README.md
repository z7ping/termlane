# XTerminal Pro

轻量级 SSH 终端 + SFTP + 多服务器管理工具，基于 Tauri 构建。

> 比 Xterminal 更轻量，比 Electron 更省内存（~30MB vs ~200MB）。

## 技术栈

| 层 | 技术 |
|---|------|
| 框架 | Tauri v2 |
| 前端 | Vue 3 + Vite + TailwindCSS |
| 终端 | xterm.js |
| 后端 | Rust (ssh2-rs + tokio) |

## 快速开始

```bash
npm install
npm run dev
```

浏览器打开 http://localhost:1420 即可体验完整 UI + 模拟终端。

---

## 已实现功能（26 项）

### SSH 连接
- ✅ 密码认证
- ✅ SSH 密钥认证
- ✅ 代理跳板（跳板机中转连接内网服务器）
- ✅ 连接配置 CRUD（增删改查）
- ✅ 测试连接（先测后保存）
- ✅ 连接分组管理
- ✅ 配置导入/导出（JSON）
- ✅ 连接健康监控（延迟/CPU/内存/磁盘/运行时间）
- ✅ 连接速度测试（延迟柱状图 + 评级）

### 终端
- ✅ xterm.js 终端模拟器
- ✅ 多标签页（双击关闭/右键菜单/启动恢复）
- ✅ Split Pane（分屏）
- ✅ 搜索（Ctrl+Shift+F）
- ✅ 命令历史（↑↓ 切换）
- ✅ 鼠标选中自动复制 + 右键粘贴
- ✅ 快捷键（Ctrl+C/L/T/W/B/F11/?）
- ✅ Toast 通知系统

### SFTP 文件管理
- ✅ 本地 + 远程双栏
- ✅ 目录浏览（双击进入）
- ✅ 文件上传/下载 + 传输队列
- ✅ 文件图标（按扩展名）
- ✅ 路径手动输入

### 增强功能
- ✅ 快捷命令管理（预设 + 自定义）
- ✅ 端口转发（本地/远程/SOCKS5）
- ✅ 批量命令执行（多服务器并行）
- ✅ 会话录制 + 回放 + 导出 asciinema 格式
- ✅ 设置面板（字体/光标/超时）
- ✅ 快捷键帮助（?/F1）
- ✅ 新手引导（首次打开 5 步引导）
- ✅ 自动更新检查
- ✅ 多语言（中/英）
- ✅ 浏览器模拟模式（无 Tauri 也能用）

---

## 待办功能（37 项）

### 🔴 打包发布（4 项）
- [ ] Linux 打包 — 需先安装依赖（见下方）
- [ ] Windows 打包 — 需 VS Build Tools + Rust（见下方）
- [ ] macOS 打包 — 需 Xcode（见下方）
- [ ] 发布 v1.0.0 到 Gitea Releases

### 🟡 功能增强（5 项）
- [x] 终端链接可点击（Ctrl+点击打开浏览器）
- [x] 命令自动补全（Tab 补全 + 模糊匹配）
- [ ] 多服务器对比视图（并排显示多台输出）
- [x] 终端配色方案（Dracula/Monokai/Solarized 等）
- [ ] 快捷键自定义（用户自己设置）

### 🟡 终端交互（5 项）
- [x] 命令完成提醒（后台命令跑完响铃/闪标签）
- [ ] 终端拖拽文件自动上传到当前目录
- [x] 选中文本右键菜单（复制/搜索/翻译）
- [ ] 终端内嵌小窗（查看长输出不丢失上下文）
- [ ] 分屏同步输入（打字同时发给两个终端）

### 🟡 文件管理（4 项）
- [ ] 右键菜单（重命名/压缩/权限/详情）
- [ ] 文件预览（图片/文本/JSON 内联预览）
- [ ] 批量选择（Shift+点击范围选/Ctrl+点击多选）
- [ ] 拖拽移动文件

### 🟡 连接管理（4 项）
- [x] 连接标签颜色区分（生产红/测试黄/开发绿）
- [ ] 按延迟自动排序（最快排前面）
- [x] 空闲超时自动断开
- [ ] 断线自动重连

### 🟡 布局（3 项）
- [x] 记住上次窗口大小和位置
- [ ] 多显示器记住位置
- [ ] 画中画模式（终端悬浮在其他窗口上方）

### 🟡 辅助功能（3 项）
- [x] 截图功能（终端区域截图保存）
- [x] 命令片段库（从网上复制的命令直接执行）
- [x] 定时执行（设定时任务自动执行命令）

### 🟢 工程优化（5 项）
- [ ] 错误边界组件（防止单组件崩溃）
- [ ] 虚拟滚动（大目录性能优化）
- [ ] 懒加载（非活跃标签不渲染）
- [ ] 配置加密存储（keyring/AES）
- [ ] 启动性能优化（首屏 < 500ms）

### 🟢 遗留小项（4 项）
- [ ] 拖拽排序连接/标签
- [ ] 收藏/置顶连接
- [ ] 标签页固定（钉住）
- [ ] 面板折叠/展开动画

---

## 打包发布

### Linux

```bash
# 安装系统依赖
sudo apt install -y pkg-config libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev

# 安装 Rust + Tauri CLI
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
cargo install tauri-cli

# 构建
cd xterminal-pro
npm install
cargo-tauri build

# 输出: src-tauri/target/release/bundle/deb/*.deb
```

### macOS

```bash
# 安装 Xcode Command Line Tools
xcode-select --install

# 安装 Rust + Tauri CLI
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
cargo install tauri-cli

# 构建
cd xterminal-pro
npm install
cargo-tauri build

# 输出: src-tauri/target/release/bundle/dmg/*.dmg
```

### Windows

```powershell
# 1. 安装 Visual Studio Build Tools
#    下载: https://visualstudio.microsoft.com/visual-cpp-build-tools/
#    选择 "Desktop development with C++" → 安装（约 6GB）

# 2. 安装 Rust
winget install Rustlang.Rustup
# 或下载 rustup-init.exe: https://rustup.rs
# 安装后重启终端

# 3. 验证
rustc --version
cargo --version

# 4. 安装 Node.js
winget install OpenJS.NodeJS

# 5. 安装 Tauri CLI
cargo install tauri-cli

# 6. 拉取代码 + 构建
git clone https://gitea.7ping.site/ai-area/xterminal-pro.git
cd xterminal-pro
npm install
cargo-tauri dev        # 开发模式
cargo-tauri build      # 打包

# 输出: src-tauri\target\release\bundle\msi\XTerminal Pro_0.1.0_x64.msi
```

---

## 目录结构

```
├── src/                         # Vue 3 前端
│   ├── App.vue                  # 主入口
│   ├── utils/
│   │   ├── tauri.js             # Tauri IPC + 浏览器模拟
│   │   └── i18n.js              # 多语言
│   └── components/              # 17 个组件
│       ├── TerminalPanel.vue    # 终端（搜索/分屏/命令历史）
│       ├── SftpPanel.vue        # SFTP 文件管理
│       ├── BatchCommand.vue     # 批量命令执行
│       ├── QuickCommands.vue    # 快捷命令
│       ├── PortForward.vue      # 端口转发
│       ├── ConnectionMonitor.vue# 连接监控
│       ├── SpeedTest.vue        # 速度测试
│       ├── SessionRecorder.vue  # 会话录制
│       ├── ConnectionDialog.vue # 连接对话框
│       ├── ImportExport.vue     # 导入导出
│       ├── Settings.vue         # 设置
│       ├── ShortcutHelp.vue     # 快捷键帮助
│       ├── Onboarding.vue       # 新手引导
│       ├── Toast.vue / Tooltip.vue / UpdateNotifier.vue
│       ├── Sidebar.vue / TabBar.vue / StatusBar.vue / TitleBar.vue
├── src-tauri/                   # Rust 后端
│   └── src/
│       ├── main.rs              # 10+ Tauri 命令
│       ├── ssh.rs               # SSH（密码/密钥/跳板）
│       ├── config.rs            # 配置持久化
│       └── sftp.rs              # SFTP
└── build.sh                     # 构建脚本
```

## License

MIT
