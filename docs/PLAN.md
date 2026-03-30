# XTerminal Pro — 实现计划

## 阶段一：项目骨架（Day 1-2）

- [x] 1.1 Tauri 项目初始化（`cargo tauri init`）
- [x] 1.2 前端工程搭建（Vue 3 + Vite + TailwindCSS）
- [x] 1.3 基础布局（侧边栏 + 主内容区 + 状态栏）
- [x] 1.4 暗色/亮色主题切换
- [x] 1.5 Rust 后端基础结构（配置存储模块）

## 阶段二：SSH 连接（Day 3-4）

- [x] 2.1 SSH 连接核心（ssh2-rs，密码认证）
- [x] 2.2 SSH 密钥认证
- [x] 2.3 连接配置 CRUD（增删改查）
- [x] 2.4 配置加密存储（keyring + AES）
- [x] 2.5 连接分组管理

## 阶段三：终端模拟器（Day 5-6）

- [x] 3.1 xterm.js 集成
- [x] 3.2 SSH ↔ 终端数据桥接（WebSocket）
- [x] 3.3 多标签页管理
- [x] 3.4 终端搜索功能
- [x] 3.5 Split Pane 支持

## 阶段四：SFTP 文件管理（Day 7-9）

- [x] 4.1 SFTP 会话管理
- [x] 4.2 远程目录浏览（树形 + 列表）
- [x] 4.3 本地目录浏览
- [x] 4.4 文件上传下载（带进度条）
- [x] 4.5 拖拽操作支持
- [x] 4.6 传输队列管理

## 阶段五：增强功能（Day 10-12）

- [x] 5.1 快捷命令管理（CRUD + 一键执行）
- [x] 5.2 端口转发（本地/远程/动态）
- [x] 5.3 会话录制与回放
- [x] 5.4 代理跳板支持
- [x] 5.5 批量命令执行（多服务器）

## 阶段六：打磨发布（Day 13-14）

- [ ] 6.1 打包构建（Windows/macOS/Linux）
- [ ] 6.2 自动更新机制
- [ ] 6.3 性能优化
- [ ] 6.4 文档和截图
- [ ] 6.5 发布 v1.0.0

---

## 依赖清单

### Rust
- `tauri` (v2) — 框架
- `ssh2` — SSH 连接
- `tokio` — 异步运行时
- `serde` + `serde_json` — 序列化
- `keyring` — 系统密钥环
- `aes-gcm` — 加密
- `dirs` — 系统目录

### 前端
- `vue` (v3) — UI 框架
- `xterm` + `xterm-addon-fit` + `xterm-addon-search` — 终端
- `tailwindcss` — 样式
- `@tauri-apps/api` — Tauri IPC
