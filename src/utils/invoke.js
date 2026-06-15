// Tauri invoke 安全包装

import { invoke as tauriInvoke } from './tauri.js'

/**
 * 安全调用 Tauri 命令，错误仅输出到 console.warn
 * @param {string} command - 命令名
 * @param {Object} [args] - 命令参数
 * @returns {Promise<any>} 命令返回值
 */
export async function safeInvoke(command, args) {
  try {
    return await tauriInvoke(command, args)
  } catch (err) {
    console.warn(`[safeInvoke] ${command} failed:`, err)
    throw err
  }
}
