<!-- VirtualList.vue - 高性能虚拟滚动列表 -->
<template>
  <div ref="containerRef" class="overflow-y-auto" @scroll="onScroll" :style="{ height: height + 'px' }">
    <div :style="{ height: totalHeight + 'px', position: 'relative' }">
      <div v-for="item in visibleItems" :key="item.key" :style="{ position: 'absolute', top: item.top + 'px', left: 0, right: 0, height: itemHeight + 'px' }">
        <slot :item="item.data" :index="item.index" />
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, watch, onMounted, nextTick } from 'vue'

const props = defineProps({
  items: { type: Array, default: () => [] },
  itemHeight: { type: Number, default: 32 },
  height: { type: Number, default: 400 },
  buffer: { type: Number, default: 5 },
})

const containerRef = ref(null)
const scrollTop = ref(0)

const totalHeight = computed(() => props.items.length * props.itemHeight)

const visibleRange = computed(() => {
  const start = Math.max(0, Math.floor(scrollTop.value / props.itemHeight) - props.buffer)
  const visibleCount = Math.ceil(props.height / props.itemHeight) + props.buffer * 2
  const end = Math.min(props.items.length, start + visibleCount)
  return { start, end }
})

const visibleItems = computed(() => {
  const { start, end } = visibleRange.value
  const result = []
  for (let i = start; i < end; i++) {
    result.push({
      key: i,
      index: i,
      data: props.items[i],
      top: i * props.itemHeight,
    })
  }
  return result
})

function onScroll(e) {
  scrollTop.value = e.target.scrollTop
}

// Reset scroll when items change
watch(() => props.items.length, () => {
  nextTick(() => {
    if (containerRef.value) scrollTop.value = containerRef.value.scrollTop
  })
})
</script>
