// 空闲超时自动断开
export class IdleTimeoutManager {
  constructor(timeoutMinutes = 30) {
    this.timeout = timeoutMinutes * 60 * 1000
    this.timers = new Map()
  }

  reset(connectionId, onTimeout) {
    this.clear(connectionId)
    const timer = setTimeout(() => {
      onTimeout(connectionId)
      this.timers.delete(connectionId)
    }, this.timeout)
    this.timers.set(connectionId, timer)
  }

  clear(connectionId) {
    const timer = this.timers.get(connectionId)
    if (timer) {
      clearTimeout(timer)
      this.timers.delete(connectionId)
    }
  }

  clearAll() {
    for (const timer of this.timers.values()) clearTimeout(timer)
    this.timers.clear()
  }

  setTimeout(minutes) {
    this.timeout = minutes * 60 * 1000
  }
}

export function createIdleManager(minutes = 30) {
  return new IdleTimeoutManager(minutes)
}
