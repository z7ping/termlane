<template>
  <Teleport to="body">
    <Transition name="modal">
      <div v-if="show" class="fixed inset-0 bg-black/60 flex items-center justify-center z-50" @click.self="$emit('close')">
        <div
          class="bg-gray-800 rounded-lg border border-gray-600 p-4 space-y-3"
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
