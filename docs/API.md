# Termlane - API 文档

本文档描述 Termlane 的 Rust 后端 API 和前端 IPC 接口。

## 架构概览

```
Frontend (Vue 3)
    ↓ Tauri IPC
Rust Backend (src-tauri/src/)
    ↓ Native
System Resources (SSH/SFTP/Keyring)
```

## Rust 后端 API

所有 API 通过 Tauri Commands 暴露给前端，使用 `invoke()` 调用。

### 模块列表

| 模块 | 文件 | 职责 |
|------|------|------|
| SSH | `ssh.rs` | SSH 连接管理、PTY Shell |
| SFTP | `sftp.rs` | 文件传输、目录浏览 |
| Config | `config.rs` | 配置持久化、窗口状态 |
| Local PTY | `local_pty.rs` | 本地终端模拟 |
| Updater | `updater.rs` | 应用更新检查 |

---

## SSH 模块 (`ssh.rs`)

### 连接管理

#### `ssh_connect`
**用途**：建立密码认证的 SSH 连接（遗留接口）

**签名**：
```rust
async fn ssh_connect(
    host: String,
    port: u16,
    username: String,
    password: String
) -> Result<String, String>
```

**返回**：
- `Ok(String)`：session ID
- `Err(String)`：错误信息

**示例**：
```javascript
const sessionId = await invoke('ssh_connect', {
  host: '192.168.1.100',
  port: 22,
  username: 'root',
  password: 'password'
})
```

#### `ssh_connect_key`
**用途**：建立密钥认证的 SSH 连接

**签名**：
```rust
async fn ssh_connect_key(
    host: String,
    port: u16,
    username: String,
    key_path: String,
    passphrase: String
) -> Result<String, String>
```

#### `ssh_connect_jump`
**用途**：通过跳板机建立连接

**签名**：
```rust
async fn ssh_connect_jump(
    jump_host: String,
    jump_port: u16,
    jump_user: String,
    jump_pass: String,
    target_host: String,
    target_port: u16,
    target_user: String,
    target_pass: String
) -> Result<String, String>
```

### PTY Shell（持久会话）

#### `ssh_start_shell`
**用途**：启动 PTY Shell（支持 vim/top/htop）

**签名**：
```rust
fn ssh_start_shell(
    app: tauri::AppHandle,
    host: String,
    port: u16,
    username: String,
    password: String,
    key_path: Option<String>,
    passphrase: Option<String>,
    cols: Option<u16>,
    rows: Option<u16>
) -> Result<String, String>
```

**`app` 用途**：用于向前端发送事件（`shell_output`、`shell_disconnected`）

**前端事件监听**：
```javascript
// 监听 Shell 输出
listen('shell_output', (event) => {
  const { session_id, data } = event.payload
  terminal.write(data)
})

// 监听断开连接
listen('shell_disconnected', (event) => {
  console.log('Shell disconnected:', event.payload.session_id)
})
```

#### `ssh_shell_input`
**用途**：向 Shell 发送输入

**签名**：
```rust
fn ssh_shell_input(session_id: String, data: String) -> Result<(), String>
```

**示例**：
```javascript
await invoke('ssh_shell_input', {
  session_id: 'xxx',
  data: 'ls -la\n'
})
```

#### `ssh_shell_resize`
**用途**：调整终端大小（响应窗口调整）

**签名**：
```rust
fn ssh_shell_resize(session_id: String, cols: u16, rows: u16) -> Result<(), String>
```

#### `ssh_disconnect`
**用途**：断开 SSH 连接

**签名**：
```rust
fn ssh_disconnect(session_id: String) -> Result<(), String>
```

#### `ssh_list_sessions`
**用途**：获取所有活跃会话列表

**签名**：
```rust
fn ssh_list_sessions() -> Vec<SshSession>
```

**返回结构**：
```rust
pub struct SshSession {
    pub id: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub connected: bool,
}
```

### 监控与测试

#### `ssh_monitor`
**用途**：获取服务器资源监控数据

**签名**：
```rust
async fn ssh_monitor(session_id: String) -> Result<MonitorData, String>
```

**返回结构**：
```rust
pub struct MonitorData {
    pub cpu_usage: f64,         // CPU 使用率 (0-100)
    pub memory_total: u64,       // 内存总量 (字节)
    pub memory_used: u64,       // 内存使用 (字节)
    pub memory_percent: f64,    // 内存使用率 (0-100)
    pub disk_total: u64,        // 磁盘总量 (字节)
    pub disk_used: u64,         // 磁盘使用 (字节)
    pub disk_percent: f64,      // 磁盘使用率 (0-100)
    pub load_1: f64,            // 1分钟负载
    pub load_5: f64,            // 5分钟负载
    pub load_15: f64,           // 15分钟负载
    pub uptime_seconds: u64,   // 运行时间 (秒)
}
```

#### `ssh_ping`
**用途**：测试连接延迟

**签名**：
```rust
async fn ssh_ping(host: String, port: u16, count: Option<u32>) -> Result<Vec<PingResult>, String>
```

---

## SFTP 模块 (`sftp.rs`)

### `sftp_list`
**用途**：列出远程目录内容

**签名**：
```rust
async fn sftp_list(session_id: String, path: String) -> Result<Vec<FileInfo>, String>
```

**返回结构**：
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

### `sftp_download`
**用途**：下载远程文件

**签名**：
```rust
async fn sftp_download(
    session_id: String,
    remote_path: String,
    local_path: String
) -> Result<String, String>
```

### `sftp_upload`
**用途**：上传本地文件

**签名**：
```rust
async fn sftp_upload(
    session_id: String,
    local_path: String,
    remote_path: String
) -> Result<String, String>
```

### `sftp_delete`
**用途**：删除远程文件或目录

**签名**：
```rust
async fn sftp_delete(session_id: String, path: String) -> Result<(), String>
```

### `sftp_mkdir`
**用途**：创建远程目录

**签名**：
```rust
async fn sftp_mkdir(session_id: String, path: String) -> Result<(), String>
```

### `sftp_read_file`
**用途**：读取远程文件内容（用于在线编辑）

**签名**：
```rust
async fn sftp_read_file(session_id: String, path: String) -> Result<String, String>
```

### `sftp_write_file`
**用途**：写入远程文件（保存编辑）

**签名**：
```rust
async fn sftp_write_file(
    session_id: String,
    path: String,
    content: String
) -> Result<(), String>
```

### `sftp_rename`
**用途**：重命名远程文件或目录

**签名**：
```rust
async fn sftp_rename(
    session_id: String,
    old_path: String,
    new_path: String
) -> Result<(), String>
```

---

## Config 模块 (`config.rs`)

### `save_connections`
**用途**：保存所有连接配置（加密存储）

**签名**：
```rust
fn save_connections(connections: Vec<Connection>) -> Result<(), String>
```

### `load_connections`
**用途**：加载所有连接配置

**签名**：
```rust
fn load_connections() -> Result<Vec<Connection>, String>
```

### `save_settings`
**用途**：保存应用设置

**签名**：
```rust
fn save_settings(settings: Settings) -> Result<(), String>
```

### `load_settings`
**用途**：加载应用设置

**签名**：
```rust
fn load_settings() -> Result<Settings, String>
```

### 窗口状态管理

#### `save_window_state`
**用途**：保存窗口位置、大小、最大化状态

**签名**：
```rust
fn save_window_state(state: WindowState) -> Result<(), String>
```

**结构**：
```rust
pub struct WindowState {
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub maximized: Option<bool>,
    pub display_id: Option<String>,
}
```

#### `load_window_state`
**用途**：加载窗口状态（用于恢复上次窗口布局）

**签名**：
```rust
fn load_window_state() -> Result<WindowState, String>
```

---

## Local PTY 模块 (`local_pty.rs`)

### `local_start_shell`
**用途**：启动本地 shell（无需 SSH 连接）

**签名**：
```rust
fn local_start_shell(
    app: tauri::AppHandle,
    cols: Option<u16>,
    rows: Option<u16>
) -> Result<String, String>
```

**前端事件监听**：
```javascript
// 与 SSH Shell 相同，通过 'shell_output' 事件接收数据
listen('shell_output', (event) => {
  const { session_id, data } = event.payload
  // 区分本地会话（session_id 以 'local-' 开头）
})
```

### `local_shell_input`
**用途**：向本地 shell 发送输入

**签名**：
```rust
fn local_shell_input(session_id: String, data: String) -> Result<(), String>
```

### `local_shell_resize`
**用途**：调整本地终端大小

**签名**：
```rust
fn local_shell_resize(session_id: String, cols: u16, rows: u16) -> Result<(), String>
```

---

## Updater 模块 (`updater.rs`)

### `check_update`
**用途**：检查应用更新

**签名**：
```rust
async fn check_update() -> Result<UpdateInfo, String>
```

**返回结构**：
```rust
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub release_url: String,
    pub release_notes: String,
    pub has_update: bool,
}
```

---

## 前端工具 (`src/utils/`)

### Tauri 封装 (`tauri.js`)

**用途**：统一封装 `invoke` 和 `listen`，支持浏览器模拟模式

```javascript
import { invoke, listen, isTauri } from './utils/tauri.js'

// 检查是否在 Tauri 环境中
if (isTauri()) {
  const sessionId = await invoke('ssh_connect', { /* ... */ })
} else {
  // 浏览器模式，使用模拟数据
  console.log('Running in browser mode')
}
```

### 国际化 (`i18n.js`)

**用途**：多语言支持（中/英）

```javascript
import { t } from './utils/i18n.js'

// 获取翻译文本
const text = t('common.save') // 保存 / Save
```

---

## 错误处理

### Rust 错误类型

所有 API 返回 `Result<T, String>`，错误信息为人类可读字符串。

**常见错误**：
- `"Connection refused"`：连接被拒绝
- `"Authentication failed"`：认证失败
- `"Timeout"`：连接超时
- `"Permission denied"`：权限不足

### 前端错误处理

```javascript
try {
  const sessionId = await invoke('ssh_connect', { /* ... */ })
} catch (error) {
  console.error('SSH connection failed:', error)
  // 显示错误提示给用户
  showToast('error', error)
}
```

---

## 安全注意事项

### 密码传输
- ✅ 密码通过 Tauri IPC 传输（进程间，不经过网络）
- ✅ 不在日志中输出密码
- ✅ 不在配置文件中明文存储密码（使用系统密钥链）

### 密钥管理
- ✅ 私钥文件路径不暴露给前端
- ✅ 密钥密码使用系统密钥链存储

### XSS 防护
- ✅ 启用 CSP 安全策略
- ✅ 避免 `innerHTML`，使用 `textContent`

---

## 性能优化

### 批量操作
- 批量上传/下载：使用 Promise.all 并行处理
- 大文件传输：使用流式传输，避免内存溢出

### 连接池
- 复用 SSH Session，避免频繁建立连接
- 超时自动断开（30 分钟无操作）

---

## 开发调试

### 日志输出
```rust
// Rust 中使用标准日志
println!("Debug: {:?}", data);

// 前端使用 console.log
console.log('Debug:', data);
```

### 浏览器模拟模式
运行 `npm run dev` 后，可在浏览器中打开 http://localhost:1420

前端会自动检测 `!isTauri()` 并使用模拟数据，方便 UI 调试。

---

## 参考资料

- [Tauri Commands]( https://tauri.app/v1/api/config/#commands)
- [ssh2-rs 文档](https://docs.rs/ssh2/)
- [Vue 3 Composition API](https://vuejs.org/guide/extras/composition-api.html)
