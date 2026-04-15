# XTerminal Pro - 架构设计文档

## 系统架构

```
┌─────────────────────────────────────────────────────────────┐
│                     Tauri Desktop App                        │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────────────────────────────────────────────────┐  │
│  │              Frontend (WebView)                       │  │
│  │                                                      │  │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐          │  │
│  │  │Sidebar   │  │TabBar    │  │StatusBar │          │  │
│  │  │连接管理   │  │标签页    │  │状态栏     │          │  │
│  │  └──────────┘  └──────────┘  └──────────┘          │  │
│  │                                                      │  │
│  │  ┌──────────────────────────────────────────────┐  │  │
│  │  │           Main Content Area                  │  │  │
│  │  │                                            │  │  │
│  │  │  ┌─────────────┐  ┌─────────────┐          │  │  │
│  │  │  │ TerminalPanel│  │ SftpPanel  │          │  │  │
│  │  │  │  xterm.js   │  │ 文件管理    │          │  │  │
│  │  │  └─────────────┘  └─────────────┘          │  │  │
│  │  │  ┌─────────────┐  ┌─────────────┐          │  │  │
│  │  │  │BatchCommand │  │Connection  │          │  │  │
│  │  │  │ 批量命令    │  │Monitor     │          │  │  │
│  │  │  └─────────────┘  └─────────────┘          │  │  │
│  │  └──────────────────────────────────────────────┘  │  │
│  │                                                      │  │
│  │  Framework: Vue 3 + Vite + TailwindCSS           │  │
│  └──────────────────────────────────────────────────────┘  │
│                          ↓ Tauri IPC                      │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────────────────────────────────────────────────┐  │
│  │              Rust Backend (Native)                    │  │
│  │                                                      │  │
│  │  ┌──────────────────────────────────────────────┐  │  │
│  │  │           Tauri Command Layer                │  │  │
│  │  │  (ssh_connect, sftp_list, save_config)    │  │  │
│  │  └──────────────────────────────────────────────┘  │  │
│  │                                                      │  │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐       │  │
│  │  │  SSH     │  │  SFTP    │  │ Config   │       │  │
│  │  │  Module  │  │  Module  │  │  Module  │       │  │
│  │  │          │  │          │  │          │       │  │
│  │  │ ssh2-rs │  │ sftp-rs  │  │ serde    │       │  │
│  │  │ tokio   │  │          │  │ keyring  │       │  │
│  │  └──────────┘  └──────────┘  └──────────┘       │  │
│  └──────────────────────────────────────────────────────┘  │
│                          ↓ Native APIs                    │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐                │
│  │ Network  │  │  File    │  │Keyring   │                │
│  │  (SSH)   │  │ System   │  │(OS密钥链) │                │
│  └──────────┘  └──────────┘  └──────────┘                │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## 前端架构

### 目录结构

```
src/
├── App.vue                 # 主应用入口
├── main.js                 # 应用初始化
├── utils/                  # 工具模块
│   ├── tauri.js           # Tauri IPC 封装
│   └── i18n.js            # 国际化
├── components/            # Vue 组件
│   ├── Sidebar.vue        # 侧边栏（连接列表）
│   ├── TabBar.vue         # 标签页
│   ├── TerminalPanel.vue  # 终端面板
│   ├── SftpPanel.vue      # SFTP 文件管理
│   ├── BatchCommand.vue   # 批量命令
│   ├── QuickCommands.vue  # 快捷命令
│   ├── PortForward.vue    # 端口转发
│   ├── ConnectionMonitor.vue # 连接监控
│   ├── SpeedTest.vue      # 速度测试
│   ├── SessionRecorder.vue # 会话录制
│   ├── Notes.vue          # 笔记
│   ├── Bookmarks.vue      # 书签
│   ├── ProxyConfig.vue    # 代理配置
│   ├── MacroRecorder.vue  # 宏录制
│   ├── ScheduledTasks.vue # 定时任务
│   ├── Settings.vue       # 设置面板
│   ├── ConnectionDialog.vue # 连接对话框
│   ├── Toast.vue          # Toast 通知
│   ├── StatusBar.vue      # 状态栏
│   ├── TitleBar.vue       # 标题栏
│   ├── ShortcutHelp.vue   # 快捷键帮助
│   ├── Onboarding.vue     # 新手引导
│   ├── ErrorBoundary.vue  # 错误边界
│   └── VirtualList.vue    # 虚拟滚动列表
└── styles/
    └── tailwind.css       # TailwindCSS 配置
```

### 组件层次

```
App.vue (Root)
├── TitleBar
├── Sidebar
├── TabBar
├── Main Content Area
│   ├── TerminalPanel
│   ├── SftpPanel
│   ├── BatchCommand
│   ├── ConnectionMonitor
│   ├── SpeedTest
│   ├── SessionRecorder
│   ├── Notes
│   ├── Bookmarks
│   ├── ProxyConfig
│   ├── QuickCommands
│   ├── PortForward
│   ├── ScheduledTasks
│   └── MacroRecorder
├── StatusBar
├── ConnectionDialog (Modal)
├── ShortcutHelp (Modal)
└── Onboarding (Modal)
```

### 状态管理

XTerminal Pro 使用 Vue 3 的响应式系统进行状态管理，不使用外部状态管理库（如 Pinia）。

**全局状态**（App.vue）：
- `connections`：所有连接配置
- `activeConnectionId`：当前选中连接 ID
- `tabs`：所有标签页
- `activeTabId`：当前标签页 ID
- `viewMode`：当前视图模式（terminal/sftp/batch/monitor/...）

**状态流**：
```
用户操作 → 更新响应式数据 → 组件重新渲染
      ↓
   invoke() → Rust Backend → Native API
      ↓
   listen() 事件监听 → 更新前端状态
```

### 事件通信

**父组件 → 子组件**（Props）：
```vue
<TerminalPanel
  :tab="tab"
  :active="tab.id === activeTabId"
  @connected="onSessionConnected"
  @disconnected="onSessionDisconnected"
/>
```

**子组件 → 父组件**（Emits）：
```javascript
emit('connected', { session_id: 'xxx' })
emit('disconnected', 'xxx')
```

**Rust Backend → 前端**（Tauri Events）：
```javascript
listen('shell_output', (event) => {
  console.log('Shell output:', event.payload)
})

listen('shell_disconnected', (event) => {
  console.log('Shell disconnected:', event.payload.session_id)
})
```

## 后端架构

### Rust 模块结构

```
src-tauri/src/
├── main.rs       # Tauri 应用入口、Command 注册
├── ssh.rs        # SSH 连接、PTY Shell 管理
├── sftp.rs       # SFTP 文件操作
├── config.rs     # 配置持久化、窗口状态
├── local_pty.rs  # 本地终端模拟
└── updater.rs    # 应用更新检查
```

### 模块职责

#### 1. main.rs
- Tauri 应用初始化
- 注册所有 Commands
- 窗口事件监听（关闭、调整大小）
- 窗口状态自动保存

#### 2. ssh.rs
- SSH 连接管理（密码/密钥/跳板机）
- PTY Shell 会话管理
- 连接监控（CPU/内存/磁盘/负载）
- 速度测试（TCP ping）
- 会话录制（PTY 输出监听）

**数据结构**：
```rust
// SSH 会话信息
pub struct SshSession {
    pub id: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub connected: bool,
}

// PTY Shell 会话
pub struct ShellSession {
    pub id: String,
    pub channel: Channel,
    pub app: AppHandle,
}

// 监控数据
pub struct MonitorData {
    pub cpu_usage: f64,
    pub memory_total: u64,
    pub memory_used: u64,
    pub memory_percent: f64,
    pub disk_total: u64,
    pub disk_used: u64,
    pub disk_percent: f64,
    pub load_1: f64,
    pub load_5: f64,
    pub load_15: f64,
    pub uptime_seconds: u64,
}
```

#### 3. sftp.rs
- SFTP 会话管理
- 文件列表（`ls`）
- 文件上传/下载
- 文件重命名、删除
- 目录创建、删除
- 文件内容读写（在线编辑）

**数据结构**：
```rust
pub struct FileInfo {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub is_dir: bool,
    pub modified: Option<String>,
    pub permissions: Option<String>,
}
```

#### 4. config.rs
- 连接配置存储（加密）
- 应用设置存储
- 窗口状态持久化
- 文件路径解析

**数据结构**：
```rust
pub struct Connection {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub auth_type: AuthType,
    pub group: String,
    pub tags: Vec<String>,
    pub jump_host: Option<JumpHost>,
}

pub enum AuthType {
    Password,
    Key { path: String },
    KeyWithPassphrase { path: String },
}

pub struct WindowState {
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub maximized: Option<bool>,
    pub display_id: Option<String>,
}
```

#### 5. local_pty.rs
- 本地 shell 启动
- PTY 会话管理
- 输入/输出转发

#### 6. updater.rs
- 检查最新版本
- 下载更新
- 应用更新

### 线程模型

**Tokio 异步运行时**：
```rust
#[tokio::main]
async fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            // ... commands
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**SSH 连接线程**：
- 每个 SSH 连接使用 Tokio 异步任务
- PTY Shell 使用独立线程监听输出
- 通过 Channel 进行线程间通信

### 并发安全

**Mutex 保护**：
```rust
use std::sync::Mutex;

static SESSIONS: std::sync::LazyLock<Mutex<HashMap<String, Session>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

// 使用宏简化 Mutex 锁操作
macro_rules! lock {
    ($mutex:expr) => {
        $mutex.lock().unwrap_or_else(|e| e.into_inner())
    };
}
```

## 数据流

### SSH 连接建立流程

```
Frontend                    Rust Backend                Native
    │                          │                           │
    ├─ invoke('ssh_start_shell')                          │
    │──────────────────────────►│                           │
    │                          ├─ create SSH Session      │
    │                          ├─ allocate PTY             │
    │                          ├─ start shell              │
    │                          ├─ spawn output thread      │
    │                          │                           │
    │◄──────────────────────────┤ (session_id)              │
    │                          │                           │
    │                         listen('shell_output') ◄──────┤
    │◄─────────────────────────┤ (data)                    │
    │  terminal.write(data)    │                           │
```

### SFTP 文件列表流程

```
Frontend                    Rust Backend                SSH/SFTP
    │                          │                           │
    ├─ invoke('sftp_list', path)                         │
    │──────────────────────────►│                           │
    │                          ├─ get SSH session          │
    │                          ├─ create SFTP session      │
    │                          ├─ sftp.readdir()          │
    │                          │──────────────────────────►│
    │                          │◄──────────────────────────┤
    │                          ├─ parse FileInfo            │
    │                          │                           │
    │◄──────────────────────────┤ (Vec<FileInfo>)          │
    │  render file list        │                           │
```

### 终端输入输出流程

```
Input Flow:
Terminal UI                Frontend                 Rust Backend          SSH
    │                         │                        │                │
    ├─ user types 'ls'        │                        │                │
    │────────────────────────►│                        │                │
    │                         ├─ invoke('ssh_shell_input')            │
    │                         │───────────────────────►│                │
    │                         │                        ├─ write to PTY   │
    │                         │                        │───────────────►│

Output Flow:
Terminal UI                Frontend                 Rust Backend          SSH
    │                         │                        │                │
    │                         │                        │◄───────────────│
    │                         │                        ├─ read from PTY  │
    │◄────────────────────────┤ (event: shell_output)  │                │
    │  terminal.write(data)   │                        │                │
```

## 安全架构

### 密码存储

```
用户输入密码
    │
    ├─ invoke('ssh_connect', password)
    │
    ├─ Rust Backend 接收
    │
    ├─ 使用密码建立 SSH 连接
    │
    ├─ 连接成功后：
    │   ├─ 使用系统密钥链存储加密密码
    │   ├─ Linux: secret-service (libsecret)
    │   ├─ macOS: Keychain
    │   └─ Windows: Credential Manager
    │
    └─ 内存中清零明文密码
```

### 配置文件加密

```
配置保存流程:
Connection Data
    │
    ├─ 序列化 (serde_json)
    │
    ├─ AES-256-GCM 加密
    │   ├─ 随机生成 IV
    │   ├─ 派生密钥 (PBKDF2)
    │   └─ 加密数据
    │
    ├─ 写入配置文件 (~/.config/xterminal-pro/connections.json)
    │
    └─ 文件权限设置为 600
```

### CSP 安全策略

```html
<!-- index.html -->
<meta
  http-equiv="Content-Security-Policy"
  content="default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; connect-src 'self' https: ws: wss:;"
/>
```

### XSS 防护

- ✅ 避免 `innerHTML`，使用 `textContent`
- ✅ 使用 Vue 模板自动转义
- ✅ CSP 禁止内联脚本（`script-src 'self'`）
- ✅ 配置文件不执行任何代码

## 性能优化

### 1. 懒加载

```javascript
// App.vue
const SftpPanel = defineAsyncComponent(() =>
  import('./components/SftpPanel.vue')
)
const BatchCommand = defineAsyncComponent(() =>
  import('./components/BatchCommand.vue')
)
// ... 其他组件
```

### 2. 虚拟滚动

```javascript
// VirtualList.vue
// 仅渲染可见区域的文件项
const visibleItems = computed(() => {
  const start = scrollTop.value / itemHeight.value
  const end = start + visibleCount.value
  return allItems.value.slice(start, end)
})
```

### 3. 连接池

```rust
// ssh.rs
// 复用 SSH Session，避免频繁建立连接
static SESSIONS: LazyLock<Mutex<HashMap<String, Session>>> = ...;
```

### 4. 异步文件传输

```rust
// sftp.rs
// 使用 Tokio 异步传输，避免阻塞主线程
async fn sftp_upload(session_id: String, local_path: String, remote_path: String) -> Result<String, String> {
    let sftp = get_sftp_session(&session_id).await?;
    sftp.send_file(local_path, remote_path).await?;
    Ok("Upload complete".to_string())
}
```

### 5. 批量操作并行

```javascript
// 前端批量上传
const uploadPromises = selectedFiles.map(file =>
  invoke('sftp_upload', {
    session_id: this.sessionId,
    local_path: file.path,
    remote_path: this.remotePath + '/' + file.name
  })
)

await Promise.all(uploadPromises)
```

## 扩展点

### 添加新的 Tauri Command

1. 在对应的 Rust 模块中实现函数：
```rust
// ssh.rs
#[tauri::command]
async fn ssh_custom_command(param: String) -> Result<String, String> {
    // 实现
    Ok("result".to_string())
}
```

2. 在 `main.rs` 中注册：
```rust
.invoke_handler(tauri::generate_handler![
    // ... 其他命令
    ssh_custom_command,
])
```

3. 在前端调用：
```javascript
const result = await invoke('ssh_custom_command', { param: 'value' })
```

### 添加新的前端事件监听

1. 在 Rust 中发送事件：
```rust
// ssh.rs
app.emit("custom_event", serde_json::json!({
    "data": "value"
}))?;
```

2. 在前端监听：
```javascript
listen('custom_event', (event) => {
  console.log('Event received:', event.payload)
})
```

### 添加新的视图模式

1. 创建组件：
```vue
<!-- src/components/NewView.vue -->
<template>
  <div class="h-full">新视图内容</div>
</template>
```

2. 在 App.vue 中注册：
```javascript
const NewView = defineAsyncComponent(() =>
  import('./components/NewView.vue')
)
```

3. 添加到视图模式列表：
```javascript
const viewModes = [
  { label: '终端', value: 'terminal' },
  // ...
  { label: '新视图', value: 'new' }
]
```

4. 在模板中渲染：
```vue
<NewView v-if="viewMode === 'new'" />
```

## 技术债务与优化方向

### 短期（P0）
- [ ] 添加 TypeScript 支持（迁移 `main.js` → `main.ts`）
- [ ] 增加前端单元测试覆盖率到 50%
- [ ] 拆分大型组件（Onboarding.vue 628 行）
- [ ] 统一错误处理机制

### 中期（P1）
- [ ] 使用 Pinia 替代响应式全局状态
- [ ] 添加 WebSocket 支持（实时日志流）
- [ ] 实现 SSH 隧道（端口转发增强）
- [ ] 添加性能监控面板

### 长期（P2）
- [ ] 插件系统（支持第三方扩展）
- [ ] 主题引擎（自定义主题）
- [ ] 多语言框架扩展（i18n 增强）
- [ ] 离线模式支持（PWA）

## 参考资料

- [Tauri 官方文档](https://tauri.app/)
- [Vue 3 文档](https://vuejs.org/)
- [TailwindCSS 文档](https://tailwindcss.com/)
- [ssh2-rs 文档](https://docs.rs/ssh2/)
- [xterm.js 文档](https://xtermjs.org/)
