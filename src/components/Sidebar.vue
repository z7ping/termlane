<template>
  <div class="w-64 bg-gray-850 border-r border-gray-700 flex flex-col overflow-hidden" :style="{ width: sidebarWidth + 'px' }" style="background: #252526;">
    <!-- Header -->
    <div class="p-3 flex items-center justify-between border-b border-gray-700">
      <span class="text-xs font-medium text-gray-400 uppercase tracking-wider">连接</span>
      <button @click="$emit('add')" class="text-gray-400 hover:text-white text-lg leading-none" title="新建连接">+</button>
    </div>

    <!-- Search -->
    <div class="px-2 py-1 border-b border-gray-700">
      <input
        v-model="searchQuery"
        class="w-full bg-gray-900 text-xs text-gray-300 px-2 py-1 rounded border border-gray-600 focus:outline-none focus:border-blue-500"
        placeholder="🔍 搜索连接..."
      />
    </div>

    <!-- Connection List -->
    <div class="flex-1 overflow-y-auto p-2">
      <div v-for="group in filteredGroups" :key="group.name" class="mb-2">
        <div class="text-xs text-gray-500 px-2 py-1 flex items-center gap-1 cursor-pointer" @click="toggleGroup(group.name)">
          <span class="transition-transform text-[10px]" :class="collapsedGroups[group.name] ? '-rotate-90' : ''">▼</span>
          <span>{{ group.name }}</span>
          <span class="text-gray-600 ml-auto">{{ group.items.length }}</span>
        </div>
        <div v-show="!collapsedGroups[group.name]">
          <div
            v-for="conn in group.items"
            :key="conn.id"
            @click="$emit('select', conn)"
            @contextmenu.prevent="showContextMenu($event, conn)"
            class="flex items-center gap-2 px-2 py-1.5 rounded cursor-pointer text-sm group relative"
            :class="activeId === conn.id ? 'bg-blue-600/30 text-blue-300' : 'text-gray-300 hover:bg-gray-700'"
          >
            <span>{{ conn.icon || '🖥️' }}</span>
            <span class="truncate flex-1">{{ conn.name }}</span>
            <span v-if="conn.host !== 'localhost'" class="text-[10px] text-gray-500">{{ conn.host }}</span>
          </div>
        </div>
      </div>
      <div v-if="filteredGroups.length === 0" class="text-center text-gray-500 text-xs py-4">
        {{ searchQuery ? '没有匹配的连接' : '暂无连接' }}
      </div>
    </div>

    <!-- Quick Commands -->
    <div class="border-t border-gray-700 p-2">
      <div class="text-xs text-gray-500 px-2 py-1">快捷命令</div>
      <div class="flex flex-wrap gap-1 px-1">
        <button v-for="cmd in quickCommands" :key="cmd" @click="$emit('quick-command', cmd)" class="text-xs px-2 py-0.5 bg-gray-700 hover:bg-gray-600 rounded text-gray-300">{{ cmd }}</button>
      </div>
    </div>

    <!-- Resize Handle -->
    <div
      class="absolute right-0 top-0 bottom-0 w-1 cursor-col-resize hover:bg-blue-500/50"
      @mousedown="startResize"
    />

    <!-- Context Menu -->
    <div v-if="contextMenu.show" class="fixed z-50 bg-gray-800 border border-gray-600 rounded shadow-lg py-1 min-w-[140px]" :style="{ left: contextMenu.x + 'px', top: contextMenu.y + 'px' }">
      <button @click="onCtxAction('edit')" class="w-full px-3 py-1.5 text-sm text-gray-300 hover:bg-gray-700 text-left">✏️ 编辑</button>
      <button @click="onCtxAction('duplicate')" class="w-full px-3 py-1.5 text-sm text-gray-300 hover:bg-gray-700 text-left">📋 复制</button>
      <button @click="onCtxAction('test')" class="w-full px-3 py-1.5 text-sm text-gray-300 hover:bg-gray-700 text-left">🔗 测试连接</button>
      <div class="border-t border-gray-700 my-1" />
      <button @click="onCtxAction('delete')" class="w-full px-3 py-1.5 text-sm text-red-400 hover:bg-gray-700 text-left">🗑 删除</button>
    </div>
    <div v-if="contextMenu.show" class="fixed inset-0 z-40" @click="contextMenu.show = false" />
  </div>
</template>

<script setup>
import { computed, reactive, ref } from 'vue'

const props = defineProps({
  connections: Array,
  activeId: String,
})

const emit = defineEmits(['select', 'add', 'delete', 'quick-command', 'edit', 'duplicate', 'test'])

const searchQuery = ref('')
const collapsedGroups = reactive({})
const sidebarWidth = ref(256)
const quickCommands = ['top', 'htop', 'df -h', 'free -h', 'docker ps', 'tail -f']

const contextMenu = reactive({ show: false, x: 0, y: 0, conn: null })

const filteredGroups = computed(() => {
  const groups = {}
  for (const conn of props.connections) {
    const g = conn.group || '默认'
    if (searchQuery.value && !conn.name.toLowerCase().includes(searchQuery.value.toLowerCase()) && !conn.host.includes(searchQuery.value)) continue
    if (!groups[g]) groups[g] = { name: g, items: [] }
    groups[g].items.push(conn)
  }
  return Object.values(groups)
})

function toggleGroup(name) { collapsedGroups[name] = !collapsedGroups[name] }

function showContextMenu(e, conn) {
  contextMenu.show = true
  contextMenu.x = e.clientX
  contextMenu.y = e.clientY
  contextMenu.conn = conn
}

function onCtxAction(action) {
  const conn = contextMenu.conn
  contextMenu.show = false
  if (!conn) return
  if (action === 'edit') emit('edit', conn)
  else if (action === 'duplicate') emit('duplicate', conn)
  else if (action === 'test') emit('test', conn)
  else if (action === 'delete') emit('delete', conn.id)
}

let resizing = false
function startResize(e) {
  resizing = true
  const startX = e.clientX
  const startW = sidebarWidth.value
  const onMove = (e) => { if (resizing) sidebarWidth.value = Math.max(180, Math.min(500, startW + e.clientX - startX)) }
  const onUp = () => { resizing = false; document.removeEventListener('mousemove', onMove); document.removeEventListener('mouseup', onUp) }
  document.addEventListener('mousemove', onMove)
  document.addEventListener('mouseup', onUp)
}
</script>
