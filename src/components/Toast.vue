<template>
  <div class="toast-container">
    <transition-group name="toast">
      <div
        v-for="t in toasts"
        :key="t.id"
        class="toast-item"
        :class="'toast-' + t.type"
      >
        <span class="toast-icon">{{ iconMap[t.type] || iconMap.info }}</span>
        <span class="toast-msg">{{ t.message }}</span>
      </div>
    </transition-group>
  </div>
</template>

<script setup>
import { ref, provide } from 'vue'

const toasts = ref([])

const iconMap = {
  success: '✓',
  error: '✕',
  info: 'ℹ',
  warning: '⚠',
}

function show(message, type = 'info', duration = 3000) {
  const id = Date.now() + Math.random()
  toasts.value.push({ id, message, type })
  setTimeout(() => {
    toasts.value = toasts.value.filter(t => t.id !== id)
  }, duration)
}

provide('toast', { show })
defineExpose({ show })
</script>

<style scoped>
.toast-container {
  position: fixed;
  top: 1rem;
  right: 1rem;
  z-index: 9999;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.toast-item {
  display: flex;
  align-items: center;
  gap: 0.625rem;
  padding: 0.625rem 1rem;
  min-width: 220px;
  max-width: 380px;
  font-size: 0.875rem;
  line-height: 1.4;
  border-radius: var(--radius);
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  color: #e5e7eb;
  box-shadow: 0 4px 24px rgba(0, 0, 0, 0.35);
  backdrop-filter: blur(8px);
}

.toast-icon {
  flex-shrink: 0;
  width: 1.5rem;
  height: 1.5rem;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  font-size: 0.8rem;
  font-weight: 700;
}

.toast-msg {
  flex: 1;
  word-break: break-word;
}

/* --- type accents --- */
.toast-success .toast-icon {
  background: color-mix(in oklch, var(--success) 25%, transparent);
  color: var(--success);
}
.toast-success {
  border-left: 3px solid var(--success);
}

.toast-error .toast-icon {
  background: color-mix(in oklch, var(--danger) 25%, transparent);
  color: var(--danger);
}
.toast-error {
  border-left: 3px solid var(--danger);
}

.toast-warning .toast-icon {
  background: color-mix(in oklch, var(--warning) 25%, transparent);
  color: var(--warning);
}
.toast-warning {
  border-left: 3px solid var(--warning);
}

.toast-info .toast-icon {
  background: color-mix(in oklch, var(--accent) 25%, transparent);
  color: var(--accent);
}
.toast-info {
  border-left: 3px solid var(--accent);
}

/* --- slide-in / slide-out animation (from right) --- */
.toast-enter-active {
  transition: transform 0.35s cubic-bezier(0.16, 1, 0.3, 1),
              opacity 0.35s cubic-bezier(0.16, 1, 0.3, 1);
}
.toast-leave-active {
  transition: transform 0.25s cubic-bezier(0.55, 0, 1, 0.45),
              opacity 0.25s cubic-bezier(0.55, 0, 1, 0.45);
}

.toast-enter-from {
  transform: translateX(110%);
  opacity: 0;
}
.toast-leave-to {
  transform: translateX(110%);
  opacity: 0;
}

/* keep list reflow smooth */
.toast-move {
  transition: transform 0.3s ease;
}
</style>
