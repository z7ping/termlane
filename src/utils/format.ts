// 格式化工具函数

/**
 * 格式化字节数为可读字符串
 * @param bytes - 字节数
 * @returns 格式化后的字符串 (如 '1.5 GB')
 */
export function formatBytes(bytes: number): string {
  if (!bytes) return '0 B'
  const k = 1024
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return (bytes / Math.pow(k, i)).toFixed(1) + ' ' + units[i]
}
