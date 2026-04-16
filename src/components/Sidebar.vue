<template>
  <div class="relative flex flex-col overflow-hidden select-none" role="navigation" aria-label="SSH连接侧边栏" :style="{ width: sidebarWidth + 'px' }" style="background: var(--bg-surface); border-right: 1px solid var(--border-subtle);">
    <!-- Header -->
    <div class="px-4 py-3 flex items-center" style="border-bottom: 1px solid var(--border-subtle);">
      <template v-if="!collapsedSidebar">
        <span class="text-xs font-semibold uppercase tracking-widest" style="color: var(--fg-muted);">连接</span>
        <div class="flex-1"></div>
        <span class="text-xs px-1.5 py-0.5 rounded" aria-live="polite" style="background: var(--bg-base); color: var(--fg-muted);">{{ connections.length }}</span>
        <button @click="$emit('add')" type="button" aria-label="添加新连接" class="w-5 h-5 flex items-center justify-center rounded text-sm transition-colors ml-1.5" style="color: var(--fg-muted);" title="添加连接">+</button>
      </template>
      <button @click="toggleCollapse" type="button" :aria-label="collapsedSidebar ? '展开侧栏' : '收起侧栏'" class="w-5 h-5 flex items-center justify-center rounded text-sm transition-colors" :class="{ 'ml-auto': !collapsedSidebar }" style="color: var(--fg-muted);" :title="collapsedSidebar ? '展开侧栏' : '收起侧栏'">
        {{ collapsedSidebar ? '→' : '←' }}
      </button>
    </div>

    <!-- Search -->
    <div v-if="!collapsedSidebar" class="px-2 py-1.5" style="border-bottom: 1px solid var(--border-subtle);">
      <input v-model="searchQuery" aria-label="搜索连接" placeholder="🔍 搜索..." class="w-full text-xs px-2 py-1 rounded border focus:outline-none transition-colors" style="background: var(--bg-base); color: var(--fg-secondary); border-color: var(--border);" />
    </div>

    <!-- Tree -->
    <div class="flex-1 overflow-y-auto py-1">
      <template v-for="node in tree" :key="node.key">
        <TreeItem :node="node" :active-id="activeId" :depth="0" :latency-map="latencyMap" :collapsed-sidebar="collapsedSidebar"
          @select="$emit('select', $event)" @toggle="toggleNode" @ctx="onCtxEvent" />
      </template>
      <div v-if="tree.length === 0 && !collapsedSidebar" class="text-center py-10">
        <div class="text-2xl mb-2 opacity-30">{{ searchQuery ? '🔍' : '🔌' }}</div>
        <div class="text-xs" style="color: var(--fg-muted);">{{ searchQuery ? '没有匹配' : '点击 + 添加连接' }}</div>
      </div>
    </div>

    <!-- Quick Commands -->
    <div v-if="!collapsedSidebar" class="border-t px-2 py-2" style="border-color: var(--border-subtle);">
      <div class="text-xs px-1 mb-1.5 uppercase tracking-wider" style="color: var(--fg-muted);">快捷命令</div>
      <div class="flex flex-wrap gap-1">
        <button v-for="cmd in quickCommands" :key="cmd.l" @click="$emit('quick-command', cmd.c)"
          class="text-xs px-1.5 py-0.5 rounded font-mono transition-colors" style="background: var(--bg-base); color: var(--fg-muted); hover:background: var(--bg-hover);">
          {{ cmd.l }}
        </button>
      </div>
    </div>

    <!-- Resize -->
    <div v-if="!collapsedSidebar" class="absolute right-0 top-0 bottom-0 w-0.5 cursor-col-resize hover:bg-blue-500/50" @mousedown="startResize" />

    <!-- Context Menu -->
    <div v-if="ctx.show" role="menu" aria-label="连接操作菜单" class="fixed z-50 rounded-lg shadow-2xl py-1 min-w-[150px]" style="background: var(--bg-elevated); border: 1px solid var(--border);" :style="{ left: ctx.x + 'px', top: ctx.y + 'px' }">
      <button @click="doCtx('select')" type="button" role="menuitem" aria-label="连接到服务器" class="ctx-item">📂 连接</button>
      <button @click="doCtx('edit')" type="button" role="menuitem" aria-label="编辑连接信息" class="ctx-item">✏️ 编辑</button>
      <button @click="doCtx('duplicate')" type="button" role="menuitem" aria-label="复制连接" class="ctx-item">📋 复制</button>
      <button @click="doCtx('test')" type="button" role="menuitem" aria-label="测试连接" class="ctx-item">🔗 测试</button>
      <div class="border-t my-1" style="border-color: var(--border);" />
      <button @click="doCtx('fav')" type="button" role="menuitem" :aria-label="ctx.conn?.favorite ? '取消收藏' : '收藏连接'" class="ctx-item">⭐ {{ ctx.conn?.favorite ? '取消收藏' : '收藏' }}</button>
      <div class="border-t my-1" style="border-color: var(--border);" />
      <button @click="doCtx('delete')" type="button" role="menuitem" aria-label="删除连接" class="ctx-item" style="color: var(--danger);">🗑 删除</button>
    </div>
    <div v-if="ctx.show" class="fixed inset-0 z-40" @click="ctx.show = false" aria-label="关闭菜单" />
  </div>
</template>

<script setup>
import { computed, reactive, ref, defineComponent } from 'vue'

const props = defineProps({ connections: Array, activeId: String, latencyMap: { type: Object, default: () => ({}) } })
const emit = defineEmits(['select', 'add', 'delete', 'quick-command', 'edit', 'duplicate', 'test', 'favorite'])

const searchQuery = ref('')
const collapsed = reactive({})
const collapsedSidebar = ref(false)
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

function toggleCollapse() {
  collapsedSidebar.value = !collapsedSidebar.value
  sidebarWidth.value = collapsedSidebar.value ? 48 : 240
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
  if (collapsedSidebar.value) return
  resizing = true; const startX = e.clientX; const startW = sidebarWidth.value
  const onMove = (e) => { if (resizing) sidebarWidth.value = Math.max(180, Math.min(400, startW + e.clientX - startX)) }
  const onUp = () => { resizing = false; document.removeEventListener('mousemove', onMove); document.removeEventListener('mouseup', onUp) }
  document.addEventListener('mousemove', onMove); document.addEventListener('mouseup', onUp)
}

// TreeItem component - Vue template
const TreeItem = defineComponent({
  name: 'TreeItem',
  props: { node: Object, activeId: String, depth: Number, latencyMap: { type: Object, default: () => ({}) }, collapsedSidebar: Boolean },
  emits: ['select', 'toggle', 'ctx'],
  template: `
    <template v-if="node.isGroup">
      <div role="treeitem" :aria-expanded="!node.collapsed" :aria-label="node.label + ' 分组 (' + node.count + '个连接)'">
        <div
          @click="$emit('toggle', node.path)"
          role="button"
          tabindex="0"
          class="flex items-center gap-1.5 px-2 py-[5px] cursor-pointer hover:bg-white/5 transition-colors group"
          :style="{ paddingLeft: (8 + depth * 16) + 'px' }"
        >
          <span class="flex items-center justify-center w-3 h-3 transition-transform duration-150" :class="{ 'rotate-90': !node.collapsed }" style="color: var(--fg-muted);" aria-hidden="true">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M9 18l6-6-6-6" />
            </svg>
          </span>
          <span>📁</span>
          <span class="text-xs truncate flex-1 font-medium" style="color: var(--fg-secondary);">{{ node.label }}</span>
          <span class="text[10px] px-1 rounded" style="background: var(--bg-hover); color: var(--fg-muted);">{{ node.count }}</span>
        </div>
        <template v-if="!node.collapsed">
          <TreeItem
            v-for="child in node.children"
            :key="child.key"
            :node="child"
            :active-id="activeId"
            :depth="depth + 1"
            :latency-map="latencyMap"
            :collapsed-sidebar="collapsedSidebar"
            @select="$emit('select', $event)"
            @toggle="$emit('toggle', $event)"
            @ctx="$emit('ctx', $event)"
          />
        </template>
      </div>
    </template>
    <template v-else>
      <div
        @click="$emit('select', node.conn)"
        @contextmenu.prevent="(e) => { e.preventDefault(); $emit('ctx', { event: e, conn: node.conn }) }"
        class="flex items-center gap-2 px-2 py-[5px] cursor-pointer transition-colors"
        :class="{ 'bg-white/5': activeId === node.conn.id }"
        :style="{ paddingLeft: (12 + depth * 16) + 'px' }"
      >
        <span v-if="node.conn.color" class="w-1.5 h-1.5 rounded-full flex-shrink-0" :style="{ background: { red:'#ef4444', yellow:'#eab308', green:'#22c55e', blue:'#3b82f6', purple:'#a855f7' }[node.conn.color] || '#6b7280' }"></span>
        <span class="flex items-center">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="2" y="3" width="20" height="14" rx="2" ry="2" />
            <line x1="8" y1="21" x2="16" y2="21" />
            <line x1="12" y1="17" x2="12" y2="21" />
          </svg>
        </span>
        <span class="text-xs truncate flex-1" :style="{ color: activeId === node.conn.id ? 'var(--accent)' : 'var(--fg-secondary)' }">{{ node.conn.name }}</span>
        <span v-if="node.conn.favorite" class="text-xs" style="color: #eab308;">⭐</span>
        <span v-if="node.conn.host && node.conn.host !== 'localhost'" class="text-xs font-mono truncate max-w-[70px]" style="color: var(--fg-muted);">{{ node.conn.host }}</span>
        <span v-if="!node.conn.host || node.conn.host === 'localhost' || node.conn.host === '127.0.0.1'"></span>
        <span v-else class="text-xs font-mono flex-shrink-0 ml-auto" :style="{ color: latencyMap[node.conn.id] == null ? 'var(--fg-muted)' : latencyMap[node.conn.id] < 50 ? 'var(--success)' : latencyMap[node.conn.id] < 150 ? 'var(--warning)' : 'var(--danger)' }">
          {{ latencyMap[node.conn.id] == null ? '—' : latencyMap[node.conn.id] + 'ms' }}
        </span>
      </div>
    </template>
  `
})

defineExpose({ TreeItem })
</script>

<style scoped>
.ctx-item {
  @apply w-full px-3 py-1.5 text-xs text-left transition-colors;
  color: var(--fg-secondary);
}
.ctx-item:hover {
  background: var(--bg-hover);
}
</style>
