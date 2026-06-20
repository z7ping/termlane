// Tauri invoke 安全包装

import { invoke as tauriInvoke } from './tauri'

/**
 * 安全调用 Tauri 命令，错误仅输出到 console.warn
 * @param command - 命令名
 * @param args - 命令参数
 * @returns 命令返回值
 */
export async function safeInvoke(command: string, args?: Record<string, unknown>): Promise<unknown> {
  try {
    return await tauriInvoke(command, args)
  } catch (err) {
    console.warn(`[safeInvoke] ${command} failed:`, err)
    throw err
  }
}
