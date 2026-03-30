<template>
  <div class="h-9 bg-gray-800 border-b border-gray-700 flex items-center overflow-x-auto">
    <div
      v-for="tab in tabs"
      :key="tab.id"
      @click="$emit('select', tab.id)"
      @dblclick="$emit('close', tab.id)"
      @contextmenu.prevent="showContextMenu($event, tab)"
      class="flex items-center gap-1.5 px-3 h-full cursor-pointer text-xs border-r border-gray-700 select-none min-w-0 max-w-48"
      :class="activeId === tab.id ? 'bg-gray-900 text-white border-t-2 border-t-blue-500' : 'text-gray-400 hover:bg-gray-750'"
    >
      <span class="truncate">{{ tab.name }}</span>
      <button @click.stop="$emit('close', tab.id)" class="text-gray-500 hover:text-white hover:bg-gray-600 rounded w-4 h-4 flex items-center justify-center leading-none">✕</button>
    </div>
    <button @click="$emit('new')" class="px-2 h-full text-gray-500 hover:text-white hover:bg-gray-700 text-lg leading-none" title="新终端">+</button>

    <!-- Context Menu -->
    <div v-if="ctx.show" class="fixed z-50 bg-gray-800 border border-gray-600 rounded shadow-lg py-1 min-w-[140px]" :style="{ left: ctx.x + 'px', top: ctx.y + 'px' }">
      <button @click="onCtx('close')" class="w-full px-3 py-1.5 text-sm text-gray-300 hover:bg-gray-700 text-left">关闭</button>
      <button @click="onCtx('closeOthers')" class="w-full px-3 py-1.5 text-sm text-gray-300 hover:bg-gray-700 text-left">关闭其他</button>
      <button @click="onCtx('closeAll')" class="w-full px-3 py-1.5 text-sm text-gray-300 hover:bg-gray-700 text-left">关闭全部</button>
    </div>
    <div v-if="ctx.show" class="fixed inset-0 z-40" @click="ctx.show = false" />
  </div>
</template>

<script setup>
import { reactive } from 'vue'

defineProps({ tabs: Array, activeId: String })
const emit = defineEmits(['select', 'close', 'new', 'closeOthers', 'closeAll'])

const ctx = reactive({ show: false, x: 0, y: 0, tab: null })

function showContextMenu(e, tab) {
  ctx.show = true; ctx.x = e.clientX; ctx.y = e.clientY; ctx.tab = tab
}

function onCtx(action) {
  const tab = ctx.tab
  ctx.show = false
  if (!tab) return
  if (action === 'close') emit('close', tab.id)
  else if (action === 'closeOthers') emit('closeOthers', tab.id)
  else if (action === 'closeAll') emit('closeAll')
}
</script>
