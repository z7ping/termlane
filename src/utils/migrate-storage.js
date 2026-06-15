/**
 * XTerminal Pro 存储迁移脚本
 * 将旧的非安全存储迁移到新的安全存储
 */

import { secureStore } from './secure-store-browser.js'

/**
 * 迁移书签数据
 */
async function migrateBookmarks() {
  try {
    const oldKey = 'xterminal_bookmarks'
    const oldData = localStorage.getItem(oldKey)

    if (!oldData) {
      console.log('Bookmarks: No old data found, skipping')
      return true
    }

    const bookmarks = JSON.parse(oldData)
    const success = await secureStore.set('bookmarks', bookmarks)

    if (success) {
      // 迁移成功，删除旧数据
      localStorage.removeItem(oldKey)
      console.log(`✓ Bookmarks migrated successfully (${bookmarks.length} items)`)
    } else {
      console.error('✗ Bookmarks migration failed')
    }

    return success
  } catch (error) {
    console.error('Bookmarks migration error:', error)
    return false
  }
}

/**
 * 迁移收藏夹数据
 */
async function migrateFavorites() {
  try {
    const oldKey = 'xterminal_favorites'
    const oldData = localStorage.getItem(oldKey)

    if (!oldData) {
      console.log('Favorites: No old data found, skipping')
      return true
    }

    const favorites = JSON.parse(oldData)
    const success = await secureStore.set('favorites', favorites)

    if (success) {
      localStorage.removeItem(oldKey)
      console.log(`✓ Favorites migrated successfully (${favorites.length} items)`)
    } else {
      console.error('✗ Favorites migration failed')
    }

    return success
  } catch (error) {
    console.error('Favorites migration error:', error)
    return false
  }
}

/**
 * 迁移定时任务数据
 */
async function migrateSchedulerTasks() {
  try {
    const oldKey = 'scheduler_tasks'
    const oldData = localStorage.getItem(oldKey)

    if (!oldData) {
      console.log('Scheduler tasks: No old data found, skipping')
      return true
    }

    const tasks = JSON.parse(oldData)
    const success = await secureStore.set('scheduler_tasks', tasks)

    if (success) {
      localStorage.removeItem(oldKey)
      console.log(`✓ Scheduler tasks migrated successfully (${tasks.length} items)`)
    } else {
      console.error('✗ Scheduler tasks migration failed')
    }

    return success
  } catch (error) {
    console.error('Scheduler tasks migration error:', error)
    return false
  }
}

/**
 * 迁移密码数据
 * 从旧的 Base64 编码迁移到新的 AES-GCM 加密
 */
async function migratePasswords() {
  try {
    const prefix = 'xt_pwd_'
    let migratedCount = 0

    // 查找所有旧密码键
    const keys = []
    for (let i = 0; i < localStorage.length; i++) {
      const key = localStorage.key(i)
      if (key?.startsWith(prefix)) {
        keys.push(key)
      }
    }

    if (keys.length === 0) {
      console.log('Passwords: No old passwords found, skipping')
      return true
    }

    for (const oldKey of keys) {
      try {
        const oldData = localStorage.getItem(oldKey)
        if (!oldData) continue

        // 旧的 Base64 编码数据
        const password = JSON.parse(decodeURIComponent(atob(oldData)))
        const connectionId = oldKey.replace(prefix, '')

        // 使用新的安全存储
        const success = await secureStore.set(`pwd_${connectionId}`, password)

        if (success) {
          localStorage.removeItem(oldKey)
          migratedCount++
        } else {
          console.error(`✗ Password migration failed for ${connectionId}`)
        }
      } catch (error) {
        console.error(`Password migration error for ${oldKey}:`, error)
      }
    }

    console.log(`✓ Passwords migrated successfully (${migratedCount} items)`)
    return true
  } catch (error) {
    console.error('Passwords migration error:', error)
    return false
  }
}

/**
 * 迁移宏数据
 */
async function migrateMacros() {
  try {
    const oldKey = 'xterminal_macros'
    const oldData = localStorage.getItem(oldKey)

    if (!oldData) {
      console.log('Macros: No old data found, skipping')
      return true
    }

    const macros = JSON.parse(oldData)
    const success = await secureStore.set('xterminal_macros', macros)

    if (success) {
      localStorage.removeItem(oldKey)
      console.log(`✓ Macros migrated successfully (${macros.length} items)`)
    } else {
      console.error('✗ Macros migration failed')
    }

    return success
  } catch (error) {
    console.error('Macros migration error:', error)
    return false
  }
}

/**
 * 执行所有迁移
 */
export async function migrateAll() {
  console.log('🔄 Starting storage migration...')

  const results = await Promise.all([
    migrateBookmarks(),
    migrateFavorites(),
    migrateSchedulerTasks(),
    migratePasswords(),
    migrateMacros(),
  ])

  const successCount = results.filter(r => r).length
  const totalCount = results.length

  console.log('\n' + '='.repeat(50))
  console.log(`Migration complete: ${successCount}/${totalCount} succeeded`)
  console.log('='.repeat(50))

  // 标记迁移已完成
  localStorage.setItem('xt_storage_migrated', 'v1')

  return successCount === totalCount
}

/**
 * 检查是否需要迁移
 */
export function needsMigration() {
  return localStorage.getItem('xt_storage_migrated') !== 'v1'
}

/**
 * 自动迁移（在应用启动时调用）
 */
export async function autoMigrate() {
  if (needsMigration()) {
    console.log('Storage requires migration...')
    await migrateAll()
  } else {
    console.log('✓ Storage already migrated, skipping')
  }
}