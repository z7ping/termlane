<template>
  <div class="h-7 bg-blue-600 flex items-center px-3 text-xs text-white select-none justify-between">
    <div class="flex items-center gap-3">
      <span>⌨️ XTerminal Pro v{{ version }}</span>
      <span v-if="connection" class="flex items-center gap-1">
        <span class="w-2 h-2 rounded-full" :class="sessionId ? 'bg-green-400' : 'bg-yellow-400'" />
        {{ connection.username }}@{{ connection.host }}{{ sessionId ? '' : ' (未连接)' }}
      </span>
      <span v-else class="flex items-center gap-1">
        <span class="w-2 h-2 rounded-full bg-green-400" />
        本地演示
      </span>
    </div>
    <div class="flex items-center gap-3 text-blue-200">
      <span>UTF-8</span>
      <span>{{ currentTime }}</span>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted } from 'vue'

defineProps({
  connection: Object,
  sessionId: String,
})

const version = ref('0.1.0')
const currentTime = ref('')
let timer = null

function updateTime() {
  currentTime.value = new Date().toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' })
}

onMounted(() => {
  updateTime()
  timer = setInterval(updateTime, 1000)
})

onUnmounted(() => clearInterval(timer))
</script>
