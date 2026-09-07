# Termlane

一个面向开发者日常多服务器工作的轻量桌面终端。

目标不是堆功能，而是把常用的 **SSH + 终端 + SFTP + 多服务器管理** 做得启动快、资源占用低、操作直接、配置尽量少。

> 当前版本元数据为 **0.1.0**，项目处于 1.0 收口阶段。仓库中的历史计划或旧文档如与当前源码冲突，以当前源码、Issue 和构建配置为准。

## 产品原则

- **轻量优先**：不为了减少少量代码而引入整套框架或大型依赖。
- **核心优先**：终端、SSH、SFTP 和连接管理优先于低频增强功能。
- **成熟能力优先复用**：协议、安全、窗口状态、终端模拟等基础能力优先采用官方或成熟实现。
- **不做假完成**：有组件、有命令或历史计划勾选，不等于已经达到发布质量。
- **桌面原生链路才算真实验证**：浏览器模式只用于 UI / mock，不代表 SSH、PTY、SFTP 已通过真实运行验证。

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
- SSH 私钥认证
- 交互式 PTY Shell
- 本地终端
- 多标签页
- 终端搜索
- 连接断开后的有限自动重连
- `~/.ssh/known_hosts` 主机身份校验
- 首次连接显示主机密钥算法和 SHA256 指纹，用户确认后才信任
- 非默认 SSH 端口按 OpenSSH `[host]:port` 规则保存和检查主机密钥

### SFTP / 文件

- 本地 / 远程双栏文件浏览
- 单文件上传 / 下载
- 多文件选择与传输队列
- 新建目录、重命名、递归删除、权限修改
- 拖拽文件操作
- 远程文本文件在线编辑
- 大目录虚拟列表

1.0 **不承诺目录递归上传 / 下载**。当前 Native 上传接口会明确拒绝目录；目录管理与递归删除是独立能力。

远端服务器必须提供 SFTP subsystem。若 SSH 可连接但 SFTP 不可用，文件工作区会显示 SFTP 操作错误；这不应影响已经建立的交互式终端会话。

### 已接入工具

- TCP 连接延迟
- 会话录制与回放
- 笔记
- 远程目录书签
- 快捷命令
- 命令序列

这些工具只按当前真实链路描述，不把页面、Mock 或未接 Native 的实现算作完成。

## 1.0 明确不支持的能力

以下能力不进入 Termlane 1.0 产品承诺：

- ProxyJump / 跳板机
- 端口转发
- 批量命令执行
- 服务器资源监控
- 定时任务
- 代理配置
- 真正的多 Pane / 分屏终端

历史仓库中曾存在这些页面、Mock 或半实现代码，目前已从 1.0 导航和主要运行链路撤下。后续重新加入时必须有真实 Native 链路和对应验收。

### ProxyJump 说明

历史 `ssh_connect_jump` 并不是真实 ProxyJump：它只验证 jump → target 的 `direct-tcpip` 通道可建立，随后客户端仍直接连接 target，因此已删除。

如果后续重新提供 ProxyJump，目标 SSH Session 必须真正建立在中间通道之上；不会为了保留入口而维护第二套半成熟 SSH 栈。

## 平台支持

Termlane 1.0 当前正式收口范围为：

- **Windows**：MSI
- **Linux**：DEB

**macOS 暂不列入 1.0 正式支持范围。** 当前没有 macOS 打包、签名和安装验证链，因此不会仅因为 Tauri 理论上支持 macOS 就宣称已支持。后续补齐 `.icns`、签名、构建与实机安装验证后再开放。

## 自动更新

当前只有 **GitHub Releases 版本检查**，没有模拟下载安装。

1.0 正式应用内更新仍需要完整的 Tauri signed updater 链，包括：

- updater artifact
- 签名私钥 / 公钥
- 更新签名
- endpoint / 静态更新元数据
- 安装与失败恢复验证

在这套链路完成前，版本检查不等于自动更新。

## 凭证与主机身份安全

### 桌面凭证

- `connections.json` 只保存非敏感连接配置。
- 密码和私钥 passphrase 统一由 Rust `keyring` 写入操作系统密钥环。
- Tab 快照和普通 localStorage 不持久化 `password` / `passphrase`。
- 历史 `xterminal-pwd_<id>` 明文 localStorage 数据会迁移到安全后端，然后删除明文键。
- 从 XTerminal Pro 改名后，旧 `xterminal-*` 普通存储、`xterminal-pro` 配置目录和 Keyring 服务会迁移到 Termlane 命名空间。
- 复制连接只复制非敏感配置，不复制凭证。

### 浏览器开发模式

浏览器没有操作系统 Keyring，因此只使用 AES-GCM fallback 支撑 UI / mock 开发。AES key 位于 sessionStorage，这一安全等级不能与桌面 Keyring 等同。

### SSH Host Key

- 已知且匹配：允许连接。
- 首次未知：返回算法 + SHA256 指纹，用户明确点击“信任并连接”后才写入 `~/.ssh/known_hosts`。
- 非 22 端口：使用 `check_port` 和 `[host]:port` 规则。
- 已知但不匹配：硬拒绝，不允许静默覆盖。

## Tauri 安全边界

- CSP 已启用。
- `withGlobalTauri` 已关闭；前端通过 `@tauri-apps/api` ESM 调用 Tauri。
- Tauri capabilities 按最小权限原则维护。

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
npm test
npm run test:smoke
npx vue-tsc --noEmit
npm run build

cd src-tauri
cargo check
cargo test
```

`.github/workflows/ci.yml` 用于代码质量门禁；`.github/workflows/build.yml` 提供手动 Windows MSI / Linux DEB 构建基线。真实安装通过与否仍以目标系统安装验证为准。

## 构建

```bash
npm run tauri:build
```

当前 Tauri bundle 配置只声明 1.0 收口范围内的：

- Windows MSI
- Linux DEB

也可以使用仓库中的 `build.bat` / `build.sh` 或手动触发 GitHub 构建工作流。

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
    ├── lib.rs           Tauri Builder / command 注册
    ├── commands.rs      IPC 参数验证与调用入口
    ├── ssh.rs           SSH / PTY
    ├── sftp.rs          SFTP / 本地文件操作
    ├── local_pty.rs     本地终端
    ├── config.rs        连接配置 / Keyring / 录制元数据
    └── updater.rs       GitHub Release 版本检查
```

## 1.0 收口路线

当前公开跟踪两个未完成方向：

1. [#1 UI / 交互产品化整理](https://github.com/z7ping/termlane/issues/1)
2. [#2 1.0 发布基线：核心链路、质量、安全与跨平台验证](https://github.com/z7ping/termlane/issues/2)

发布前停止新增非阻塞功能；源码完成、真实桌面验收、安装验证分开记录。

## 文档

- [更新日志](CHANGELOG.md)
- [贡献指南](CONTRIBUTING.md)

涉及当前能力、版本、依赖或安全结论时，请以源码、测试、Issue 和构建配置为准。

## License

MIT
