<template>
  <div class="fixed top-4 right-4 z-50 space-y-2">
    <transition-group name="toast">
      <div
        v-for="t in toasts"
        :key="t.id"
        class="px-4 py-2 rounded shadow-lg text-sm flex items-center gap-2 min-w-[200px]"
        :class="toastClass(t.type)"
      >
        <span>{{ icon(t.type) }}</span>
        <span>{{ t.message }}</span>
      </div>
    </transition-group>
  </div>
</template>

<script setup>
import { ref, provide } from 'vue'

const toasts = ref([])

function icon(type) {
  return { success: '✓', error: '✗', info: 'ℹ', warning: '⚠' }[type] || 'ℹ'
}

function toastClass(type) {
  return {
    success: 'bg-green-600 text-white',
    error: 'bg-red-600 text-white',
    info: 'bg-gray-700 text-gray-200',
    warning: 'bg-yellow-600 text-white',
  }[type] || 'bg-gray-700 text-gray-200'
}

function show(message, type = 'info', duration = 3000) {
  const id = Date.now()
  toasts.value.push({ id, message, type })
  setTimeout(() => {
    toasts.value = toasts.value.filter(t => t.id !== id)
  }, duration)
}

provide('toast', { show })
defineExpose({ show })
</script>

<style scoped>
.toast-enter-active, .toast-leave-active { transition: all 0.3s ease; }
.toast-enter-from { transform: translateX(100%); opacity: 0; }
.toast-leave-to { transform: translateX(100%); opacity: 0; }
</style>
