<template>
  <slot v-if="!hasError" />
  <div v-else class="h-full flex items-center justify-center bg-gray-900">
    <div class="text-center p-8">
      <div class="text-5xl mb-4">😵</div>
      <div class="text-lg text-gray-200 mb-2">组件出错了</div>
      <div class="text-sm text-gray-500 mb-4">{{ errorMessage }}</div>
      <button @click="reset" class="px-4 py-2 bg-blue-600 hover:bg-blue-500 rounded text-white text-sm">重试</button>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted, onErrorCaptured } from 'vue'

const hasError = ref(false)
const errorMessage = ref('')

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
}
</script>
