<template>
  <template v-if="node.isGroup">
    <div role="treeitem" :aria-expanded="!node.collapsed" :aria-label="node.label + ' 分组 (' + node.count + '个连接)'">
      <div
        role="button"
        tabindex="0"
        class="tree-row group-row"
        :style="{ paddingLeft: (8 + depth * 16) + 'px' }"
        @click="$emit('toggle', node.path)"
        @keydown.enter="$emit('toggle', node.path)"
        @keydown.space.prevent="$emit('toggle', node.path)"
      >
        <ChevronRight
          :size="13"
          :stroke-width="1.8"
          class="tree-chevron"
          :class="{ expanded: !node.collapsed }"
          aria-hidden="true"
        />
        <Folder :size="14" :stroke-width="1.8" class="tree-icon" aria-hidden="true" />
        <template v-if="!collapsedSidebar">
          <span class="text-xs truncate flex-1 font-medium" style="color: var(--fg-secondary);">{{ node.label }}</span>
          <span class="tree-count">{{ node.count }}</span>
        </template>
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
      role="treeitem"
      tabindex="0"
      class="tree-row connection-row"
      :class="{ active: activeId === node.conn.id }"
      :style="{ paddingLeft: collapsedSidebar ? '16px' : (12 + depth * 16) + 'px' }"
      :aria-label="node.conn.name"
      @click="$emit('select', node.conn)"
      @keydown.enter="$emit('select', node.conn)"
      @keydown.space.prevent="$emit('select', node.conn)"
      @contextmenu.prevent="(event) => $emit('ctx', { event, conn: node.conn })"
    >
      <span
        v-if="node.conn.color && !collapsedSidebar"
        class="w-1.5 h-1.5 rounded-full flex-shrink-0"
        :style="{ background: connectionColor(node.conn.color) }"
      />

      <Server :size="14" :stroke-width="1.8" class="tree-icon flex-shrink-0" aria-hidden="true" />

      <template v-if="!collapsedSidebar">
        <span class="text-xs truncate flex-1" :style="{ color: activeId === node.conn.id ? 'var(--accent)' : 'var(--fg-secondary)' }">
          {{ node.conn.name }}
        </span>
        <Star v-if="node.conn.favorite" :size="12" :stroke-width="1.8" fill="currentColor" class="favorite-icon" aria-label="已收藏" />
        <span
          v-if="node.conn.host && node.conn.host !== 'localhost'"
          class="text-xs font-mono truncate max-w-[70px]"
          style="color: var(--fg-muted);"
        >
          {{ node.conn.host }}
        </span>
        <span
          v-if="node.conn.host && node.conn.host !== 'localhost' && node.conn.host !== '127.0.0.1'"
          class="text-xs font-mono flex-shrink-0 ml-auto"
          :style="{ color: latencyColor(latencyMap[node.conn.id]) }"
        >
          {{ latencyMap[node.conn.id] == null ? '—' : latencyMap[node.conn.id] + 'ms' }}
        </span>
      </template>
    </div>
  </template>
</template>

<script setup>
import { ChevronRight, Folder, Server, Star } from 'lucide-vue-next'

defineOptions({ name: 'TreeItem' })
defineProps({
  node: Object,
  activeId: String,
  depth: Number,
  latencyMap: { type: Object, default: () => ({}) },
  collapsedSidebar: Boolean,
})
defineEmits(['select', 'toggle', 'ctx'])

const connectionColors = {
  red: '#ef4444',
  yellow: '#eab308',
  green: '#22c55e',
  blue: '#3b82f6',
  purple: '#a855f7',
}

function connectionColor(color) {
  return connectionColors[color] || '#6b7280'
}

function latencyColor(latency) {
  if (latency == null) return 'var(--fg-muted)'
  if (latency < 50) return 'var(--success)'
  if (latency < 150) return 'var(--warning)'
  return 'var(--danger)'
}
</script>

<style scoped>
.tree-row {
  min-height: 28px;
  display: flex;
  align-items: center;
  gap: 6px;
  padding-right: 8px;
  cursor: pointer;
  outline: none;
  transition: background-color var(--transition-fast), color var(--transition-fast);
}

.tree-row:hover,
.tree-row:focus-visible {
  background: var(--bg-hover);
}

.connection-row.active {
  background: var(--bg-hover);
}

.connection-row.active .tree-icon {
  color: var(--accent);
}

.tree-icon,
.tree-chevron {
  color: var(--fg-muted);
}

.tree-chevron {
  transition: transform var(--transition-fast);
}

.tree-chevron.expanded {
  transform: rotate(90deg);
}

.tree-count {
  padding: 1px 4px;
  border-radius: 4px;
  background: var(--bg-hover);
  color: var(--fg-muted);
  font-size: 10px;
}

.favorite-icon {
  color: var(--warning);
  flex-shrink: 0;
}
</style>
