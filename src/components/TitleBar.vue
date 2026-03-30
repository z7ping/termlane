<template>
  <div class="h-9 bg-gray-800 border-b border-gray-700 flex items-center px-3 select-none drag-region">
    <button @click="$emit('toggle-sidebar')" class="mr-3 text-gray-400 hover:text-white p-1 no-drag">
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16"/></svg>
    </button>
    <span class="text-sm font-medium text-gray-300">XTerminal Pro</span>
    <div class="flex-1" />
    <div class="flex gap-2 text-gray-500 text-xs">
      <span class="hover:text-gray-300 cursor-pointer">⚙️</span>
    </div>
  </div>
</template>

<script setup>
defineEmits(['toggle-sidebar'])
</script>

<style scoped>
.drag-region { -webkit-app-region: drag; }
.no-drag { -webkit-app-region: no-drag; }
</style>
