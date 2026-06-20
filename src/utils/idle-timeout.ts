// 空闲超时自动断开

export type TimeoutCallback = (connectionId: string) => void

export class IdleTimeoutManager {
  private timeout: number
  private timers: Map<string, ReturnType<typeof setTimeout>>

  constructor(timeoutMinutes = 30) {
    this.timeout = timeoutMinutes * 60 * 1000
    this.timers = new Map()
  }

  reset(connectionId: string, onTimeout: TimeoutCallback): void {
    this.clear(connectionId)
    const timer = setTimeout(() => {
      onTimeout(connectionId)
      this.timers.delete(connectionId)
    }, this.timeout)
    this.timers.set(connectionId, timer)
  }

  clear(connectionId: string): void {
    const timer = this.timers.get(connectionId)
    if (timer) {
      clearTimeout(timer)
      this.timers.delete(connectionId)
    }
  }

  clearAll(): void {
    for (const timer of this.timers.values()) clearTimeout(timer)
    this.timers.clear()
  }

  setTimeout(minutes: number): void {
    this.timeout = minutes * 60 * 1000
  }
}

export function createIdleManager(minutes = 30): IdleTimeoutManager {
  return new IdleTimeoutManager(minutes)
}
