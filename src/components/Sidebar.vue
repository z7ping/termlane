<template>
  <div
    class="sidebar relative flex flex-col overflow-hidden select-none"
    role="navigation"
    aria-label="SSH 连接侧边栏"
    :style="{ width: sidebarWidth + 'px' }"
  >
    <div class="sidebar-header">
      <template v-if="!collapsedSidebar">
        <span class="text-xs font-semibold uppercase tracking-widest" style="color: var(--fg-muted);">连接</span>
        <div class="flex-1" />
        <span class="connection-count" aria-live="polite">{{ connections.length }}</span>
        <button class="sidebar-icon-button" type="button" aria-label="添加新连接" title="添加连接" @click="$emit('add')">
          <Plus :size="14" :stroke-width="2" />
        </button>
      </template>

      <button
        class="sidebar-icon-button"
        :class="{ 'ml-auto': !collapsedSidebar }"
        type="button"
        :aria-label="collapsedSidebar ? '展开侧栏' : '收起侧栏'"
        :title="collapsedSidebar ? '展开侧栏' : '收起侧栏'"
        @click="toggleCollapse"
      >
        <PanelLeftOpen v-if="collapsedSidebar" :size="14" :stroke-width="1.8" />
        <PanelLeftClose v-else :size="14" :stroke-width="1.8" />
      </button>
    </div>

    <div v-if="!collapsedSidebar" class="search-wrap">
      <Search :size="13" :stroke-width="1.8" class="search-icon" aria-hidden="true" />
      <input v-model="searchQuery" aria-label="搜索连接" placeholder="搜索连接" class="search-input" />
    </div>

    <div class="flex-1 overflow-y-auto py-1" role="tree" aria-label="连接列表">
      <template v-for="node in tree" :key="node.key">
        <TreeItem
          :node="node"
          :active-id="activeId"
          :depth="0"
          :latency-map="latencyMap"
          :collapsed-sidebar="collapsedSidebar"
          @select="$emit('select', $event)"
          @toggle="toggleNode"
          @ctx="onCtxEvent"
        />
      </template>

      <div v-if="tree.length === 0 && !collapsedSidebar" class="empty-state">
        <Search v-if="searchQuery" :size="22" :stroke-width="1.4" />
        <Unplug v-else :size="22" :stroke-width="1.4" />
        <div class="text-xs mt-2">{{ searchQuery ? '没有匹配的连接' : '点击 + 添加连接' }}</div>
      </div>
    </div>

    <div v-if="!collapsedSidebar" class="quick-commands">
      <div class="text-xs px-1 mb-1.5 uppercase tracking-wider" style="color: var(--fg-muted);">快捷命令</div>
      <div class="flex flex-wrap gap-1">
        <button
          v-for="cmd in quickCommands"
          :key="cmd.l"
          type="button"
          class="quick-command"
          @click="$emit('quick-command', cmd.c)"
        >
          {{ cmd.l }}
        </button>
      </div>
    </div>

    <div
      v-if="!collapsedSidebar"
      class="resize-handle absolute right-0 top-0 bottom-0 w-1 cursor-col-resize"
      aria-hidden="true"
      @mousedown="startResize"
    />

    <div
      v-if="ctx.show"
      role="menu"
      aria-label="连接操作菜单"
      class="context-menu fixed z-50"
      :style="{ left: ctx.x + 'px', top: ctx.y + 'px' }"
    >
      <button type="button" role="menuitem" class="ctx-item" @click="doCtx('select')">
        <LogIn :size="14" :stroke-width="1.8" />
        <span>连接</span>
      </button>
      <button type="button" role="menuitem" class="ctx-item" @click="doCtx('edit')">
        <Pencil :size="14" :stroke-width="1.8" />
        <span>编辑</span>
      </button>
      <button type="button" role="menuitem" class="ctx-item" @click="doCtx('duplicate')">
        <Copy :size="14" :stroke-width="1.8" />
        <span>复制</span>
      </button>
      <button type="button" role="menuitem" class="ctx-item" @click="doCtx('test')">
        <Cable :size="14" :stroke-width="1.8" />
        <span>测试连接</span>
      </button>
      <div class="menu-separator" />
      <button type="button" role="menuitem" class="ctx-item" @click="doCtx('fav')">
        <Star :size="14" :stroke-width="1.8" :fill="ctx.conn?.favorite ? 'currentColor' : 'none'" />
        <span>{{ ctx.conn?.favorite ? '取消收藏' : '收藏' }}</span>
      </button>
      <div class="menu-separator" />
      <button type="button" role="menuitem" class="ctx-item danger" @click="doCtx('delete')">
        <Trash2 :size="14" :stroke-width="1.8" />
        <span>删除</span>
      </button>
    </div>
    <div v-if="ctx.show" class="fixed inset-0 z-40" aria-label="关闭菜单" @click="ctx.show = false" />
  </div>
</template>

<script setup>
import { computed, reactive, ref } from 'vue'
import {
  Cable,
  Copy,
  LogIn,
  PanelLeftClose,
  PanelLeftOpen,
  Pencil,
  Plus,
  Search,
  Star,
  Trash2,
  Unplug,
} from 'lucide-vue-next'
import TreeItem from './TreeItem.vue'

const props = defineProps({
  connections: { type: Array, default: () => [] },
  activeId: String,
  latencyMap: { type: Object, default: () => ({}) },
})
const emit = defineEmits(['select', 'add', 'delete', 'quick-command', 'edit', 'duplicate', 'test', 'favorite'])

const searchQuery = ref('')
const collapsed = reactive({})
const collapsedSidebar = ref(false)
const sidebarWidth = ref(240)
const quickCommands = [
  { l: 'top', c: 'top' },
  { l: 'df -h', c: 'df -h' },
  { l: 'free', c: 'free -h' },
  { l: 'docker', c: 'docker ps' },
  { l: 'tail', c: 'tail -f /var/log/syslog' },
]
const ctx = reactive({ show: false, x: 0, y: 0, conn: null })

const tree = computed(() => {
  const query = searchQuery.value.trim().toLowerCase()
  const filtered = props.connections.filter(connection => {
    if (!query) return true
    return connection.name?.toLowerCase().includes(query)
      || connection.host?.toLowerCase().includes(query)
      || connection.group?.toLowerCase().includes(query)
  })

  filtered.sort((a, b) => {
    if (a.favorite && !b.favorite) return -1
    if (!a.favorite && b.favorite) return 1
    return (a.group || '').localeCompare(b.group || '') || a.name.localeCompare(b.name)
  })

  const root = []

  for (const conn of filtered) {
    const parts = (conn.group || '默认').split('/')
    let currentChildren = root
    let currentPath = ''

    for (const part of parts) {
      currentPath = currentPath ? `${currentPath}/${part}` : part
      let groupNode = currentChildren.find(node => node.isGroup && node.label === part)

      if (!groupNode) {
        groupNode = {
          key: `g:${currentPath}`,
          isGroup: true,
          label: part,
          path: currentPath,
          children: [],
          count: 0,
          collapsed: collapsed[currentPath] ?? false,
        }
        currentChildren.push(groupNode)
      }

      groupNode.count++
      currentChildren = groupNode.children
    }

    currentChildren.push({ key: `c:${conn.id}`, isGroup: false, conn })
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

function onCtxEvent({ event, conn }) {
  ctx.show = true
  ctx.x = Math.min(event.clientX, window.innerWidth - 170)
  ctx.y = Math.min(event.clientY, window.innerHeight - 230)
  ctx.conn = conn
}

function doCtx(action) {
  const connection = ctx.conn
  ctx.show = false
  if (!connection) return

  if (action === 'select') emit('select', connection)
  else if (action === 'edit') emit('edit', connection)
  else if (action === 'duplicate') emit('duplicate', connection)
  else if (action === 'test') emit('test', connection)
  else if (action === 'fav') emit('favorite', connection)
  else if (action === 'delete') emit('delete', connection.id)
}

function startResize(event) {
  if (collapsedSidebar.value) return

  const startX = event.clientX
  const startWidth = sidebarWidth.value

  const onMove = moveEvent => {
    sidebarWidth.value = Math.max(180, Math.min(400, startWidth + moveEvent.clientX - startX))
  }
  const onUp = () => {
    document.removeEventListener('mousemove', onMove)
    document.removeEventListener('mouseup', onUp)
  }

  document.addEventListener('mousemove', onMove)
  document.addEventListener('mouseup', onUp)
}
</script>

<style scoped>
.sidebar {
  flex-shrink: 0;
  background: var(--bg-surface);
  border-right: 1px solid var(--border-subtle);
}

.sidebar-header {
  min-height: 38px;
  display: flex;
  align-items: center;
  gap: 5px;
  padding: 6px 8px 6px 12px;
  border-bottom: 1px solid var(--border-subtle);
}

.connection-count {
  padding: 1px 5px;
  border-radius: 5px;
  background: var(--bg-base);
  color: var(--fg-muted);
  font-size: 10px;
}

.sidebar-icon-button {
  width: 26px;
  height: 26px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 0;
  border-radius: 5px;
  background: transparent;
  color: var(--fg-muted);
  transition: background-color var(--transition-fast), color var(--transition-fast);
}

.sidebar-icon-button:hover {
  background: var(--bg-hover);
  color: var(--fg-primary);
}

.search-wrap {
  position: relative;
  padding: 6px 8px;
  border-bottom: 1px solid var(--border-subtle);
}

.search-icon {
  position: absolute;
  left: 16px;
  top: 50%;
  transform: translateY(-50%);
  color: var(--fg-muted);
  pointer-events: none;
}

.search-input {
  width: 100%;
  height: 28px;
  padding: 0 8px 0 28px;
  border: 1px solid var(--border);
  border-radius: 6px;
  outline: none;
  background: var(--bg-base);
  color: var(--fg-secondary);
  font-size: 12px;
  transition: border-color var(--transition-fast), background-color var(--transition-fast);
}

.search-input:focus {
  border-color: var(--accent);
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 36px 12px;
  color: var(--fg-muted);
  text-align: center;
}

.quick-commands {
  padding: 8px;
  border-top: 1px solid var(--border-subtle);
}

.quick-command {
  padding: 2px 6px;
  border: 0;
  border-radius: 5px;
  background: var(--bg-base);
  color: var(--fg-muted);
  font-family: monospace;
  font-size: 11px;
  transition: background-color var(--transition-fast), color var(--transition-fast);
}

.quick-command:hover {
  background: var(--bg-hover);
  color: var(--fg-secondary);
}

.resize-handle {
  background: transparent;
  transition: background-color var(--transition-fast);
}

.resize-handle:hover {
  background: var(--accent);
}

.context-menu {
  min-width: 162px;
  padding: 5px;
  border: 1px solid var(--border);
  border-radius: 8px;
  background: var(--bg-elevated);
  box-shadow: var(--shadow-lg);
}

.ctx-item {
  width: 100%;
  height: 30px;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 8px;
  border: 0;
  border-radius: 5px;
  background: transparent;
  color: var(--fg-secondary);
  font-size: 12px;
  text-align: left;
  transition: background-color var(--transition-fast), color var(--transition-fast);
}

.ctx-item:hover {
  background: var(--bg-hover);
  color: var(--fg-primary);
}

.ctx-item.danger {
  color: var(--danger);
}

.menu-separator {
  height: 1px;
  margin: 4px 2px;
  background: var(--border-subtle);
}
</style>
