<template>
  <div :key="errorKey" class="h-full">
    <slot v-if="!hasError" />
    <div v-else class="error-state">
      <CircleAlert :size="34" :stroke-width="1.5" />
      <div class="error-title">当前视图加载失败</div>
      <div class="error-message">{{ errorMessage }}</div>
      <button type="button" @click="reset">
        <RotateCcw :size="13" :stroke-width="1.8" />
        <span>重试当前视图</span>
      </button>
    </div>
  </div>
</template>

<script setup>
import { onErrorCaptured, ref } from 'vue'
import { CircleAlert, RotateCcw } from 'lucide-vue-next'

const hasError = ref(false)
const errorMessage = ref('')
const errorKey = ref(0)

onErrorCaptured(error => {
  hasError.value = true
  errorMessage.value = error?.message || '未知组件错误'
  console.error('[XTerminal] View render error:', error)
  return false
})

function reset() {
  hasError.value = false
  errorMessage.value = ''
  errorKey.value++
}
</script>

<style scoped>
.error-state { height: 100%; min-height: 240px; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 8px; padding: 24px; background: var(--bg-base); color: var(--danger); text-align: center; }
.error-title { color: var(--fg-primary); font-size: 14px; font-weight: 600; }
.error-message { max-width: 520px; color: var(--fg-muted); font-size: 10px; line-height: 1.5; overflow-wrap: anywhere; }
.error-state button { height: 30px; display: inline-flex; align-items: center; gap: 5px; margin-top: 4px; padding: 0 10px; border: 0; border-radius: 6px; background: var(--bg-hover); color: var(--fg-secondary); font-size: 11px; }
.error-state button:hover { color: var(--fg-primary); }
</style>
