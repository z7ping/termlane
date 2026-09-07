# CLAUDE.md

本文件为在 Termlane 仓库内工作的编码 Agent 提供约束。

## 项目定位

Termlane 是基于 Tauri v2、Vue 3、xterm 和 Rust 的轻量桌面终端，核心是 SSH、Terminal、SFTP 与多服务器管理。当前源码版本为 `0.1.0`，处于 1.0 收口阶段。

优先级：核心链路真实性 > 稳定性 / 安全 > 交互完整性 > 低频功能数量。

## 常用命令

```bash
npm ci
npm run dev            # 浏览器 UI / mock
npm run tauri:dev      # 真实桌面运行
npm test
npm run test:smoke
npx vue-tsc --noEmit
npm run build
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
```

## 架构边界

```text
Vue 3 WebView
  -> @tauri-apps/api ESM invoke/listen
  -> Tauri command boundary
  -> Rust ssh2 / portable-pty / keyring / filesystem
```

- `src/utils/tauri.ts` 是前端 Tauri IPC / browser mock 边界。
- 浏览器模式只能证明 UI 和前端逻辑，不能证明 SSH、PTY、SFTP、Keyring 或安装包可用。
- 不重新引入 `window.__TAURI__`；`withGlobalTauri` 保持关闭。
- `src-tauri/src/ssh.rs` 负责 SSH / PTY / Host Key。
- `src-tauri/src/sftp.rs` 使用 `ssh2::Sftp`；不要重新退回 shell + base64 文件协议。
- `src-tauri/src/config.rs` 保存非敏感配置和 OS Keyring 凭证。

## 凭证与 Host Key

桌面 secret 的唯一权威存储是 OS Keyring。

- 不得把 `password` / `passphrase` 写入 `connections.json`、Tab、localStorage、日志或普通状态快照。
- 历史 `xterminal-pwd_<id>` 只作为迁移输入，由 `credentials.ts` 迁入安全后端后删除。
- Browser AES-GCM fallback 仅服务浏览器 mock，不得描述为桌面等价安全。
- SSH 首次未知主机必须展示算法和 SHA256 指纹并要求用户确认。
- known host mismatch 必须硬拒绝，不提供静默覆盖。

## 1.0 明确不宣称的能力

在真实实现与桌面验收前，不恢复以下入口或宣传：

- ProxyJump
- 多 Pane 分屏
- 端口转发
- 批量执行
- 主机监控
- 定时任务

后端命令、旧页面或 browser mock 单独存在都不算产品能力。

## 依赖与实现原则

- 协议、安全、窗口状态、终端模拟和更新链优先复用官方或成熟实现。
- 不为了“统一风格”引入大型状态库或工具库。
- 替换基础实现时删除旧路径，不保留补丁式平行实现。
- 删除未使用依赖时必须同步更新 lockfile。
- 性能数字只能使用实测结果。

## 发布主线

当前只跟踪两个未完成主线：

- [#1 UI / 交互产品化整理](https://github.com/z7ping/termlane/issues/1)
- [#2 1.0 发布基线：核心链路、质量、安全与跨平台验证](https://github.com/z7ping/termlane/issues/2)

完成项必须有源码、测试、真实桌面验证或安装验证支撑。发布前停止新增非阻塞功能。
