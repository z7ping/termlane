# XTerminal Pro - 编码标准与注释规范

## 目录

- [代码注释规范](#代码注释规范)
- [JavaScript/Vue 注释规范](#javascriptvue-注释规范)
- [Rust 注释规范](#rust-注释规范)
- [需要补充注释的核心模块](#需要补充注释的核心模块)
- [注释最佳实践](#注释最佳实践)

## 代码注释规范

### 注释分类

1. **文件头注释** - 说明文件用途和作者
2. **函数/方法注释** - 说明功能和参数
3. **复杂逻辑注释** - 解释算法或业务逻辑
4. **TODO/FIXME 注释** - 标记待办事项

### 注释语言

- 使用中文注释（项目主要面向中文开发者）
- 必要时使用英文注释（如 API 文档、对外接口）

## JavaScript/Vue 注释规范

### 文件头

```javascript
/**
 * @fileoverview 组件功能描述
 * @author 作者名
 * @since 创建日期
 * @version 当前版本
 */
```

### 组件注释

```vue
<template>
  <!-- SSH 连接对话框组件，用于创建和编辑 SSH 连接配置 -->
  <div class="connection-dialog">
    <!-- 基本信息 -->
    <div>...</div>
  </div>
</template>

<script setup>
/**
 * SSH 连接对话框组件
 * 支持密码认证、密钥认证、跳板机连接
 */
import { ref, computed, onMounted } from 'vue'

// Props 定义
const props = defineProps({
  editing: {
    type: Object,
    default: null
    // 正在编辑的连接对象，null 表示新建
  }
})

// Emits 定义
const emit = defineEmits([
  'save',    // 保存连接，参数: connection
  'close'    // 关闭对话框
])

// 表单数据
const form = ref({
  name: '',
  host: '',
  port: 22,
  username: '',
  authType: 'password',
  password: '',
  keyPath: '',
  passphrase: '',
  useJumpHost: false,
  jumpHost: '',
  jumpPort: 22,
  jumpUsername: '',
  jumpPassword: '',
  group: '',
  tags: []
})

/**
 * 验证表单数据
 * @returns {boolean} 验证结果
 */
const validateForm = () => {
  // 验证必填字段
  if (!form.value.name || !form.value.host || !form.value.username) {
    showToast('error', '请填写必填字段')
    return false
  }
  
  // 验证端口
  if (form.value.port < 1 || form.value.port > 65535) {
    showToast('error', '端口范围：1-65535')
    return false
  }
  
  return true
}

/**
 * 处理连接保存
 */
const handleSave = () => {
  if (!validateForm()) return
  
  // 保存连接
  emit('save', { ...form.value })
  emit('close')
}
</script>
```

### 工具函数注释

```javascript
/**
 * 格式化文件大小
 * @param {number} bytes - 字节数
 * @param {number} [precision=2] - 精度
 * @returns {string} 格式化后的大小（如 1.5 MB）
 */
export function formatFileSize(bytes, precision = 2) {
  if (bytes === 0) return '0 B'
  
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  
  return parseFloat((bytes / Math.pow(k, i)).toFixed(precision)) + ' ' + sizes[i]
}

/**
 * 解析 SSH 连接字符串
 * @param {string} str - 连接字符串，格式：user@host:port
 * @returns {Object} 解析后的连接信息
 */
export function parseConnectionString(str) {
  // 实现解析逻辑...
}
```

## Rust 注释规范

### 文件头

```rust
//! SSH 连接管理模块
//! 
//! 提供 SSH 连接、PTY Shell、跳板机等功能
//! 使用 ssh2-rs 库实现底层连接

use std::collections::HashMap;
use std::sync::Mutex;
use ssh2::Session;
```

### 结构体注释

```rust
/// SSH 会话信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshSession {
    /// 会话唯一标识符
    pub id: String,
    /// 主机地址
    pub host: String,
    /// 端口号
    pub port: u16,
    /// 用户名
    pub username: String,
    /// 是否已连接
    pub connected: bool,
}

/// 监控数据结构
/// 
/// 包含服务器的 CPU、内存、磁盘、负载等信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorData {
    /// CPU 使用率（百分比）
    pub cpu_usage: f64,
    /// 内存总量（字节）
    pub memory_total: u64,
    /// 内存已使用量（字节）
    pub memory_used: u64,
    /// 内存使用率（百分比）
    pub memory_percent: f64,
    /// 磁盘总量（字节）
    pub disk_total: u64,
    /// 磁盘已使用量（字节）
    pub disk_used: u64,
    /// 磁盘使用率（百分比）
    pub disk_percent: f64,
    /// 1分钟平均负载
    pub load_1: f64,
    /// 5分钟平均负载
    pub load_5: f64,
    /// 15分钟平均负载
    pub load_15: f64,
    /// 系统运行时间（秒）
    pub uptime_seconds: u64,
}
```

### 函数注释

```rust
/// 建立 SSH 连接
/// 
/// # 参数
/// 
/// * `host` - 主机地址
/// * `port` - 端口号
/// * `username` - 用户名
/// * `password` - 密码
/// 
/// # 返回值
/// 
/// * `Ok(String)` - 会话ID
/// * `Err(String)` - 错误信息
/// 
/// # 示例
/// 
/// ```rust
/// let session_id = ssh_connect(
///     "192.168.1.100".to_string(),
///     22,
///     "root".to_string(),
///     "password".to_string()
/// ).await?;
/// ```
#[tauri::command]
async fn ssh_connect(
    host: String,
    port: u16,
    username: String,
    password: String,
) -> Result<String, String> {
    // 验证输入参数
    if host.is_empty() {
        return Err("主机地址不能为空".to_string());
    }
    
    // 创建 TCP 连接
    let tcp = TcpStream::connect(format!("{}:{}", host, port))
        .await
        .map_err(|e| format!("连接失败: {}", e))?;
    
    // 创建 SSH 会话
    let mut session = Session::new()?;
    session.set_tcp_stream(tcp);
    session.handshake()
        .map_err(|e| format!("SSH 握手失败: {}", e))?;
    
    // 用户认证
    session.userauth_password(&username, &password)
        .map_err(|e| format!("认证失败: {}", e))?;
    
    // 生成会话ID
    let session_id = Uuid::new_v4().to_string();
    
    // 存储会话
    lock!(SESSIONS).insert(session_id.clone(), session);
    lock!(SESSION_INFO).insert(session_id.clone(), SshSession {
        id: session_id.clone(),
        host,
        port,
        username,
        connected: true,
    });
    
    Ok(session_id)
}
```

## 需要补充注释的核心模块

### 前端组件

#### 1. TerminalPanel.vue
```javascript
// 当前缺少注释的部分：
// - PTY 连接建立流程
// - 数据处理逻辑
// - 事件处理机制
```

#### 2. SftpPanel.vue
```javascript
// 当前缺少注释的部分：
// - 文件列表渲染逻辑
// - 拖拽上传实现
// - 批量选择处理
```

#### 3. ConnectionMonitor.vue
```javascript
// 当前缺少注释的部分：
// - 监控数据获取逻辑
// - 图表渲染逻辑
// - 数据刷新机制
```

### Rust 后端

#### 1. ssh.rs
```rust
// 当前缺少注释的部分：
// - SSH 连接池管理
// - PTY Shell 输出处理线程
// - 会话录制实现
```

#### 2. sftp.rs
```rust
// 当前缺少注释的部分：
// - 文件传输进度处理
// - 并发上传管理
// - 错误恢复机制
```

#### 3. config.rs
```rust
// 当前缺少注释的部分：
// - 加密存储实现
// - 配置文件迁移逻辑
// - 窗口状态同步
```

## 注释最佳实践

### ✅ 推荐做法

1. **解释为什么，而不是是什么**
   ```javascript
   // ❌ 不好的注释
   i++; // i 加 1
   
   // ✅ 好的注释
   i++; // 跳过已处理的元素
   ```

2. **保持注释简洁**
   ```javascript
   // ❌ 过于啰嗦
   // 这个函数的主要作用是接收一个包含用户信息的对象作为参数，
   // 然后从这个对象中提取出用户的ID，接着使用这个ID去数据库
   // 中查询对应的用户详细信息，最后返回查询结果。
   
   // ✅ 简洁明了
   // 根据用户ID查询详细信息
   ```

3. **注释复杂的算法**
   ```javascript
   // 实现快速排序
   const quickSort = (arr) => {
     if (arr.length < 2) return arr
     
     // 选择基准值
     const pivot = arr[Math.floor(arr.length / 2)]
     
     // 分区：小于基准 | 等于基准 | 大于基准
     const left = arr.filter(x => x < pivot)
     const middle = arr.filter(x => x === pivot)
     const right = arr.filter(x => x > pivot)
     
     // 递归排序并合并
     return [...quickSort(left), ...middle, ...quickSort(right)]
   }
   ```

4. **使用 TODO 和 FIXME**
   ```javascript
   // TODO: 优化大文件上传性能
   // FIXME: 处理网络中断的重连逻辑
   // NOTE: 这里需要保持线程安全
   ```

### ❌ 避免的做法

1. **不要注释掉的代码**
   ```javascript
   // ❌ 删除这些代码
   // const oldFunction = () => {
   //   // ...
   // }
   
   // ✅ 使用版本控制系统管理代码历史
   ```

2. **不要添加冗余的注释**
   ```javascript
   // ❌ 显而易见，无需注释
   const userName = 'john'; // 用户名
   
   // ✅ 只在必要时添加注释
   const API_TIMEOUT = 30000; // API 超时时间（毫秒）
   ```

3. **不要使用模糊的注释**
   ```javascript
   // ❌ 模糊不清
   // 处理一些特殊情况
   
   // ✅ 具体明确
   // 处理权限不足的特殊情况
   ```

## 注释工具

### VS Code 插件

- **Document This** - 自动生成 JSDoc 注释
- **Better Comments** - 美化注释显示
- **Todo Tree** - 管理 TODO 注释

### 代码检查

```bash
# JavaScript 注释检查
npm install -g eslint-plugin-jsdoc

# Rust 文档注释检查
cargo doc --no-deps --document-private-items
```

## 文档生成

### 前端 API 文档

使用 JSDoc 生成 API 文档：

```bash
npm install -g jsdoc
jsdoc src/utils/ -d docs/api/
```

### Rust 文档

使用 rustdoc 生成文档：

```bash
cargo doc --open
```

## 更新计划

### 第一阶段：核心模块注释（P0）

1. TerminalPanel.vue - 终端核心组件
2. SftpPanel.vue - 文件管理组件  
3. ssh.rs - SSH 连接模块
4. sftp.rs - SFTP 模块

### 第二阶段：辅助模块注释（P1）

1. ConnectionMonitor.vue - 监控组件
2. BatchCommand.vue - 批量命令组件
3. config.rs - 配置模块

### 第三阶段：完善注释标准（P2）

1. 建立注释审查流程
2. 添加注释覆盖率检查
3. 创建注释生成脚本

---

## 参考资料

- [JSDoc 文档](https://jsdoc.app/)
- [Rust 文档注释](https://doc.rust-lang.org/rustdoc/the-doc-attribute.html)
- [Google JavaScript 风格指南](https://google.github.io/styleguide/jsguide.html)
- [Rust API 指南](https://rust-lang.github.io/api-guidelines/)