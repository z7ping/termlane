<template>
  <BaseModal :show="show" :width="width" :title="title" @close="$emit('close')">
    <div class="confirm-dialog">
      <div class="dialog-title">{{ title }}</div>
      <p>{{ message }}</p>
      <div class="dialog-actions">
        <button type="button" class="secondary-button" @click="$emit('close')">{{ cancelLabel }}</button>
        <button
          type="button"
          :class="danger ? 'danger-button' : 'primary-button'"
          @click="$emit('confirm')"
        >
          {{ confirmLabel }}
        </button>
      </div>
    </div>
  </BaseModal>
</template>

<script setup>
import BaseModal from './BaseModal.vue'

defineProps({
  show: Boolean,
  title: { type: String, default: '确认操作' },
  message: { type: String, required: true },
  confirmLabel: { type: String, default: '确定' },
  cancelLabel: { type: String, default: '取消' },
  danger: Boolean,
  width: { type: String, default: '360px' },
})

defineEmits(['close', 'confirm'])
</script>

<style scoped>
.dialog-title {
  color: var(--fg-primary);
  font-size: 14px;
  font-weight: 600;
}

.confirm-dialog p {
  margin: 8px 0 0;
  color: var(--fg-secondary);
  font-size: 12px;
  line-height: 1.6;
}

.dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 18px;
}

.secondary-button,
.primary-button,
.danger-button {
  height: 30px;
  padding: 0 11px;
  border: 0;
  border-radius: var(--radius-sm);
  font-size: 11px;
}

.secondary-button {
  background: var(--bg-hover);
  color: var(--fg-secondary);
}

.primary-button {
  background: var(--accent);
  color: white;
}

.danger-button {
  background: var(--danger);
  color: white;
}
</style>
