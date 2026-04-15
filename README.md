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

## 🔍 工程审查发现问题

> 审查日期：2026-04-16  
> 审查技能：superpowers-tdd + UI/UX + 安全 + 测试

### 🔴 高优先级问题

#### 1. 安全漏洞
- **Vite 高危漏洞**：vite <=6.4.1 存在两个高危安全漏洞
  - 路径遍历漏洞 (GHSA-4w7w-66w2-5vf9)
  - 任意文件读取漏洞 (GHSA-p9ff-h696-f583)
  - 修复：运行 `npm audit fix`
- **xterm 依赖过时**：使用 `xterm@^5`（已弃用），应迁移到 `@xterm/xterm`
- **localStorage 敏感存储**：58处使用 localStorage，可能存储敏感信息

#### 2. TDD/测试问题（违反 superpowers-tdd）
- ❌ **只有一个 smoke.test.js**，没有真正的单元测试
- ❌ **没有遵循 RED-GREEN-REFACTOR 循环**
- ❌ **核心功能无测试**：SSH连接、SFTP、配置管理、窗口状态管理等都没有测试
- ⚠️ **建议**：所有新功能必须先写测试再写实现

#### 3. UI/UX 问题
- ❌ **可访问性为 0**：0处使用 aria-* 标签
- ⚠️ **字体太小**：不符合UI标准
- ⚠️ **弹窗点击背景关闭**：新建连接弹窗点击背景会自动关闭

### 🟡 中优先级问题

#### 4. 技术债务
- ⚠️ **没有 TypeScript**：使用 `main.js` 而非 `main.ts`
- ⚠️ **代码复杂度高**：Onboarding.vue 有 628 行，TerminalPanel.vue 有 523 行，SftpPanel.vue 有 516 行
- ⚠️ **没有 tsconfig.json**
- ⚠️ **错误处理不够细**：虽有99处 try/catch/throw，但粒度不够

### 🟢 低优先级问题

#### 5. 工程优化
- 💡 **建议添加环境配置**：没有 .env 文件
- 💡 **建议添加预提交钩子**：确保代码质量
- 💡 **建议添加代码格式化配置**：统一代码风格

---

## 环境要求

| 工具 | 版本 |
|------|------|
| Node.js | ≥18.0.0 |
| Rust | ≥1.70.0 |
| npm | ≥9.0.0 |

## 系统依赖

### Ubuntu/Debian
```bash
sudo apt install build-essential libssl-dev pkg-config
```

### macOS
```bash
xcode-select --xinstall
```

### Windows
安装 Visual Studio Build Tools + Rust

## 安装

### 从源码构建
```bash
npm install
npm run tauri build
```

### 预构建包
前往 [Releases](https://github.com/7ping/XTerminal-Pro/releases) 页面下载对应平台的安装包。

## 配置

- **配置文件**：
  - Linux/macOS：`~/.config/xterminal-pro/`
  - Windows：`%APPDATA%\xterminal-pro\`
- **连接配置**：`~/.config/xterminal-pro/connections.json`
- **凭证存储**：系统密钥链（keyring）

---

## 故障排除

### 连接失败
- 检查网络连通性：`ping <host>`
- 确认 SSH 端口（默认 22）开放是否正常
- 检查用户名密码是否正确

### 密钥认证失败
- 确保私钥文件权限为 600
- 检查私钥格式（OpenSSH 格式）

### 编译失败
- 确保 Rust 工具链已更新：`rustup update`
- 检查系统依赖是否安装完整

### 应用启动失败
- 检查配置文件是否损坏，尝试删除 `~/.config/xterminal-pro/` 重新开始
- 查看应用日志获取详细错误信息

---

## 更新日志

### v2.0.0 (当前)
- ✅ 完成 33 项核心功能
- ✅ SSH 密码/密钥/跳板机认证
- ✅ PTY Shell 持久会话（支持 vim/top/htop）
- ✅ SFTP 文件管理（上传/下载/编辑/权限）
- ✅ 连接健康监控 + 速度测试
- ✅ 会话录制（asciinema v2）
- ✅ 凭证安全存储（OS 密钥链）
- ✅ 多级分组 + 标签颜色

---

## 快捷键

| 快捷键 | 功能 |
|--------|------|
| `Ctrl+C` | 复制选中内容（终端内） / 发送中断信号 |
| `Ctrl+L` | 清空终端屏幕 |
| `Ctrl+T` | 新建终端标签页 |
| `Ctrl+W` | 关闭当前标签页 |
| `Ctrl+F11` | 全屏切换 |
| `Ctrl+?` | 显示快捷键帮助 |
| `Ctrl+Shift+F` | 终端内搜索 |
| `↑/↓` | 命令历史切换 |
| `右键` | 粘贴剪贴板内容 |

---

## 功能状态

### ✅ 已完成（33 项，真实可用）

**SSH 连接**
- 密码认证 / 密钥认证 / 跳板机
- 连接配置 CRUD + 导入/导出（JSON）
- 连接分组管理 + 标签颜色（红/黄/绿/蓝/紫）
- 测试连接（真实 TCP ping）
- 断线自动重连（指数退避 3 次）
- 空闲超时（30 分钟自动断开）

**终端**
- PTY Shell 持久会话（支持 vim/top/htop）— SSH + 本地
- xterm.js 终端模拟器（WebGL 加速）
- 多标签页（双击关闭/右键菜单/启动恢复）
- Split Pane（分屏）
- 搜索（Ctrl+Shift+F）
- 命令历史（↑↓ 切换）
- 鼠标选中自动复制 + 右键粘贴
- 快捷键（Ctrl+C/L/T/W/F11/?）

**SFTP 文件管理**
- 本地 + 远程双栏（真实 SSH 执行 ls/cat）
- 目录浏览（双击进入）
- 文件上传/下载 + 传输队列
- 批量选择（Shift+click 范围/Ctrl+click 多选）
- 拖拽上传/移动
- 右键菜单（重命名/删除/权限/编辑/复制路径）
- 远程文件在线编辑（dirty indicator）
- 批量删除

**监控 & 测试**
- 连接健康监控（CPU/内存/磁盘/负载/运行时间，真实数据）
- 连接速度测试（真实 TCP ping + 延迟柱状图 + 评级）
- 自动刷新（5 秒间隔）

**会话管理**
- 会话录制（监听 PTY 输出，asciinema v2 格式）
- 录制回放 + 导出 asciinema

**增强功能**
- 快捷命令管理 → 直接发送到终端
- 端口转发（本地/远程/SOCKS5）
- 批量命令执行
- 笔记模块（Markdown，按连接存储）
- 书签（远程目录快速跳转）
- 代理配置（HTTP/SOCKS5，认证+绕过列表）
- 凭证安全存储（OS 密钥链/keyring）

**应用**
- 设置面板（字体/光标/超时）
- 快捷键帮助 / 新手引导
- 多语言（中/英）
- 窗口状态持久化
- ErrorBoundary 错误边界
- 多级分组（"/" 分隔符嵌套）
- 12 个视图模式（终端/文件/批量/监控/测速/录制/笔记/书签/代理/命令/转发/定时）
- 浏览器模拟模式（无 Tauri 也能用 UI）

### ⚠️ 待完善

- 连接监控：有真实数据，缺实时折线图趋势
- 多级分组：当前为扁平分组
- 快捷动作（宏录制）

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
- [x] **SFTP 批量操作** — Shift/Ctrl 多选、批量上传/下载/删除
- [x] **SFTP 拖拽** — 本地→远程拖拽上传、远程目录间拖拽移动
- [x] **本地终端** — portable-pty 本地 shell，支持 vim/top/htop

### P2 — 完善体验

- [x] **SFTP 批量选择** — Shift+click 范围选、Ctrl+click 多选、批量删除
- [x] **SFTP 拖拽** — 本地拖到远程面板上传、拖到目录移动、远程拖拽排序
- [x] **SFTP 右键菜单增强** — 编辑/重命名/下载/权限/复制路径
- [x] **SFTP 在线编辑** — 未保存标记 dirty indicator
- [x] **断线自动重连** — 连接断开后指数退避重试（最多3次）
- [x] **空闲超时** — 30 分钟无操作自动断开
- [x] **窗口状态持久化** — 关闭时保存窗口位置大小
- [x] **笔记模块** — 连接关联的笔记，支持 Markdown 编辑
- [x] **书签** — 常用远程目录快速跳转
- [x] **代理配置** — HTTP/SOCKS5 全局代理设置 UI，认证+绕过列表
- [x] **快捷命令→终端** — 点击快捷命令直接发送到活跃终端
- [x] **ErrorBoundary** — 错误边界包裹主要视图区域
- [x] **多级分组** — "/" 分隔符嵌套，树形展示，收藏优先
- [x] **定时任务** — cron 表达式，预设模板，启用/暂停

### P3 — 工程优化

- [x] **窗口状态持久化** — 关闭时保存大小/位置
- [x] **ErrorBoundary** — 错误边界包裹主要视图
- [x] **虚拟滚动** — SFTP 大目录（>100文件）自动启用 VirtualList
- [x] **打包** — Linux .deb 构建成功（安装包 4.9MB，二进制 16MB）
- [x] **启动优化** — 懒加载 18 个组件，24 个异步 chunk，首屏 119KB gzip
- [x] **错误边界** — 防止单组件崩溃
- [ ] **打包** — Windows/macOS 安装包

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

## 开发工作流
- 默认在 dev 分支开发
- 完成后创建 PR: dev → main
- 由维护者审核合并
