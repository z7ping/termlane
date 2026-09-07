<template>
  <div class="toast-container" aria-live="polite" aria-atomic="false">
    <TransitionGroup name="toast">
      <div
        v-for="toast in toasts"
        :key="toast.id"
        class="toast-item"
        :class="`toast-${toast.type}`"
        role="status"
      >
        <span class="toast-icon" aria-hidden="true">
          <component :is="iconFor(toast.type)" :size="14" :stroke-width="1.9" />
        </span>
        <span class="toast-message">{{ toast.message }}</span>
      </div>
    </TransitionGroup>
  </div>
</template>

<script setup>
import { ref } from 'vue'
import { CheckCircle2, CircleAlert, Info, TriangleAlert } from 'lucide-vue-next'

const toasts = ref([])
const MAX_TOASTS = 5
const VALID_TYPES = new Set(['success', 'error', 'info', 'warning'])
const icons = {
  success: CheckCircle2,
  error: CircleAlert,
  warning: TriangleAlert,
  info: Info,
}

function iconFor(type) {
  return icons[type] || Info
}

function show(message, type = 'info', duration = 2500) {
  const normalizedType = VALID_TYPES.has(type) ? type : 'info'
  const id = crypto.randomUUID()
  toasts.value.push({ id, message: String(message ?? ''), type: normalizedType })

  if (toasts.value.length > MAX_TOASTS) {
    toasts.value = toasts.value.slice(-MAX_TOASTS)
  }

  window.setTimeout(() => {
    toasts.value = toasts.value.filter(toast => toast.id !== id)
  }, Math.max(500, duration))
}

defineExpose({ show })
</script>

<style scoped>
.toast-container { position: fixed; top: 12px; right: 12px; z-index: 9999; display: flex; flex-direction: column; gap: 6px; pointer-events: none; }
.toast-item { min-width: 220px; max-width: min(380px, calc(100vw - 24px)); display: flex; align-items: flex-start; gap: 8px; padding: 8px 10px; border: 1px solid var(--border); border-left-width: 3px; border-radius: 7px; background: var(--bg-elevated); color: var(--fg-secondary); box-shadow: var(--shadow-md); font-size: 11px; line-height: 1.45; }
.toast-icon { width: 22px; height: 22px; display: inline-flex; align-items: center; justify-content: center; flex-shrink: 0; border-radius: 999px; }
.toast-message { flex: 1; min-width: 0; padding-top: 3px; overflow-wrap: anywhere; }
.toast-success { border-left-color: var(--success); }
.toast-success .toast-icon { background: color-mix(in srgb, var(--success) 14%, transparent); color: var(--success); }
.toast-error { border-left-color: var(--danger); }
.toast-error .toast-icon { background: color-mix(in srgb, var(--danger) 14%, transparent); color: var(--danger); }
.toast-warning { border-left-color: var(--warning); }
.toast-warning .toast-icon { background: color-mix(in srgb, var(--warning) 14%, transparent); color: var(--warning); }
.toast-info { border-left-color: var(--accent); }
.toast-info .toast-icon { background: var(--accent-hover); color: var(--accent); }
.toast-enter-active { transition: transform 180ms ease-out, opacity 180ms ease-out; }
.toast-leave-active { transition: transform 140ms ease-in, opacity 140ms ease-in; }
.toast-enter-from, .toast-leave-to { transform: translateX(16px); opacity: 0; }
.toast-move { transition: transform 160ms ease; }
</style>
