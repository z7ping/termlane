<template>
  <div class="statusbar h-7 flex items-center px-3 text-xs select-none justify-between">
    <div class="status-left">
      <span class="status-dot" :class="sessionId ? 'connected' : 'idle'" aria-hidden="true" />
      <span class="connection-label">{{ connectionLabel }}</span>
      <span class="status-label">{{ sessionId ? '已连接' : '未连接' }}</span>
    </div>

    <div class="status-right">
      <span>UTF-8</span>
      <span>{{ currentTime }}</span>
    </div>
  </div>
</template>

<script setup>
import { computed, onMounted, onUnmounted, ref } from 'vue'

const props = defineProps({ connection: Object, sessionId: String })

const currentTime = ref('')
let timer = null

const connectionLabel = computed(() => {
  const connection = props.connection
  if (!connection) return '本地终端'
  if (connection.authType === 'local' || ['localhost', '127.0.0.1'].includes(connection.host)) return '本地终端'
  const username = connection.username?.trim()
  const host = connection.host?.trim()
  if (username && host) return `${username}@${host}`
  return connection.name || host || '远程连接'
})

function updateTime() {
  currentTime.value = new Date().toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' })
}

onMounted(() => {
  updateTime()
  timer = setInterval(updateTime, 60_000)
})

onUnmounted(() => clearInterval(timer))
</script>

<style scoped>
.statusbar {
  background: var(--bg-surface);
  border-top: 1px solid var(--border-subtle);
}

.status-left,
.status-right {
  display: flex;
  align-items: center;
}

.status-left {
  min-width: 0;
  gap: 8px;
}

.status-right {
  flex-shrink: 0;
  gap: 12px;
  color: var(--fg-muted);
}

.connection-label {
  min-width: 0;
  overflow: hidden;
  color: var(--fg-secondary);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.status-dot {
  width: 6px;
  height: 6px;
  flex-shrink: 0;
  border-radius: 999px;
}

.status-dot.connected {
  background: var(--success);
}

.status-dot.idle {
  background: var(--fg-muted);
}

.status-label {
  flex-shrink: 0;
  color: var(--fg-muted);
  font-size: 11px;
}

@media (max-width: 620px) {
  .status-right > span:first-child {
    display: none;
  }
}
</style>
