# 贡献指南

Termlane 当前处于 1.0 收口阶段。优先修复核心 SSH / Terminal / SFTP 链路、质量问题和发布阻塞，不建议在 1.0 前扩展新的低频功能。

## 开发环境

建议使用：

- Node.js 22
- npm 10+
- Rust stable
- Tauri v2 所需系统依赖

Ubuntu / Debian 可安装：

```bash
sudo apt-get update
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev \
  libappindicator3-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  libssl-dev \
  libdbus-1-dev \
  patchelf \
  pkg-config
```

Windows 需要 Rust 的 MSVC 工具链及 Visual Studio C++ Build Tools。macOS 需要 Xcode Command Line Tools。

## 获取代码

```bash
git clone https://github.com/z7ping/termlane.git
cd termlane
npm ci
```

浏览器模式只用于 UI / mock：

```bash
npm run dev
```

真实 SSH、PTY、SFTP 和 Keyring 行为必须在桌面模式验证：

```bash
npm run tauri:dev
```

## 提交前检查

```bash
npm test
npm run test:smoke
npx vue-tsc --noEmit
npm run build

cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
```

涉及 Rust 代码时还建议执行：

```bash
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
```

不要用关闭检查、降低类型约束或扩大 `any` 的方式绕过真实失败。

## 分支与 Pull Request

1. 从最新 `main` 创建短生命周期分支，例如 `fix/...`、`feat/...`、`refactor/...`。
2. 一个 PR 聚焦一个明确问题，避免同时维护新旧两套实现。
3. PR 直接以 `main` 为目标分支。
4. 使用 Conventional Commits 风格的提交信息。
5. 能复现的 Bug 应补测试；不能自动化的桌面行为请写明实际验证步骤和环境。

## 1.0 边界

当前不应把以下能力描述为 1.0 已支持，除非已经重新实现并完成真实桌面验收：

- ProxyJump
- 多 Pane 分屏
- 端口转发
- 批量执行
- 主机监控
- 定时任务

浏览器 mock、组件存在或后端命令存在都不等于产品能力完成。

## 安全要求

- 不要在 Issue、日志、测试或提交中放入真实密码、Token、私钥、服务器地址或个人环境路径。
- 桌面凭证只能进入 OS Keyring；普通配置、localStorage 和 Tab 快照不得保存 `password` / `passphrase`。
- 不要弱化 SSH Host Key mismatch 的硬拒绝逻辑。
- 安全、协议和平台基础能力优先使用成熟实现，不自造半套协议。

## 报告问题

直接创建 GitHub Issue，并提供：

- Termlane 版本或 commit
- 操作系统
- 可复现步骤
- 预期行为与实际行为
- 必要的错误日志（先移除凭证、IP、用户名等敏感内容）

当前 1.0 收口进度以 [#1 UI / 交互产品化整理](https://github.com/z7ping/termlane/issues/1) 和 [#2 1.0 发布基线](https://github.com/z7ping/termlane/issues/2) 为准。
