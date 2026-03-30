<template>
  <div class="w-64 bg-gray-850 border-r border-gray-700 flex flex-col overflow-hidden" style="width: var(--sidebar-width); background: #252526;">
    <!-- Header -->
    <div class="p-3 flex items-center justify-between border-b border-gray-700">
      <span class="text-xs font-medium text-gray-400 uppercase tracking-wider">连接</span>
      <button @click="$emit('add')" class="text-gray-400 hover:text-white text-lg leading-none" title="新建连接">+</button>
    </div>

    <!-- Connection List -->
    <div class="flex-1 overflow-y-auto p-2">
      <div v-for="group in groupedConnections" :key="group.name" class="mb-2">
        <div class="text-xs text-gray-500 px-2 py-1 flex items-center gap-1">
          <span>📁</span> {{ group.name }}
        </div>
        <div
          v-for="conn in group.items"
          :key="conn.id"
          @click="$emit('select', conn)"
          class="flex items-center gap-2 px-2 py-1.5 rounded cursor-pointer text-sm"
          :class="activeId === conn.id ? 'bg-blue-600/30 text-blue-300' : 'text-gray-300 hover:bg-gray-700'"
        >
          <span>{{ conn.icon || '🖥️' }}</span>
          <span class="truncate flex-1">{{ conn.name }}</span>
          <button
            v-if="conn.id !== 'local'"
            @click.stop="$emit('delete', conn.id)"
            class="text-gray-600 hover:text-red-400 text-xs opacity-0 group-hover:opacity-100"
          >✕</button>
        </div>
      </div>
    </div>

    <!-- Quick Commands -->
    <div class="border-t border-gray-700 p-2">
      <div class="text-xs text-gray-500 px-2 py-1">快捷命令</div>
      <div class="flex flex-wrap gap-1 px-1">
        <button
          v-for="cmd in quickCommands"
          :key="cmd"
          class="text-xs px-2 py-0.5 bg-gray-700 hover:bg-gray-600 rounded text-gray-300"
        >{{ cmd }}</button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue'

const props = defineProps({
  connections: Array,
  activeId: String,
})

defineEmits(['select', 'add', 'delete'])

const quickCommands = ['top', 'htop', 'df -h', 'free -h', 'docker ps', 'tail -f']

const groupedConnections = computed(() => {
  const groups = {}
  for (const conn of props.connections) {
    const g = conn.group || '默认'
    if (!groups[g]) groups[g] = { name: g, items: [] }
    groups[g].items.push(conn)
  }
  return Object.values(groups)
})
</script>
