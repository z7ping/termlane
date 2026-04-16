# XTerminal Pro 浏览器版本安全存储

这是 XTerminal Pro 的纯浏览器版本安全存储实现，当 `@tauri-apps/plugin-secure-storage` 安装有问题时的备用解决方案。

## 特性

- ✅ **纯浏览器兼容** - 不依赖任何 Tauri API
- ✅ **AES-GCM 加密** - 使用 Web Crypto API 进行强加密
- ✅ **会话密钥管理** - 密钥存储在 sessionStorage，会话结束时自动清除
- ✅ **本地存储** - 加密数据存储在 localStorage
- ✅ **完整 API** - 与 Tauri 版本保持相同的接口

## 使用方法

### 1. 导入模块

```javascript
import { 
  secureStore,
  storePassword,
  getPassword,
  removePassword,
  storeBookmarks,
  getBookmarks,
  storeFavorites,
  getFavorites
} from './secure-store-browser.js'
```

### 2. 基本使用

```javascript
// 存储密码
await storePassword('connection-1', 'my-secret-password')

// 读取密码
const password = await getPassword('connection-1')

// 删除密码
await removePassword('connection-1')
```

### 3. 存储书签

```javascript
const bookmarks = [
  { id: 1, name: 'Server 1', host: '192.168.1.100' },
  { id: 2, name: 'Server 2', host: '192.168.1.101' }
]

await storeBookmarks(bookmarks)
const savedBookmarks = await getBookmarks()
```

## API 参考

### 核心类: BrowserSecureStorage

```javascript
const store = new BrowserSecureStorage()

// 设置值
await store.set('key', { data: 'value' })

// 获取值
const value = await store.get('key')

// 删除键
await store.remove('key')

// 清除所有数据
await store.clear()

// 获取所有键
const keys = await store.keys()

// 检查键是否存在
const hasKey = await store.has('key')
```

### 辅助函数

| 函数 | 描述 |
|------|------|
| `storePassword(id, pwd)` | 存储连接密码（键名: `pwd_{id}`） |
| `getPassword(id)` | 获取连接密码 |
| `removePassword(id)` | 删除连接密码 |
| `storeBookmarks(bookmarks)` | 存储书签列表 |
| `getBookmarks()` | 获取书签列表 |
| `storeFavorites(favorites)` | 存储收藏夹 |
| `getFavorites()` | 获取收藏夹 |
| `storeSchedulerTasks(tasks)` | 存储定时任务 |
| `getSchedulerTasks()` | 获取定时任务 |
| `checkBrowserSupport()` | 检查浏览器兼容性 |
| `getStorageInfo()` | 获取存储统计信息 |

## 安全特性

1. **AES-GCM 加密**: 使用 256 位密钥的 AES-GCM 算法
2. **随机 IV**: 每次加密都生成新的初始化向量
3. **会话密钥**: 密钥存储在 sessionStorage，浏览器关闭时清除
4. **前缀隔离**: 所有存储键都添加 `xt_secure_` 前缀，避免冲突

## 浏览器兼容性

需要以下浏览器 API 支持：

- `window.crypto.subtle` - Web Crypto API
- `localStorage` - 本地存储
- `sessionStorage` - 会话存储
- `btoa` / `atob` - Base64 编码/解码

现代浏览器（Chrome 49+, Firefox 34+, Safari 10+）都支持这些 API。

## 从 Tauri 版本迁移

如果需要从 Tauri 版本迁移到浏览器版本，只需更改导入：

```javascript
// 原 Tauri 版本
import { secureStore } from './secure-store.js'

// 改为浏览器版本
import { secureStore } from './secure-store-browser.js'
```

API 完全兼容，无需修改其他代码。

## 测试

运行测试验证功能：

```bash
# 浏览器环境测试
# 参考 secure-store-browser-usage.js 中的示例代码
```

## 注意事项

1. **安全性**: 虽然使用了强加密，但 localStorage 仍然可以在浏览器中访问。这是浏览器环境的限制。
2. **会话管理**: 密钥存储在 sessionStorage 中，用户关闭浏览器标签页后需要重新生成密钥，之前的数据将无法解密。
3. **存储空间**: localStorage 有存储限制（通常 5-10MB），不适合存储大量数据。

## 文件说明

- `secure-store.js` - 原始文件（包含 Tauri 和浏览器双版本）
- `secure-store.js.backup` - 原始文件备份
- `secure-store-browser.js` - 纯浏览器版本
- `secure-store-browser-usage.js` - 使用示例
- `README-browser-secure-storage.md` - 本说明文档