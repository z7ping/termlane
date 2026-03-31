<template>
  <div class="relative flex flex-col overflow-hidden select-none" :style="{ width: sidebarWidth + 'px' }" style="background: #252526;">
    <!-- Header -->
    <div class="px-3 py-2.5 flex items-center justify-between border-b" style="border-color: #333;">
      <span class="text-[10px] font-semibold text-gray-500 uppercase tracking-widest">连接</span>
      <div class="flex items-center gap-1.5">
        <span class="text-[10px] text-gray-600 px-1.5 py-0.5 rounded" style="background: #1e1e1e;">{{ connections.length }}</span>
        <button @click="$emit('add')" class="w-5 h-5 flex items-center justify-center text-gray-500 hover:text-white hover:bg-gray-700 rounded text-sm transition-colors" title="添加连接">+</button>
      </div>
    </div>

    <!-- Search -->
    <div class="px-2 py-1.5 border-b" style="border-color: #333;">
      <input v-model="searchQuery" class="w-full text-[11px] text-gray-300 px-2 py-1 rounded border focus:outline-none focus:border-blue-500/50 transition-colors placeholder-gray-600" style="background: #1a1a1a; border-color: #333;" placeholder="🔍 搜索..." />
    </div>

    <!-- Tree -->
    <div class="flex-1 overflow-y-auto py-1">
      <template v-for="node in tree" :key="node.key">
        <TreeItem :node="node" :active-id="activeId" :depth="0"
          @select="$emit('select', $event)" @toggle="toggleNode" @ctx="onCtxEvent" />
      </template>
      <div v-if="tree.length === 0" class="text-center py-10">
        <div class="text-2xl mb-2 opacity-30">{{ searchQuery ? '🔍' : '🔌' }}</div>
        <div class="text-[11px] text-gray-600">{{ searchQuery ? '没有匹配' : '点击 + 添加连接' }}</div>
      </div>
    </div>

    <!-- Quick Commands -->
    <div class="border-t px-2 py-2" style="border-color: #333;">
      <div class="text-[9px] text-gray-600 px-1 mb-1.5 uppercase tracking-wider">快捷命令</div>
      <div class="flex flex-wrap gap-1">
        <button v-for="cmd in quickCommands" :key="cmd.l" @click="$emit('quick-command', cmd.c)"
          class="text-[10px] px-1.5 py-0.5 rounded text-gray-500 hover:text-gray-300 hover:bg-gray-800 transition-colors font-mono" style="background: #1e1e1e;">
          {{ cmd.l }}
        </button>
      </div>
    </div>

    <!-- Resize -->
    <div class="absolute right-0 top-0 bottom-0 w-0.5 cursor-col-resize hover:bg-blue-500/50" @mousedown="startResize" />

    <!-- Context Menu -->
    <div v-if="ctx.show" class="fixed z-50 rounded-lg shadow-2xl py-1 min-w-[150px]" style="background: #2a2a2a; border: 1px solid #444;" :style="{ left: ctx.x + 'px', top: ctx.y + 'px' }">
      <button @click="doCtx('select')" class="ctx-item">📂 连接</button>
      <button @click="doCtx('edit')" class="ctx-item">✏️ 编辑</button>
      <button @click="doCtx('duplicate')" class="ctx-item">📋 复制</button>
      <button @click="doCtx('test')" class="ctx-item">🔗 测试</button>
      <div class="border-t my-1" style="border-color: #444;" />
      <button @click="doCtx('fav')" class="ctx-item">⭐ {{ ctx.conn?.favorite ? '取消收藏' : '收藏' }}</button>
      <div class="border-t my-1" style="border-color: #444;" />
      <button @click="doCtx('delete')" class="ctx-item text-red-400">🗑 删除</button>
    </div>
    <div v-if="ctx.show" class="fixed inset-0 z-40" @click="ctx.show = false" />
  </div>
</template>

<script setup>
import { computed, reactive, ref, h, defineComponent } from 'vue'

const props = defineProps({ connections: Array, activeId: String })
const emit = defineEmits(['select', 'add', 'delete', 'quick-command', 'edit', 'duplicate', 'test', 'favorite'])

const searchQuery = ref('')
const collapsed = reactive({})
const sidebarWidth = ref(240)
const quickCommands = [
  { l: 'top', c: 'top' }, { l: 'df -h', c: 'df -h' }, { l: 'free', c: 'free -h' },
  { l: 'docker', c: 'docker ps' }, { l: 'tail', c: 'tail -f /var/log/syslog' },
]
const ctx = reactive({ show: false, x: 0, y: 0, conn: null })

const colorMap = { red: '#ef4444', yellow: '#eab308', green: '#22c55e', blue: '#3b82f6', purple: '#a855f7' }
function colorHex(c) { return colorMap[c] || '#6b7280' }

// Build tree from flat connections with "/" separated groups
const tree = computed(() => {
  const filtered = props.connections.filter(c => {
    if (!searchQuery.value) return true
    const q = searchQuery.value.toLowerCase()
    return c.name.toLowerCase().includes(q) || c.host?.includes(q) || c.group?.toLowerCase().includes(q)
  })

  // Sort: favorites first, then by group, then by name
  filtered.sort((a, b) => {
    if (a.favorite && !b.favorite) return -1
    if (!a.favorite && b.favorite) return 1
    return (a.group || '').localeCompare(b.group || '') || a.name.localeCompare(b.name)
  })

  // Build tree
  const root = []
  const groupMap = {}

  for (const conn of filtered) {
    const parts = (conn.group || '默认').split('/')
    let currentChildren = root
    let currentPath = ''

    for (let i = 0; i < parts.length; i++) {
      const part = parts[i]
      currentPath = currentPath ? `${currentPath}/${part}` : part

      let groupNode = currentChildren.find(n => n.isGroup && n.label === part)
      if (!groupNode) {
        groupNode = {
          key: `g:${currentPath}`,
          isGroup: true,
          label: part,
          path: currentPath,
          icon: '📁',
          children: [],
          count: 0,
          collapsed: collapsed[currentPath] ?? false,
        }
        currentChildren.push(groupNode)
      }
      groupNode.count++
      currentChildren = groupNode.children
    }

    currentChildren.push({
      key: `c:${conn.id}`,
      isGroup: false,
      conn,
    })
  }

  return root
})

function toggleNode(path) {
  collapsed[path] = !collapsed[path]
}

function showCtx(e, conn) { ctx.show = true; ctx.x = Math.min(e.clientX, window.innerWidth - 160); ctx.y = Math.min(e.clientY, window.innerHeight - 200); ctx.conn = conn }
function onCtxEvent({ event, conn }) { showCtx(event, conn) }

function doCtx(action) {
  const c = ctx.conn; ctx.show = false; if (!c) return
  if (action === 'select') emit('select', c)
  else if (action === 'edit') emit('edit', c)
  else if (action === 'duplicate') emit('duplicate', c)
  else if (action === 'test') emit('test', c)
  else if (action === 'fav') emit('favorite', c)
  else if (action === 'delete') emit('delete', c.id)
}

let resizing = false
function startResize(e) {
  resizing = true; const startX = e.clientX; const startW = sidebarWidth.value
  const onMove = (e) => { if (resizing) sidebarWidth.value = Math.max(180, Math.min(400, startW + e.clientX - startX)) }
  const onUp = () => { resizing = false; document.removeEventListener('mousemove', onMove); document.removeEventListener('mouseup', onUp) }
  document.addEventListener('mousemove', onMove); document.addEventListener('mouseup', onUp)
}
</script>

<!-- TreeItem recursive component -->
<script>
import { defineComponent, h } from 'vue'

const TreeItem = defineComponent({
  name: 'TreeItem',
  props: { node: Object, activeId: String, depth: Number },
  emits: ['select', 'toggle', 'ctx'],
  setup(props, { emit }) {
    const indent = props.depth * 16
    return () => {
      const n = props.node
      if (n.isGroup) {
        return h('div', {}, [
          h('div', {
            onClick: () => emit('toggle', n.path),
            class: 'flex items-center gap-1.5 px-2 py-[5px] cursor-pointer hover:bg-white/5 transition-colors group',
            style: { paddingLeft: (8 + indent) + 'px' }
          }, [
            h('span', { class: `text-[8px] text-gray-600 transition-transform duration-150 ${n.collapsed ? '-rotate-90' : ''}` }, '▼'),
            h('span', { class: 'text-[11px]' }, n.icon || '📁'),
            h('span', { class: 'text-[11px] text-gray-400 truncate flex-1 font-medium' }, n.label),
            h('span', { class: 'text-[9px] text-gray-600 px-1 rounded', style: 'background:#1e1e1e' }, String(n.count)),
          ]),
          ...(n.collapsed ? [] : (n.children || []).map(child =>
            h(TreeItem, { node: child, activeId: props.activeId, depth: props.depth + 1, onSelect: (c) => emit('select', c), onToggle: (p) => emit('toggle', p), onCtx: (e) => emit('ctx', e) })
          ))
        ])
      } else {
        const c = n.conn
        const isActive = props.activeId === c.id
        return h('div', {
          onClick: () => emit('select', c),
          onContextmenu: (e) => { e.preventDefault(); emit('ctx', { event: e, conn: c }) },
          class: `flex items-center gap-2 px-2 py-[5px] cursor-pointer transition-colors ${isActive ? 'bg-blue-600/20' : 'hover:bg-white/5'}`,
          style: { paddingLeft: (12 + indent) + 'px' }
        }, [
          c.color ? h('span', { class: 'w-1.5 h-1.5 rounded-full flex-shrink-0', style: `background:${{ red:'#ef4444', yellow:'#eab308', green:'#22c55e', blue:'#3b82f6', purple:'#a855f7' }[c.color] || '#6b7280'}` }) : null,
          h('span', { class: 'text-[11px]' }, c.icon || '🖥️'),
          h('span', { class: `text-[11px] truncate flex-1 ${isActive ? 'text-blue-200' : 'text-gray-300'}` }, c.name),
          c.favorite ? h('span', { class: 'text-[9px]' }, '⭐') : null,
          c.host && c.host !== 'localhost' ? h('span', { class: 'text-[9px] text-gray-600 font-mono truncate max-w-[70px]' }, c.host) : null,
        ])
      }
    }
  }
})

export default { components: { TreeItem } }
</script>

<style scoped>
.ctx-item {
  @apply w-full px-3 py-1.5 text-[11px] text-gray-300 hover:bg-white/10 text-left transition-colors;
}
</style>
