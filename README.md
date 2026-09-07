# XTerminal Pro

一个面向开发者日常多服务器工作的轻量桌面终端。

目标不是堆功能，而是把常用的 **SSH + 终端 + SFTP + 多服务器管理** 做得启动快、资源占用低、操作直接、配置尽量少。

> 当前处于 **0.1.0 / 1.0 收口阶段**。仓库中的历史 README、PLAN 或设计文档如与当前源码冲突，以当前源码和构建配置为准。

## 产品原则

- **轻量优先**：不为了减少几十行代码而引入整套框架或大型依赖。
- **核心优先**：终端、SSH、SFTP 和连接管理优先于低频增强功能。
- **成熟能力优先复用**：协议、安全、窗口状态、终端模拟等通用基础能力优先采用官方或成熟实现。
- **不做假完成**：有组件、有命令或有 PLAN 勾选，不等于功能已经可发布。
- **桌面原生链路才算真实验证**：浏览器开发模式只用于 UI / mock，不代表 SSH、PTY、SFTP 已通过真实运行验证。

## 技术栈

| 层 | 技术 |
| --- | --- |
| 桌面框架 | Tauri v2 |
| 前端 | Vue 3 + Vite + TailwindCSS |
| 终端 | `@xterm/xterm` + 官方 addons |
| 后端 | Rust + `ssh2` + Tokio |
| 本地终端 | `portable-pty` |
| 凭证 | OS Keyring |
| 窗口状态 | Tauri 官方 `window-state` 插件 |

## 当前核心能力

### SSH / 终端

- 密码认证
- 私钥认证
- 交互式 PTY Shell
- 本地终端
- 多标签页
- 终端搜索
- 分屏
- 自动重连相关逻辑
- 连接延迟检测
- `~/.ssh/known_hosts` 主机密钥检查

### SFTP / 文件

- 本地 / 远程双栏文件浏览
- 上传 / 下载
- 批量选择和传输队列
- 新建目录、重命名、删除、权限修改
- 拖拽操作
- 远程文本文件在线编辑
- 大目录虚拟列表

### 多服务器工具

项目中还存在批量命令、监控、测速、录制、笔记、书签、代理、快捷命令、端口转发、定时任务和宏等页面/模块。

这些增强能力正在 1.0 收口中逐项核对真实实现链路。**页面存在不代表已经达到发布质量。**

## 当前明确限制

### 跳板机尚不是真实 ProxyJump

当前 `ssh_connect_jump` 的历史实现会：

1. 连接跳板机；
2. 验证跳板机可以建立到目标机的 `direct-tcpip` 通道；
3. 随后客户端仍然直接连接目标机。

因此它目前只能算跳板可达性预检查，**不能作为真实 ProxyJump 能力对外声明**。1.0 前会重新实现或从正式能力范围移除。

### Host Key 首次信任仍需收口

当前已使用 `known_hosts`：

- 已知且匹配：允许连接；
- 已知但不匹配：拒绝连接；
- 未知主机：当前采用 TOFU（首次信任后写入）。

1.0 前仍需补齐非默认 SSH 端口的正确识别，并设计明确的首次连接指纹确认交互。

### 自动更新目前只有版本检查

当前会检查是否存在更新版本，但**没有假装执行下载安装**。

正式应用内更新会在发布链建立后接入 Tauri 官方 updater，并同时完成：

- updater artifact
- 签名私钥 / 公钥
- 更新签名
- 更新 endpoint / 静态 JSON
- 安装与回滚验证

## 快速开始

### 浏览器 UI 开发

```bash
npm install
npm run dev
```

默认开发地址：`http://localhost:1420`。

浏览器模式使用 mock / fallback，只适合 UI 和前端逻辑开发。

### Tauri 桌面开发

```bash
npm install
npm run tauri:dev
```

真实 SSH / PTY / SFTP 行为必须在 Tauri 桌面运行时验证。

## 测试与检查

```bash
# 前端测试
npm test

# 冒烟测试
npm run test:smoke

# TypeScript / Vue 类型检查
npx vue-tsc --noEmit

# 前端生产构建
npm run build

# Rust（在 src-tauri/ 下）
cargo check
cargo test
```

## 构建

```bash
npm run tauri:build
```

Tauri 配置当前声明的 bundle target 包括：

- Windows: MSI
- Linux: DEB
- macOS: DMG / APP

是否作为 1.0 正式支持平台，以实际 CI / 安装验证结果为准，不以配置中存在 target 为准。

## 项目结构

```text
src/
├── main.ts
├── App.vue
├── components/          Vue 视图和组件
├── composables/         前端状态 / 交互逻辑
├── utils/               前端工具与浏览器 fallback
└── __tests__/           Vitest 测试

src-tauri/
├── tauri.conf.json
├── capabilities/
└── src/
    ├── lib.rs           Tauri Builder / 插件 / command 注册
    ├── commands.rs      IPC 参数验证与调用入口
    ├── ssh.rs           SSH / PTY
    ├── sftp.rs          文件操作
    ├── local_pty.rs     本地终端
    ├── config.rs        连接配置 / Keyring / 录制元数据
    └── updater.rs       Release 版本检查
```

## 1.0 收口路线

当前停止无边界扩功能，按三个方向推进：

1. [#1 去自研化与基础能力整合](https://github.com/z7ping/xterminal-pro/issues/1)
2. [#2 UI / 交互产品化整理](https://github.com/z7ping/xterminal-pro/issues/2)
3. [#3 1.0 发布基线：核心链路、质量、安全与跨平台验证](https://github.com/z7ping/xterminal-pro/issues/3)

历史 `docs/PLAN.md` 中的勾选仅代表当时的开发记录，不能替代当前源码验收。

## 安全边界

- 桌面密码由 Rust `keyring` 写入操作系统密钥环，不写入 `connections.json`。
- Tauri CSP 已启用，不应为了兼容功能而直接关闭。
- Browser secure-storage 仅作为浏览器开发 fallback，安全等级不能等同 OS Keyring。
- Tauri capability 应坚持最小权限原则。
- SSH Host Key mismatch 必须拒绝，不允许静默绕过。

## 文档

- [架构说明](docs/ARCHITECTURE.md)
- [API](docs/API.md)
- [测试说明](docs/TESTING.md)
- [产品规格](docs/SPEC.md)
- [开发计划](docs/PLAN.md)
- [更新日志](CHANGELOG.md)

> 文档正在随 1.0 收口同步校正。涉及当前能力、版本、依赖或安全结论时，请再次以源码验证。

## License

MIT
