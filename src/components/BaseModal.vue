<template>
  <Teleport to="body">
    <Transition name="modal">
      <div v-if="show" class="fixed inset-0 flex items-center justify-center z-50" style="background: color-mix(in srgb, #000 60%, transparent)" @click.self="$emit('close')">
        <div
          class="rounded-lg p-4 space-y-3" style="background: var(--bg-surface); border-color: var(--border-subtle)"
          :style="{ width: width }"
          @keydown.esc="$emit('close')"
        >
          <slot />
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup>
defineProps({
  show: Boolean,
  width: { type: String, default: '400px' },
})

defineEmits(['close'])
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
