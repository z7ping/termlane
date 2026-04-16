<template>
  <div class="h-7 flex items-center px-3 text-xs select-none justify-between" style="background: var(--accent); color: white;">
    <div class="flex items-center gap-3">
      <span class="font-medium">⌨️ XTerminal Pro</span>
      <span v-if="connection" class="flex items-center gap-1 opacity-90">
        <span class="w-1.5 h-1.5 rounded-full" :style="{ background: sessionId ? 'var(--success)' : 'var(--warning)' }" />
        {{ connection.username }}@{{ connection.host }}
      </span>
      <span v-else class="flex items-center gap-1 opacity-90">
        <span class="w-1.5 h-1.5 rounded-full" style="background: var(--success);" />
        本地
      </span>
    </div>
    <div class="flex items-center gap-3 opacity-75">
      <span>UTF-8</span>
      <span>{{ currentTime }}</span>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted } from 'vue'

defineProps({ connection: Object, sessionId: String })

const currentTime = ref('')
let timer = null

function updateTime() {
  currentTime.value = new Date().toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' })
}

onMounted(() => { updateTime(); timer = setInterval(updateTime, 1000) })
onUnmounted(() => clearInterval(timer))
</script>
