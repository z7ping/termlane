// 断线自动重连

export type ReconnectCallback = (connectionId: string) => Promise<void>

export class AutoReconnectManager {
  maxRetries: number
  retryDelay: number
  private retries: Map<string, number>
  private reconnectCallbacks: Map<string, ReconnectCallback>

  constructor(maxRetries = 3, retryDelay = 5_000) {
    this.maxRetries = maxRetries
    this.retryDelay = retryDelay
    this.retries = new Map()
    this.reconnectCallbacks = new Map()
  }

  register(connectionId: string, callback: ReconnectCallback): void {
    this.reconnectCallbacks.set(connectionId, callback)
    this.retries.set(connectionId, 0)
  }

  onDisconnect(connectionId: string): boolean {
    const retries = this.retries.get(connectionId) || 0
    const callback = this.reconnectCallbacks.get(connectionId)

    if (retries < this.maxRetries && callback) {
      this.retries.set(connectionId, retries + 1)

      setTimeout(() => {
        callback(connectionId).catch(() => {
          this.onDisconnect(connectionId) // Recursive retry
        })
      }, this.retryDelay * (retries + 1)) // Exponential backoff

      return true
    }

    return false
  }

  onConnect(connectionId: string): void {
    this.retries.set(connectionId, 0) // Reset on successful connection
  }

  unregister(connectionId: string): void {
    this.retries.delete(connectionId)
    this.reconnectCallbacks.delete(connectionId)
  }
}

export function createReconnectManager(maxRetries = 3, retryDelay = 5_000): AutoReconnectManager {
  return new AutoReconnectManager(maxRetries, retryDelay)
}
