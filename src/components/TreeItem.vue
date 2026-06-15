<template>
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
        <span class="text-[10px] px-1 rounded" style="background: var(--bg-hover); color: var(--fg-muted);">{{ node.count }}</span>
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
</template>

<script setup>
defineOptions({ name: 'TreeItem' })
defineProps({ node: Object, activeId: String, depth: Number, latencyMap: { type: Object, default: () => ({}) }, collapsedSidebar: Boolean })
defineEmits(['select', 'toggle', 'ctx'])
</script>
