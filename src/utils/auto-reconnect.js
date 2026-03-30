// 断线自动重连
export class AutoReconnectManager {
  constructor(maxRetries = 3, retryDelay = 5000) {
    this.maxRetries = maxRetries
    this.retryDelay = retryDelay
    this.retries = new Map()
    this.reconnectCallbacks = new Map()
  }

  register(connectionId, callback) {
    this.reconnectCallbacks.set(connectionId, callback)
    this.retries.set(connectionId, 0)
  }

  onDisconnect(connectionId) {
    const retries = this.retries.get(connectionId) || 0
    const callback = this.reconnectCallbacks.get(connectionId)

    if (retries < this.maxRetries && callback) {
      this.retries.set(connectionId, retries + 1)
      console.log(`[AutoReconnect] ${connectionId} 第${retries + 1}次重连...`)

      setTimeout(() => {
        callback(connectionId).catch(() => {
          this.onDisconnect(connectionId) // Recursive retry
        })
      }, this.retryDelay * (retries + 1)) // Exponential backoff

      return true
    }

    console.log(`[AutoReconnect] ${connectionId} 已达最大重试次数`)
    return false
  }

  onConnect(connectionId) {
    this.retries.set(connectionId, 0) // Reset on successful connection
  }

  unregister(connectionId) {
    this.retries.delete(connectionId)
    this.reconnectCallbacks.delete(connectionId)
  }
}

export function createReconnectManager(maxRetries = 3, retryDelay = 5000) {
  return new AutoReconnectManager(maxRetries, retryDelay)
}
