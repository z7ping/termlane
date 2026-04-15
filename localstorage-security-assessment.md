# XTerminal Pro LocalStorage 安全评估报告

## 执行时间
2026-04-16 06:27

## 评估范围
所有使用localStorage的源文件

## 发现的问题

### 1. 已使用secure-storage但不够安全
**文件:** `src/utils/secure-storage.js`
**问题:**
- 使用 `btoa`/`atob` 进行Base64混淆，不是真正的加密
- 密码存储为 `xt_pwd_${connectionId}` 格式
- 在浏览器环境中，Base64编码可以轻易被解码

**风险:** 高
- 攻击者可以通过浏览器开发者工具轻易查看和修改存储的密码

### 2. 敏感数据未加密存储

#### 2.1 书签 (Bookmarks)
**Key:** `xterminal_bookmarks`
**文件:** `src/components/Bookmarks.vue`
**风险:** 中等
- 书签可能包含服务器路径、主机名等敏感信息
- 直接存储JSON明文

#### 2.2 收藏夹 (Favorites)
**Key:** `favorites_storage`
**文件:** `src/utils/favorites.js`
**风险:** 中等
- 可能包含SSH连接配置信息

#### 2.3 定时任务 (Scheduler)
**Key:** `scheduler_tasks`
**文件:** `src/utils/scheduler.js`
**风险:** 中等
- 定时任务可能包含敏感命令或路径

### 3. 非敏感配置（可保留localStorage）

以下配置不包含敏感信息，继续使用localStorage是安全的：

| Key | 用途 | 文件 |
|-----|------|------|
| `xterminal-theme` | 主题设置 | main.js, Settings.vue |
| `xterminal-fontSize` | 字体大小 | Settings.vue, TerminalPanel.vue |
| `xterminal-scrollback` | 滚动回溯行数 | Settings.vue, TerminalPanel.vue |
| `xterminal-cursorBlink` | 光标闪烁 | Settings.vue |
| `xterminal-sshTimeout` | SSH超时 | Settings.vue |
| `xterminal_locale` | 语言设置 | i18n.js |
| `shortcut_*` | 快捷键映射 | Settings.vue, TerminalPanel.vue |
| `window-state` | 窗口位置/大小 | window-state.js |

## 安全修复方案

### 方案1：Tauri系统Keyring（推荐）
**适用环境:** Tauri桌面应用
**实现:**
- 使用 Tauri 的 `@tauri-apps/plugin-secure-storage` 插件
- 利用操作系统级别的密钥环存储敏感数据
- 支持Windows DPAPI, macOS Keychain, Linux libsecret

**优势:**
- 系统级加密，安全性最高
- 自动与用户登录凭证绑定
- 不依赖应用层加密密钥

### 方案2：AES-GCM加密
**适用环境:** 浏览器环境
**实现:**
- 使用 Web Crypto API (crypto.subtle)
- AES-GCM 算法进行加密
- 加密密钥从 PBKDF2 派生（使用用户输入的PIN或应用固定密钥）

**优势:**
- 真正的加密，Base64无法破解
- 即使localStorage被导出，数据也是加密的

**劣势:**
- 需要管理加密密钥
- 比Base64稍微复杂

### 方案3：混合方案
- Tauri环境：使用系统keyring
- 浏览器环境：使用AES-GCM加密 + localStorage
- 非敏感数据：继续使用原生localStorage

## 实施建议

### 阶段1：创建统一的安全存储API
创建 `src/utils/secure-store.ts` 统一接口：

```typescript
interface SecureStore {
  set(key: string, value: any): Promise<boolean>
  get(key: string): Promise<any>
  remove(key: string): Promise<void>
}
```

### 阶段2：迁移敏感数据
按优先级迁移：
1. 密码（最高优先级）
2. 书签
3. 收藏夹
4. 定时任务

### 阶段3：清理旧数据
移除 `secure-storage.js` 的btoa实现

## 统计

- **总文件数:** 18个文件使用localStorage
- **敏感数据:** 4处需要加密
- **非敏感配置:** 8处可保留
- **安全测试:** 1个测试文件

## 结论

**当前状态:** ⚠️ 部分安全
- 密码使用了混淆但不是真正的加密
- 部分敏感数据未加密

**目标状态:** ✅ 完全安全
- 使用系统keyring或AES加密
- 所有敏感数据加密存储
- 非敏感配置继续使用localStorage
