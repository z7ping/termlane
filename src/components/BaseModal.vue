<template>
  <Teleport to="body">
    <Transition name="modal">
      <div v-if="show" class="modal-backdrop" @click.self="$emit('close')" @keydown="handleKeydown">
        <div
          ref="modalRef"
          class="modal-panel"
          role="dialog"
          aria-modal="true"
          :aria-label="title || '对话框'"
          :style="{ width }"
          tabindex="-1"
        >
          <slot />
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup>
import { nextTick, ref, watch } from 'vue'

const props = defineProps({
  show: Boolean,
  width: { type: String, default: '400px' },
  title: { type: String, default: '' },
})

const emit = defineEmits(['close'])
const modalRef = ref(null)
let previousFocus = null

function focusableElements() {
  const modal = modalRef.value
  if (!modal) return []
  return [...modal.querySelectorAll(
    'button:not(:disabled), [href], input:not(:disabled), select:not(:disabled), textarea:not(:disabled), [tabindex]:not([tabindex="-1"])',
  )].filter(element => !element.hasAttribute('hidden'))
}

function handleKeydown(event) {
  if (event.key === 'Escape') {
    event.stopPropagation()
    emit('close')
    return
  }
  if (event.key !== 'Tab') return

  const focusable = focusableElements()
  if (focusable.length === 0) {
    event.preventDefault()
    modalRef.value?.focus()
    return
  }

  const first = focusable[0]
  const last = focusable.at(-1)
  if (event.shiftKey && document.activeElement === first) {
    event.preventDefault()
    last.focus()
  } else if (!event.shiftKey && document.activeElement === last) {
    event.preventDefault()
    first.focus()
  }
}

watch(() => props.show, async show => {
  if (show) {
    previousFocus = document.activeElement
    await nextTick()
    const focusable = focusableElements()
    ;(focusable[0] || modalRef.value)?.focus()
  } else if (previousFocus instanceof HTMLElement && document.contains(previousFocus)) {
    await nextTick()
    previousFocus.focus()
    previousFocus = null
  }
})
</script>

<style scoped>
.modal-backdrop { position: fixed; inset: 0; z-index: 60; display: flex; align-items: center; justify-content: center; padding: 20px; background: rgba(0, 0, 0, 0.62); }
.modal-panel { max-width: 100%; max-height: calc(100vh - 40px); overflow-y: auto; padding: 14px; border: 1px solid var(--border); border-radius: 9px; outline: none; background: var(--bg-elevated); color: var(--fg-secondary); box-shadow: var(--shadow-lg); }
.modal-enter-active, .modal-leave-active { transition: opacity 160ms ease; }
.modal-enter-active .modal-panel, .modal-leave-active .modal-panel { transition: transform 160ms ease, opacity 160ms ease; }
.modal-enter-from, .modal-leave-to { opacity: 0; }
.modal-enter-from .modal-panel, .modal-leave-to .modal-panel { opacity: 0; transform: translateY(4px) scale(0.99); }
</style>
