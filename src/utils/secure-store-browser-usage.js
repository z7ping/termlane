/**
 * XTerminal Pro 浏览器安全存储使用示例
 * 展示如何在浏览器环境中使用加密存储
 */

import { 
  secureStore, 
  storePassword, 
  getPassword, 
  removePassword,
  storeBookmarks,
  getBookmarks,
  storeFavorites,
  getFavorites,
  checkBrowserSupport,
  getStorageInfo
} from './secure-store-browser.js'

/**
 * 初始化浏览器存储
 */
async function initBrowserStorage() {
  // 检查浏览器兼容性
  if (!checkBrowserSupport()) {
    throw new Error('Browser does not support required Crypto APIs')
  }
  
  console.log('✅ Browser secure storage initialized')
  return true
}

/**
 * 密码管理示例
 */
async function passwordExample() {
  const connectionId = 'ssh-server-1'
  const password = 'my-secret-password'
  
  // 存储密码
  const stored = await storePassword(connectionId, password)
  console.log(`Password stored for ${connectionId}:`, stored ? '✅' : '❌')
  
  // 读取密码
  const retrieved = await getPassword(connectionId)
  console.log(`Password retrieved for ${connectionId}:`, retrieved === password ? '✅' : '❌')
  
  // 删除密码
  await removePassword(connectionId)
  const afterDelete = await getPassword(connectionId)
  console.log(`Password after delete:`, afterDelete === null ? '✅' : '❌')
}

/**
 * 书签管理示例
 */
async function bookmarksExample() {
  const bookmarks = [
    { id: 1, name: 'Production Server', host: '192.168.1.100', port: 22 },
    { id: 2, name: 'Development Server', host: 'dev.example.com', port: 2222 }
  ]
  
  // 存储书签
  await storeBookmarks(bookmarks)
  console.log('✅ Bookmarks stored')
  
  // 读取书签
  const retrieved = await getBookmarks()
  console.log('Retrieved bookmarks:', retrieved.length, 'items')
  
  return retrieved
}

/**
 * 存储信息统计示例
 */
async function storageInfoExample() {
  const info = await getStorageInfo()
  console.log('Storage Info:')
  console.log(`- Keys: ${info.keyCount}`)
  console.log(`- Total size: ${info.totalSize} bytes`)
  console.log('- Details:', info.details)
}

/**
 * 完整使用示例
 */
async function fullExample() {
  try {
    await initBrowserStorage()
    await passwordExample()
    await bookmarksExample()
    await storageInfoExample()
    
    console.log('\n✅ All browser storage examples completed successfully!')
  } catch (error) {
    console.error('❌ Example failed:', error)
  }
}

// 导出示例函数
export {
  initBrowserStorage,
  passwordExample,
  bookmarksExample,
  storageInfoExample,
  fullExample
}

// 如果直接运行此文件
if (typeof window === 'object' && window.location) {
  // 浏览器环境
  fullExample()
}