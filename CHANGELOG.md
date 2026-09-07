# 更新日志

Termlane 使用语义化版本。这里只记录能够由当前源码、测试或发布产物验证的变化；历史开发阶段的功能清单不视为正式发布记录。

## [Unreleased]

### 变更

- 项目从 XTerminal Pro 更名为 Termlane，并迁移到 `z7ping/termlane`。
- 1.0 产品入口收敛为终端、文件和已经接通真实链路的工具。
- SSH 主机身份校验使用 OpenSSH `known_hosts` 语义；首次连接展示 SHA256 指纹，密钥不匹配时硬拒绝。
- 远程文件管理从 shell 命令模拟层迁移到 `ssh2::Sftp` subsystem，并复用当前已认证 SSH Session。
- 桌面密码和私钥口令统一由 OS Keyring 保存；普通配置和标签快照不持久化凭证。
- 会话录制保存为 asciicast `.cast` 文件，并支持 SSH / 本地 PTY 输出。
- 构建脚本和公开 GitHub Actions 不再依赖私有 Gitea、内网 Runner 或本机固定路径。
- 更新检查切换到公开 GitHub Releases；下载安装仍等待正式的 Tauri 签名更新链。

### 移除

- 删除历史假 ProxyJump 实现；1.0 当前不宣称支持 ProxyJump。
- 删除只有第二个 xterm、没有独立 PTY / SSH Session 的假分屏实现。
- 从 1.0 产品入口移除尚未形成真实用户链路的批量执行、监控、代理、端口转发和定时任务。
- 删除相关演示页面、浏览器假能力、重复工具函数、旧存储迁移脚本和生成物。

### 修复

- 修复终端 PTY Session 与 SFTP 执行通道割裂的问题。
- 修复录制组件与浏览器 mock 的命令名不一致。
- 修复旧品牌 localStorage、配置目录和 Keyring 命名空间迁移。
- 修复 README / Issue / 构建元数据与当前 1.0 能力边界不一致的问题。

## 发布说明

当前源码版本为 `0.1.0`，项目仍处于 1.0 收口阶段。正式 `v1.0.0` 发布前的剩余门禁以 GitHub Issues #1 和 #2 为准。
