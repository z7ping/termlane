<template>
  <div
    class="relative inline-block"
    @mouseenter="onMouseEnter"
    @mouseleave="onMouseLeave"
  >
    <slot />
    <transition name="tooltip">
      <div
        v-if="show"
        class="absolute z-50 px-2 py-1 text-xs rounded shadow-lg whitespace-nowrap"
        :class="positionClass"
        :style="positionStyle"
      >
        {{ text }}
        <div class="tooltip-arrow" :class="arrowClass" />
      </div>
    </transition>
  </div>
</template>

<script setup>
import { ref, computed, onUnmounted } from 'vue'

const props = defineProps({
  text: String,
  position: { type: String, default: 'top' },
  delay: { type: Number, default: 300 },
})

const show = ref(false)
let showTimer = null
let hideTimer = null

function onMouseEnter() {
  clearTimeout(hideTimer)
  showTimer = setTimeout(() => { show.value = true }, props.delay)
}

function onMouseLeave() {
  clearTimeout(showTimer)
  hideTimer = setTimeout(() => { show.value = false }, 100)
}

onUnmounted(() => {
  clearTimeout(showTimer)
  clearTimeout(hideTimer)
})

const positionClass = computed(() => ({
  top: 'bottom-full left-1/2 -translate-x-1/2 mb-2',
  bottom: 'top-full left-1/2 -translate-x-1/2 mt-2',
  left: 'right-full top-1/2 -translate-y-1/2 mr-2',
  right: 'left-full top-1/2 -translate-y-1/2 ml-2',
}[props.position] + ' bg-gray-700 text-gray-200'))

const arrowClass = computed(() => ({
  top: 'border-t-gray-700 border-l-transparent border-r-transparent border-b-transparent bottom-[-4px] left-1/2 -translate-x-1/2',
  bottom: 'border-b-gray-700 border-l-transparent border-r-transparent border-t-transparent top-[-4px] left-1/2 -translate-x-1/2',
  left: 'border-l-gray-700 border-t-transparent border-b-transparent border-r-transparent right-[-4px] top-1/2 -translate-y-1/2',
  right: 'border-r-gray-700 border-t-transparent border-b-transparent border-l-transparent left-[-4px] top-1/2 -translate-y-1/2',
}[props.position]))

const positionStyle = computed(() => ({}))
</script>

<style scoped>
.tooltip-arrow {
  position: absolute;
  width: 0;
  height: 0;
  border-style: solid;
  border-width: 4px;
}
.tooltip-enter-active, .tooltip-leave-active { transition: opacity 0.15s; }
.tooltip-enter-from, .tooltip-leave-to { opacity: 0; }
</style>
