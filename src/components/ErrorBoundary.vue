<template>
  <div :key="errorKey">
    <slot v-if="!hasError" />
    <div v-else class="h-full flex items-center justify-center" style="background: var(--bg-base)">
      <div class="text-center p-8">
        <div class="text-5xl mb-4">😵</div>
        <div class="text-lg mb-2" style="color: var(--fg-primary)">组件出错了</div>
        <div class="text-sm mb-4" style="color: var(--fg-muted)">{{ errorMessage }}</div>
        <button @click="reset" class="px-4 py-2 rounded text-sm" style="background: var(--accent); color: white">重试</button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted, onErrorCaptured } from 'vue'

const hasError = ref(false)
const errorMessage = ref('')
const errorKey = ref(0)

onErrorCaptured((err) => {
  hasError.value = true
  errorMessage.value = err.message || '未知错误'
  return false // Prevent propagation
})

function onGlobalError(e) {
  hasError.value = true
  errorMessage.value = e.message || '未捕获的错误'
}

function onUnhandledRejection(e) {
  hasError.value = true
  errorMessage.value = e.reason?.message || '未处理的 Promise 拒绝'
}

onMounted(() => {
  window.addEventListener('error', onGlobalError)
  window.addEventListener('unhandledrejection', onUnhandledRejection)
})

onUnmounted(() => {
  window.removeEventListener('error', onGlobalError)
  window.removeEventListener('unhandledrejection', onUnhandledRejection)
})

function reset() {
  hasError.value = false
  errorMessage.value = ''
  errorKey.value++
}
</script>
