<template>
  <div class="statusbar h-7 flex items-center px-3 text-xs select-none justify-between">
    <div class="flex items-center gap-2 min-w-0">
      <span class="status-dot" :class="sessionId ? 'connected' : 'idle'" aria-hidden="true" />
      <span v-if="connection" class="truncate" style="color: var(--fg-secondary);">
        {{ connection.username }}@{{ connection.host }}
      </span>
      <span v-else style="color: var(--fg-secondary);">本地</span>
      <span class="status-label">{{ sessionId ? '已连接' : '未连接' }}</span>
    </div>

    <div class="flex items-center gap-3 flex-shrink-0" style="color: var(--fg-muted);">
      <span>UTF-8</span>
      <span>{{ currentTime }}</span>
    </div>
  </div>
</template>

<script setup>
import { onMounted, onUnmounted, ref } from 'vue'

defineProps({ connection: Object, sessionId: String })

const currentTime = ref('')
let timer = null

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
  color: var(--fg-muted);
  font-size: 11px;
}
</style>
