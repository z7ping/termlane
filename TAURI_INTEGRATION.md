# Tauri 系统密钥环集成指南

## 概述

XTerminal Pro的安全存储模块已支持Tauri系统密钥环集成，这将提供最高级别的数据保护。

## 安装步骤

### 1. 安装插件

```bash
# 使用 npm
npm install @tauri-apps/plugin-secure-storage

# 或使用 yarn
yarn add @tauri-apps/plugin-secure-storage

# 或使用 pnpm
pnpm add @tauri-apps/plugin-secure-storage
```

### 2. 配置Tauri

在 `src-tauri/Cargo.toml` 中添加：

```toml
[dependencies]
tauri-plugin-secure-storage = { version = "2" }
```

### 3. 初始化插件

在 `src-tauri/src/main.rs` 中添加：

```rust
use tauri_plugin_secure_storage::*;

fn main() {
    tauri::Builder::default()
        .plugin(
            init_secure_storage_builder("com.xterminal.pro")
                .key_suffix("XTerminalPro")
                .key_access_mode(KeyAccessMode::CurrentUser)
                .build(),
        )
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

## 安全机制

### Windows
- 使用 DPAPI (Data Protection API)
- 数据与用户凭据绑定
- 支持自动解密

### macOS
- 使用 Keychain Services
- 存储在登录钥匙串中
- 支持访问控制列表

### Linux
- 使用 libsecret/GNOME Keyring
- 或 KWallet (KDE环境)
- 支持主密码加密

## 使用示例

代码无需修改，`secure-store.ts`会自动检测环境并使用系统密钥环：

```typescript
// 这段代码在Tauri环境会自动使用系统密钥环
await storePassword('connection1', 'super_secret')
const pwd = await getPassword('connection1')
```

## 验证集成

运行应用后，检查浏览器控制台：

```
✓ Storage already migrated, skipping
```

没有错误消息表示集成成功。

## 注意事项

1. **开发环境**：开发时会降级到浏览器加密
2. **生产环境**：构建后使用系统密钥环
3. **迁移**：首次启动会自动迁移现有数据
4. **权限**：不需要额外权限配置

## 故障排除

### 错误："Secure storage plugin not found"
确保：
1. 已安装 `@tauri-apps/plugin-secure-storage`
2. Cargo.toml 中已添加依赖
3. main.rs 中已初始化插件

### 错误："Keyring access denied"
可能原因：
1. 系统密钥环服务未运行
2. 权限不足（Linux需要访问dbus）

### 数据迁移失败
1. 检查控制台错误信息
2. 确保有足够的磁盘空间
3. 尝试删除应用的密钥环数据后重试