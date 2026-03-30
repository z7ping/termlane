<template>
  <div class="h-9 bg-gray-800 border-b border-gray-700 flex items-center overflow-x-auto">
    <div
      v-for="tab in tabs"
      :key="tab.id"
      @click="$emit('select', tab.id)"
      class="flex items-center gap-1.5 px-3 h-full cursor-pointer text-xs border-r border-gray-700 select-none min-w-0 max-w-48"
      :class="activeId === tab.id ? 'bg-gray-900 text-white border-t-2 border-t-blue-500' : 'text-gray-400 hover:bg-gray-750'"
    >
      <span class="truncate">{{ tab.name }}</span>
      <button
        @click.stop="$emit('close', tab.id)"
        class="text-gray-500 hover:text-white hover:bg-gray-600 rounded w-4 h-4 flex items-center justify-center leading-none"
      >✕</button>
    </div>
    <button
      @click="$emit('new')"
      class="px-2 h-full text-gray-500 hover:text-white hover:bg-gray-700 text-lg leading-none"
      title="新终端"
    >+</button>
  </div>
</template>

<script setup>
defineProps({
  tabs: Array,
  activeId: String,
})
defineEmits(['select', 'close', 'new'])
</script>
