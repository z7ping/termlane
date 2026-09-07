<template>
  <div
    class="tooltip-anchor"
    @mouseenter="scheduleShow"
    @mouseleave="scheduleHide"
    @focusin="scheduleShow"
    @focusout="scheduleHide"
  >
    <slot />
    <Transition name="tooltip">
      <div
        v-if="show && text"
        class="tooltip-bubble"
        :class="`tooltip-${normalizedPosition}`"
        role="tooltip"
      >
        {{ text }}
        <span class="tooltip-arrow" aria-hidden="true" />
      </div>
    </Transition>
  </div>
</template>

<script setup>
import { computed, onUnmounted, ref } from 'vue'

const props = defineProps({
  text: { type: String, default: '' },
  position: { type: String, default: 'top' },
  delay: { type: Number, default: 300 },
})

const show = ref(false)
let showTimer = null
let hideTimer = null

const normalizedPosition = computed(() => (
  ['top', 'bottom', 'left', 'right'].includes(props.position) ? props.position : 'top'
))

function clearTimers() {
  if (showTimer) window.clearTimeout(showTimer)
  if (hideTimer) window.clearTimeout(hideTimer)
  showTimer = null
  hideTimer = null
}

function scheduleShow() {
  if (hideTimer) window.clearTimeout(hideTimer)
  showTimer = window.setTimeout(() => {
    show.value = true
    showTimer = null
  }, Math.max(0, props.delay))
}

function scheduleHide() {
  if (showTimer) window.clearTimeout(showTimer)
  hideTimer = window.setTimeout(() => {
    show.value = false
    hideTimer = null
  }, 80)
}

onUnmounted(clearTimers)
</script>

<style scoped>
.tooltip-anchor {
  position: relative;
  display: inline-flex;
}

.tooltip-bubble {
  position: absolute;
  z-index: 70;
  max-width: min(280px, calc(100vw - 20px));
  padding: 5px 7px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--bg-elevated);
  color: var(--fg-primary);
  box-shadow: var(--shadow-md);
  font-size: 11px;
  line-height: 1.35;
  white-space: nowrap;
  pointer-events: none;
}

.tooltip-top { bottom: calc(100% + 8px); left: 50%; transform: translateX(-50%); }
.tooltip-bottom { top: calc(100% + 8px); left: 50%; transform: translateX(-50%); }
.tooltip-left { right: calc(100% + 8px); top: 50%; transform: translateY(-50%); }
.tooltip-right { left: calc(100% + 8px); top: 50%; transform: translateY(-50%); }

.tooltip-arrow {
  position: absolute;
  width: 7px;
  height: 7px;
  background: var(--bg-elevated);
  transform: rotate(45deg);
}

.tooltip-top .tooltip-arrow { bottom: -4px; left: calc(50% - 4px); border-right: 1px solid var(--border); border-bottom: 1px solid var(--border); }
.tooltip-bottom .tooltip-arrow { top: -4px; left: calc(50% - 4px); border-left: 1px solid var(--border); border-top: 1px solid var(--border); }
.tooltip-left .tooltip-arrow { right: -4px; top: calc(50% - 4px); border-right: 1px solid var(--border); border-top: 1px solid var(--border); }
.tooltip-right .tooltip-arrow { left: -4px; top: calc(50% - 4px); border-left: 1px solid var(--border); border-bottom: 1px solid var(--border); }

.tooltip-enter-active,
.tooltip-leave-active {
  transition: opacity var(--transition-fast), transform var(--transition-fast);
}
.tooltip-enter-from,
.tooltip-leave-to { opacity: 0; }
</style>
