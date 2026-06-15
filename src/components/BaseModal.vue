<template>
  <Teleport to="body">
    <Transition name="modal">
      <div v-if="show" class="fixed inset-0 flex items-center justify-center z-50" style="background: color-mix(in srgb, #000 60%, transparent)" @click.self="$emit('close')" @keydown="handleKeydown">
        <div
          ref="modalRef"
          class="rounded-lg p-4 space-y-3"
          role="dialog"
          aria-modal="true"
          :aria-label="title || '对话框'"
          style="background: var(--bg-surface); border-color: var(--border-subtle)"
          :style="{ width: width }"
        >
          <slot />
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup>
import { ref, watch, nextTick } from 'vue'

const props = defineProps({
  show: Boolean,
  width: { type: String, default: '400px' },
  title: { type: String, default: '' },
})

const emit = defineEmits(['close'])

const modalRef = ref(null)

function handleKeydown(e) {
  if (e.key === 'Escape') {
    e.stopPropagation()
    emit('close')
    return
  }
  if (e.key === 'Tab') {
    trapFocus(e)
  }
}

function trapFocus(e) {
  const modal = modalRef.value
  if (!modal) return
  const focusable = modal.querySelectorAll('button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])')
  if (focusable.length === 0) return
  const first = focusable[0]
  const last = focusable[focusable.length - 1]
  if (e.shiftKey) {
    if (document.activeElement === first) {
      e.preventDefault()
      last.focus()
    }
  } else {
    if (document.activeElement === last) {
      e.preventDefault()
      first.focus()
    }
  }
}

// Focus first focusable element when modal opens
watch(() => props.show, (val) => {
  if (val) {
    nextTick(() => {
      const modal = modalRef.value
      if (!modal) return
      const focusable = modal.querySelectorAll('button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])')
      if (focusable.length > 0) focusable[0].focus()
    })
  }
})
</script>

<style scoped>
.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.2s ease;
}
.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}
</style>
