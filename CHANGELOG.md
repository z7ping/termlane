# XTerminal Pro - 更新日志

所有重要变更都将记录在此文件中。

格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/)，
并且本项目遵循 [语义化版本](https://semver.org/lang/zh-CN/) 规范。

## [Unreleased]

### 新增
- TBA

### 修复
- TBA

### 变更
- TBA

## [2.0.0] - 2026-04-16

### 新增
- ✅ 完整的 SSH 终端功能（33 项核心功能）
- ✅ PTY Shell 持久会话（支持 vim/top/htop）
- ✅ SFTP 文件管理（双栏，支持上传/下载/编辑）
- ✅ 连接健康监控（CPU/内存/磁盘/负载实时数据）
- ✅ 连接速度测试（TCP ping + 延迟柱状图）
- ✅ 会话录制（asciinema v2 格式，支持回放）
- ✅ 凭证安全存储（OS 密钥链/keyring）
- ✅ 多级分组管理（"/" 分隔符嵌套）
- ✅ 标签颜色系统（红=生产/黄=测试/绿=开发/蓝=一般/紫=特殊）
- ✅ 批量命令执行（多服务器同时执行）
- ✅ 端口转发（本地/远程/SOCKS5）
- ✅ 快捷命令管理（CRUD + 一键执行）
- ✅ 代理配置（HTTP/SOCKS5，认证+绕过列表）
- ✅ 书签功能（远程目录快速跳转）
- ✅ 笔记模块（Markdown，按连接存储）
- ✅ 定时任务（cron 表达式）
- ✅ 宏录制功能
- ✅ 本地终端支持（无需 SSH）
- ✅ 多标签页管理
- ✅ Split Pane 分屏
- ✅ 终端搜索功能（Ctrl+Shift+F）
- ✅ 错误边界保护
- ✅ 窗口状态持久化
- ✅ 多语言支持（中/英）
- ✅ 应用更新检查
- ✅ 新手引导
- ✅ 快捷键帮助
- ✅ Toast 通知系统
- ✅ 虚拟滚动优化（大目录列表）
- ✅ 浏览器模拟模式（无 Tauri 也能用 UI）

### 安全修复
- ✅ 修复 Vite 高危漏洞：
  - 路径遍历漏洞 (GHSA-4w7w-66w2-5vf9)
  - 任意文件读取漏洞 (GHSA-p9ff-h696-f583)
- ✅ 迁移 xterm 到 @xterm/xterm（修复依赖过时问题）
- ✅ 添加 CSP 安全策略，防止 XSS
- ✅ localStorage key 冲突修复
- ✅ 连接配置加密存储迁移

### 性能优化
- ✅ 懒加载 18 个组件，减少首屏加载时间
- ✅ 24 个异步 chunk，按需加载
- ✅ 首屏仅 119KB gzip
- ✅ 安装包优化至 4.9MB
- ✅ Linux .deb 构建成功

### UI/UX 改进
- ✅ 修复弹窗点击背景关闭
- ✅ 修复字体大小配置
- ✅ 修复 FOUC（Flash of Unstyled Content）
- ✅ 修复点击文件报错
- ✅ 暗色/亮色主题切换
- ✅ 12 个视图模式切换

### 文档完善
- ✅ 添加 README.md 详细说明
- ✅ 添加 API 文档
- ✅ 添加架构设计文档
- ✅ 添加测试指南
- ✅ 添加环境要求/系统依赖说明
- ✅ 添加配置/故障排除指南

## [1.5.0] - 2026-04-15

### 新增
- ✅ 基础 SSH 终端功能
- ✅ 密码/密钥认证
- ✅ 基础 SFTP 功能
- ✅ 连接列表管理
- ✅ 多标签页支持
- ✅ xterm.js 集成

## [1.0.0] - 2026-03-30

### 新增
- ✅ 项目初始化
- ✅ Tauri + Vue 3 基础架构
- ✅ 基础 UI 布局
- ✅ 暗色/亮色主题
- ✅ 配置存储模块

## 历史里程碑

### 2026-04-15 - 安全漏洞修复
- 发现并修复 Vite 高危漏洞
- 升级 xterm 依赖
- 完成 18 个 Rust 测试用例
- 完善 CI/CD 流程

### 2026-04-14 - 功能完善
- 添加会话录制功能
- 实现 SFTP 批量操作
- 添加连接健康监控
- 完成端口转发功能

### 2026-04-13 - 架构优化
- 优化 PTY Shell 实现
- 添加错误边界保护
- 实现窗口状态持久化
- 完善错误处理机制

### 2026-04-12 - 核心功能
- SSH 连接模块完成
- PTY Shell 持久会话
- SFTP 文件管理系统
- 本地终端支持

### 2026-04-11 - 项目基础
- Tauri 框架搭建
- Vue 3 前端架构
- TailwindCSS 样式系统
- 基础组件库

## 版本说明

### 版本号规则
- **主版本号**：不兼容的 API 修改
- **次版本号**：向下兼容的功能性新增
- **修订号**：向下兼容的问题修正

### 发布周期
- **主版本**：重大架构变更（不定期）
- **次版本**：功能更新（每周）
- **修订版**：bug 修复（随时）

### 预发布版本
- **Alpha**：内部测试版
- **Beta**：公开测试版
- **RC**：候选发布版

## 反馈与贡献

- 🐛 报告 Bug：[Issues](https://github.com/7ping/XTerminal-Pro/issues)
- 💡 功能建议：[Discussions](https://github.com/7ping/XTerminal-Pro/discussions)
- 📝 文档改进：[Wiki](https://github.com/7ping/XTerminal-Pro/wiki)
- 🔧 贡献代码：[CONTRIBUTING.md](CONTRIBUTING.md)

## 致谢

感谢所有为 XTerminal Pro 做出贡献的开发者，以及以下开源项目：
- [Tauri](https://tauri.app/) - 跨平台桌面应用框架
- [Vue.js](https://vuejs.org/) - 渐进式 JavaScript 框架
- [xterm.js](https://xtermjs.org/) - 前端终端模拟器
- [ssh2-rs](https://docs.rs/ssh2/) - Rust SSH 客户端库
- [Tailwindcss](https://tailwindcss.com/) - 实用优先的 CSS 框架