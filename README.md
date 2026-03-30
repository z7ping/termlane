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

## 功能状态

### ✅ 已完成（真实可用）

**SSH 连接**
- 密码认证 / 密钥认证
- 连接配置 CRUD（增删改查）
- 配置导入/导出（JSON）
- 连接分组管理
- 测试连接

**终端（基础）**
- xterm.js 终端模拟器
- 多标签页（双击关闭/右键菜单）
- Split Pane（分屏）
- 搜索（Ctrl+Shift+F）
- 命令历史（↑↓ 切换）
- 鼠标选中自动复制 + 右键粘贴
- 快捷键（Ctrl+C/L/T/W/F11/?）
- Toast 通知

**SFTP（基础）**
- 本地目录浏览
- 文件图标（按扩展名）

**其他**
- 快捷命令管理
- 端口转发 UI
- 批量命令执行 UI
- 会话录制 UI
- 设置面板（字体/光标/超时）
- 快捷键帮助 / 新手引导
- 多语言（中/英）
- 浏览器模拟模式

### ⚠️ 部分完成（基础可用，需完善）

- 连接监控：有真实数据，但缺图表趋势（实时折线图）
- 会话录制：可录制+回放，但缺实时波形预览

---

## 开发计划（按优先级排序）

### P0 — 核心基础（不完成不能用）

- [x] **PTY Shell** — ssh2 `channel_shell()` 持久会话 + 实时 I/O 流，支持 vim/top/htop
- [x] **SFTP 远程操作** — `ls/cat` 真实目录列表、上传/下载
- [x] **SFTP 文件操作** — 重命名、删除、创建目录、修改权限、在线编辑
- [x] **跳板机连接** — 后端实现 SSH 隧道跳转
- [x] **远程文件在线编辑** — 读取远程文件 → 编辑 → 保存回写
- [x] **连接健康监控** — 实时 CPU/内存/磁盘/负载数据流

### P1 — 差异化功能（追平原版）

- [x] **连接速度测试** — 真实 TCP ping 延迟测试，柱状图 + 统计 + 评级
- [x] **会话录制** — 监听 PTY 输出实时录制，asciinema v2 格式，支持回放+导出
- [x] **凭证安全存储** — keyring 集成，密码存 OS 密钥链
- [x] **连接标签颜色** — 红=生产/黄=测试/绿=开发/蓝=一般/紫=特殊
- [x] **监控/测速/录制视图** — App.vue 新增三个独立视图模式
- [ ] **本地终端** — 不走 SSH，直接打开本地 shell
- [ ] **SFTP 增强** — 批量选择（Shift/Ctrl）、拖拽移动

### P2 — 完善体验

- [ ] **笔记模块** — 连接关联的笔记，支持 Markdown
- [ ] **多级分组** — 无限层级嵌套文件夹
- [ ] **书签** — 常用远程目录快速跳转
- [ ] **快捷动作** — 宏录制/回放
- [ ] **代理配置** — 全局 HTTP/SOCKS5 代理设置 UI
- [ ] **会话录制** — asciinema 格式真实录制 + 回放
- [ ] **断线自动重连** — 连接断开后指数退避重试
- [ ] **空闲超时** — 无操作自动断开

### P3 — 工程优化

- [ ] **窗口状态持久化** — 记住大小/位置/多显示器
- [ ] **虚拟滚动** — 大目录性能优化
- [ ] **启动优化** — 首屏 < 500ms
- [ ] **错误边界** — 防止单组件崩溃
- [ ] **打包** — Linux/Windows/macOS 安装包

---

## 目录结构

```
├── src/                         # Vue 3 前端
│   ├── App.vue                  # 主入口
│   ├── utils/
│   │   ├── tauri.js             # Tauri IPC + 浏览器模拟
│   │   └── i18n.js              # 多语言
│   └── components/              # 组件
│       ├── TerminalPanel.vue    # 终端
│       ├── SftpPanel.vue        # SFTP 文件管理
│       ├── BatchCommand.vue     # 批量命令
│       ├── QuickCommands.vue    # 快捷命令
│       ├── PortForward.vue      # 端口转发
│       ├── ConnectionMonitor.vue# 连接监控
│       ├── ConnectionDialog.vue # 连接对话框
│       └── ...                  # 其他组件
├── src-tauri/                   # Rust 后端
│   └── src/
│       ├── main.rs              # Tauri 命令注册
│       ├── ssh.rs               # SSH 连接 + PTY
│       ├── config.rs            # 配置持久化
│       └── sftp.rs              # SFTP 文件操作
└── build.sh                     # 构建脚本
```

## License

MIT
