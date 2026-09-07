# Termlane — 1.0 收口计划

> 本文件已从早期“14 天功能开发计划”调整为当前 1.0 收口计划。
>
> 历史阶段中的勾选曾主要表示“代码/页面已写”，并不能证明真实运行链路已经达到发布质量。当前统一以源码、真实 Tauri 运行、测试和构建结果验收。

## 产品定位

一个更轻、更快、更适合开发者日常多服务器工作的桌面终端：

- 低资源占用
- 启动快
- 操作直接
- 少配置
- 不过度膨胀

原则：核心能力做深，低频能力降噪；通用基础设施优先采用成熟实现，但不为了“去自研”反而增加不必要依赖。

---

## 当前阶段

项目已完成主要 UI / 功能骨架，当前进入 **1.0 发布前收口**。

主路线由以下 Issue 跟踪：

1. [#1 去自研化与基础能力整合](https://github.com/z7ping/xterminal-pro/issues/1)
2. [#2 UI / 交互产品化整理](https://github.com/z7ping/xterminal-pro/issues/2)
3. [#3 1.0 发布基线：核心链路、质量、安全与跨平台验证](https://github.com/z7ping/xterminal-pro/issues/3)

1.0 前原则上停止新增非阻塞功能。

---

## 阶段 A：基础能力收口

- [x] 窗口状态改用 Tauri 官方 `window-state` 插件
- [x] 删除旧 WindowState IPC / DTO / JSON 持久化双轨
- [x] 修复失效 `test:smoke`
- [x] 更新检查改用语义化版本比较
- [x] 删除假的“下载/安装进度”
- [x] 评估 VirtualList：当前小而稳定，保留本地实现
- [x] 评估 Split Resize：当前需求简单，保留本地实现
- [x] 评估 xterm addons：无真实性能/兼容问题前不额外引入
- [x] 评估 VueUse：当前不引入
- [x] Host Key 非 22 端口使用 `check_port` + `[host]:port`
- [x] Host Key 首次未知主机改为算法 + SHA256 指纹显式确认
- [x] Host Key mismatch 保持硬拒绝，不允许静默覆盖
- [x] 删除假 ProxyJump 后端 command 与连接 UI，不再把它作为 1.0 已完成能力
- [x] 清理 Tab / localStorage 中的明文 password / passphrase 持久化
- [x] 历史 `xterminal-pwd_*` 明文凭证迁移到安全后端后删除
- [x] 桌面凭证边界统一为 OS Keyring；浏览器仅保留 AES-GCM 开发 fallback
- [x] 关闭 `withGlobalTauri`，改用官方 `@tauri-apps/api` ESM
- [x] 建立 GitHub Actions 前端 + Rust 质量门禁
- [ ] 删除确认无使用的遗留依赖 / 注释并同步 lockfile
- [ ] 当前 CI 全部通过

### 转移到发布基线（#3）

以下工作依赖正式发布流水线或真实桌面/安装环境，不在 #1 里做半成品：

- [ ] Tauri 正式签名 updater（artifact、签名、公钥、endpoint、安装/回滚）
- [ ] npm / Rust 依赖安全公告逐项清零或形成明确接受记录
- [ ] 真实桌面 SSH / SFTP / PTY / 安装包验证

### ProxyJump 决策

历史实现并没有通过跳板通道建立目标 SSH Session，因此已经从当前分支删除，不再保留“看似可用”的入口。

1.0 当前决策：**不宣称支持 ProxyJump。**

后续若重新实现，需要选择真正支持 session-over-channel 的 SSH 技术路线，并单独完成安全、兼容性和资源生命周期设计；不为一个功能同时维护两套半成熟 SSH 栈。

---

## 阶段 B：UI / 交互产品化

目标不是大改布局，而是减少主界面噪音并统一交互语言。

- [ ] 一级入口只保留高频核心能力
- [ ] 低频增强能力收进二级入口
- [ ] 统一图标体系
- [ ] 统一按钮 / 输入框 / 菜单 / Dialog / Toast / 空状态
- [ ] Settings 接入正式入口
- [ ] 清理无效 `hover:` inline style 和重复视觉规则
- [ ] 小窗口 / 高分屏 / 长连接名 / 多标签边界验证
- [ ] loading / disabled / error / empty / disconnected 状态统一
- [ ] 终端区域保持一级视觉权重

详见 #2。

---

## 阶段 C：1.0 发布基线

### SSH / Terminal

- [ ] 密码认证真实验证
- [ ] 密钥认证真实验证
- [ ] 首次 Host Key 指纹确认真实桌面验证
- [ ] Host Key mismatch / 非 22 端口真实验证
- [x] 1.0 能力边界不再包含假 ProxyJump
- [ ] PTY 创建 / 输入 / resize / 关闭 / 异常退出验证
- [ ] 自动重连验证
- [ ] 多标签与资源释放验证
- [ ] 长时间空闲资源占用验证

### SFTP

- [ ] 本地 / 远程目录浏览验证
- [ ] 上传 / 下载验证
- [ ] 批量传输与错误恢复验证
- [ ] 重命名 / 删除 / mkdir / chmod 验证
- [ ] 在线编辑读写验证
- [ ] 大目录性能验证

### 测试

- [x] GitHub Actions CI 已建立
- [ ] `npm test`
- [ ] `npm run test:smoke`
- [ ] `npx vue-tsc --noEmit`
- [ ] `npm run build`
- [ ] `cargo check`
- [ ] `cargo test`
- [ ] 建立关键桌面流程集成 / E2E 验证

> CI 中某命令只有最新目标分支 run 通过后才在这里标记完成；不按历史成功片段提前勾选。

### 性能

以下目标必须以真实数据决定是否写入产品宣传，不沿用历史未验证数字：

- [ ] 冷启动耗时
- [ ] 空闲内存
- [ ] 1 / 5 / 10 个 Session 内存
- [ ] SSH 连接耗时
- [ ] 大量终端输出渲染性能
- [ ] SFTP 大目录性能

### 发布

- [ ] Windows MSI 构建 + 安装验证
- [ ] Linux DEB 构建 + 安装验证
- [ ] 明确 macOS 1.0 支持策略
- [ ] Tauri 签名 updater 链路
- [ ] README / PLAN / SPEC / API 与源码一致
- [ ] 发布说明 / 已知限制 / 截图
- [ ] 版本统一升级到 `1.0.0`
- [ ] 正式 Release

---

## 1.0 非目标

除非它阻塞核心体验，否则以下内容不应拖住 1.0：

- 为了框架一致性重写现有稳定小组件
- 为了“更先进”迁移整个前端状态管理
- 无性能证据就引入 WebGL renderer
- 无真实字符宽度问题就增加 Unicode addon
- 新增更多一级功能入口
- 扩展 AI、插件市场或其他大型功能面
- 在没有明确技术路线时恢复 ProxyJump 入口

---

## 完成定义

某项功能只有同时满足以下条件，才能从“有实现”升级为“已完成”：

1. 用户入口存在且交互完整；
2. 前端状态和错误路径明确；
3. Tauri IPC / Rust / 原生能力真实接通；
4. 不依赖 mock / placeholder / 模拟进度；
5. 有自动测试或明确的真实运行验证记录；
6. 安全和资源生命周期达到对应能力要求；
7. 文档描述与实际能力一致。

这一定义优先于历史文档中的任何勾选。
