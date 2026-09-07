# 贡献指南

感谢您对 Termlane 项目的关注！我们欢迎所有形式的贡献。

## 目录

- [开发环境搭建](#开发环境搭建)
- [贡献流程](#贡献流程)
- [代码规范](#代码规范)
- [测试要求](#测试要求)
- [文档贡献](#文档贡献)
- [问题报告](#问题报告)
- [功能请求](#功能请求)

## 开发环境搭建

### 系统要求

| 要求 | 版本 |
|------|------|
| Node.js | ≥ 18.0.0 |
| Rust | ≥ 1.70.0 |
| npm | ≥ 9.0.0 |

### 系统依赖

**Ubuntu/Debian:**
```bash
sudo apt update
sudo apt install build-essential libssl-dev pkg-config
```

**macOS:**
```bash
xcode-select --install
```

**Windows:**
- 安装 Visual Studio Build Tools
- 安装 Rust (https://rustup.rs/)

### 克隆项目

```bash
git clone https://github.com/z7ping/termlane.git
cd termlane
```

### 安装依赖

```bash
# 前端依赖
npm install

# Rust 依赖会自动安装
```

### 开发模式

```bash
npm run dev
```

访问 http://localhost:1420 查看应用。

## 贡献流程

### 1. Fork 项目

1. 访问 [Termlane](https://github.com/z7ping/termlane)
2. 点击右上角 "Fork" 按钮
3. 克隆你的 fork：

```bash
git clone https://github.com/YOUR_USERNAME/termlane.git
cd termlane
```

### 2. 创建分支

```bash
# 创建并切换到新分支
git checkout -b feat/your-feature-name

# 或者修复 bug
git checkout -b fix/your-bug-fix
```

分支命名规范：
- `feat/功能名` - 新功能
- `fix/问题描述` - Bug 修复
- `docs/文档名` - 文档更新
- `style/样式改进` - 样式改进
- `refactor/重构内容` - 代码重构
- `test/测试相关` - 测试相关
- `chore/工具相关` - 构建工具、依赖更新

### 3. 开发

遵循 [代码规范](#代码规范) 和 [测试要求](#测试要求)。

### 4. 提交

```bash
# 添加变更
git add .

# 提交（遵循提交信息规范）
git commit -m "feat: 添加 SFTP 批量上传功能"

# 推送到你的 fork
git push origin feat/your-feature-name
```

### 5. 创建 Pull Request

1. 访问你的 fork 仓库
2. 点击 "New Pull Request"
3. 选择分支：`你的分支` → `dev`
4. 填写 PR 模板
5. 等待代码审查

## 代码规范

### 提交信息规范

使用 [Conventional Commits](https://www.conventionalcommits.org/zh-hans/) 规范：

```
<类型>[可选的作用域]: <描述>

[可选的正文]

[可选的脚注]
```

**类型：**
- `feat`: 新功能
- `fix`: Bug 修复
- `docs`: 文档更新
- `style`: 代码格式（不影响逻辑）
- `refactor`: 重构
- `test`: 测试相关
- `chore`: 构建工具、依赖更新

**示例：**
```
feat(ssh): 添加跳板机连接支持

实现通过中间服务器跳转到目标服务器的功能，
支持密码和密钥认证两种方式。

Closes #123
```

### 前端代码规范

**文件命名：**
- 组件：`PascalCase.vue` (如 `TerminalPanel.vue`)
- 工具文件：`camelCase.js` (如 `tauriUtils.js`)
- 页面文件：`PascalCase.vue` (如 `ConnectionDialog.vue`)

**代码格式：**
```javascript
// 使用 Prettier 格式化
// ✅ 正确
const handleConnect = async () => {
  try {
    const sessionId = await invoke('ssh_connect', config)
    emit('connected', sessionId)
  } catch (error) {
    showToast('error', error)
  }
}

// ❌ 错误
const handleConnect=async()=>{
  try{
    const sessionId=await invoke('ssh_connect',config);
    emit('connected',sessionId)
  }catch(error){
    showToast('error',error);
  }
}
```

**Vue 组件规范：**
```vue
<template>
  <div class="component-name">
    <button @click="handleClick">{{ buttonText }}</button>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue'

const props = defineProps({
  count: {
    type: Number,
    default: 0
  }
})

const emit = defineEmits(['update', 'change'])

const localCount = ref(props.count)
const buttonText = computed(() => `Count: ${localCount.value}`)

const handleClick = () => {
  localCount.value++
  emit('update', localCount.value)
  emit('change', localCount.value)
}
</script>

<style scoped>
.component-name {
  @apply p-4 rounded-lg bg-gray-100;
}
</style>
```

### Rust 代码规范

**使用 rustfmt 格式化：**
```bash
cargo fmt
```

**使用 clippy 检查：**
```bash
cargo clippy -- -D warnings
```

**代码示例：**
```rust
// ✅ 正确
#[tauri::command]
async fn ssh_connect(
    host: String,
    port: u16,
    username: String,
    password: String,
) -> Result<String, String> {
    // 验证输入
    if host.is_empty() {
        return Err("Host cannot be empty".to_string());
    }

    // 创建连接
    match create_ssh_connection(&host, port, &username, &password).await {
        Ok(session_id) => Ok(session_id),
        Err(e) => Err(format!("Failed to connect: {}", e)),
    }
}

// ❌ 错误
#[tauri::command]
async fn ssh_connect(host:String,port:u16,username:String,password:String)->Result<String,String>{
    if host.is_empty(){
        return Err("Host cannot be empty".to_string())
    }
    match create_ssh_connection(&host,port,&username,&password).await{
        Ok(session_id)=>Ok(session_id),
        Err(e)=>Err(format!("Failed to connect: {}",e))
    }
}
```

## 测试要求

### 测试先行（TDD）

我们遵循 [RED-GREEN-REFACTOR](docs/TESTING.md) 循环：

1. **RED** - 先写失败的测试
2. **GREEN** - 写最少代码让测试通过
3. **REFACTOR** - 重构代码，保持测试通过

### 测试命令

```bash
# 运行前端测试
npm run test:unit

# 运行 Rust 测试
cargo test

# 运行所有测试
npm run test
```

### 测试覆盖率目标

| 模块 | 目标覆盖率 | 当前 |
|------|------------|------|
| Rust 后端 | 80% | ~15% |
| 前端工具函数 | 90% | ~5% |
| 核心组件 | 70% | ~0% |

新代码必须包含相应测试，确保覆盖率逐步提升。

## 文档贡献

### API 文档

- 新增 Tauri Command 需要在 [docs/API.md](docs/API.md) 中添加文档
- 包含函数签名、参数说明、返回值、示例

### 架构文档

- 重大架构变更需要更新 [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)
- 包含设计决策、数据结构、流程图

### README 更新

- 新功能需要更新 README.md 的功能列表
- 重大变更需要更新版本号和发布说明

## 问题报告

### 使用 Issue 模板

报告 Bug 请使用 [Bug Report](https://github.com/z7ping/termlane/issues/new?template=bug_report.md) 模板。

### 必要信息

**环境信息：**
- 操作系统和版本
- 应用版本
- Rust 版本
- Node.js 版本

**问题描述：**
- 预期行为
- 实际行为
- 重现步骤
- 错误信息
- 截图（UI 相关）

**最小示例：**
```javascript
// 提供最小可复现代码
const config = {
  host: '192.168.1.100',
  username: 'root',
  password: 'password'
}

// 这段代码会触发错误
connect(config)
```

## 功能请求

### 使用 Feature Request 模板

提出新功能请使用 [Feature Request](https://github.com/z7ping/termlane/issues/new?template=feature_request.md) 模板。

### 功能描述

包含以下信息：
- 功能解决的问题
- 使用场景
- 预期实现方式
- 替代方案

## 代码审查

### 审查清单

**功能性：**
- [ ] 功能实现正确
- [ ] 边界条件处理
- [ ] 错误处理完善

**代码质量：**
- [ ] 代码格式规范
- [ ] 命名清晰
- [ ] 注释充分
- [ ] 无硬编码

**测试：**
- [ ] 单元测试覆盖
- [ ] 集成测试
- [ ] 测试通过

**文档：**
- [ ] API 文档更新
- [ ] 架构文档更新
- [ ] README 更新

### 审查流程

1. 自动检查：
   - CI/CD 运行测试
   - 代码风格检查
   - 安全扫描

2. 人工审查：
   - 功能正确性
   - 代码质量
   - 性能影响

3. 审查通过：
   - 贡献者修改意见
   - 审查者批准
   - 合并到 dev 分支

## 发布流程

1. **dev 分支** - 开发和功能集成分支
2. **main 分支** - 稳定发布分支
3. **发布版本**：
   ```bash
   git checkout main
   git merge dev
   git tag v2.0.0
   git push origin v2.0.0
   ```
4. **构建发布**：
   - GitHub Actions 自动构建
   - 生成安装包
   - 创建 Release

## 行为准则

### 基本原则

- 尊重他人，礼貌沟通
- 建设性意见，具体明确
- 接受反馈，持续改进
- 专注技术，避免争议

### 沟通渠道

- **Issues** - Bug 报告、功能请求
- **Discussions** - 技术讨论、问题求助
- **Pull Request** - 代码审查、变更讨论

## 获取帮助

- 📧 邮件：admin@7ping.site
- 💬 讨论：[GitHub Discussions](https://github.com/z7ping/termlane/discussions)
- 📖 文档：[Wiki](https://github.com/z7ping/termlane/wiki)
- 🐛 问题：[Issues](https://github.com/z7ping/termlane/issues)

## 致谢

感谢所有贡献者，无论大小贡献都让 Termlane 变得更好！

---

再次感谢您的贡献！🎉