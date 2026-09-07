<template>
  <div class="tabbar h-9 flex items-center overflow-x-auto" role="tablist" aria-label="终端标签">
    <div
      v-for="tab in tabs"
      :key="tab.id"
      role="tab"
      tabindex="0"
      class="tab-item"
      :class="{ active: activeId === tab.id }"
      :aria-selected="activeId === tab.id"
      @click="$emit('select', tab.id)"
      @keydown.enter="$emit('select', tab.id)"
      @keydown.space.prevent="$emit('select', tab.id)"
      @dblclick="$emit('close', tab.id)"
      @contextmenu.prevent="showContextMenu($event, tab)"
    >
      <span class="truncate">{{ tab.name }}</span>
      <button
        type="button"
        class="tab-close"
        :aria-label="`关闭 ${tab.name}`"
        @click.stop="$emit('close', tab.id)"
      >
        <X :size="12" :stroke-width="2" />
      </button>
    </div>

    <button type="button" class="new-tab" aria-label="新终端" title="新终端" @click="$emit('new')">
      <Plus :size="15" :stroke-width="1.8" />
    </button>

    <div
      v-if="ctx.show"
      class="tab-menu fixed z-50"
      role="menu"
      aria-label="标签操作"
      :style="{ left: ctx.x + 'px', top: ctx.y + 'px' }"
    >
      <button type="button" role="menuitem" @click="onCtx('close')">关闭</button>
      <button type="button" role="menuitem" @click="onCtx('closeOthers')">关闭其他</button>
      <button type="button" role="menuitem" @click="onCtx('closeAll')">关闭全部</button>
    </div>
    <div v-if="ctx.show" class="fixed inset-0 z-40" @click="ctx.show = false" />
  </div>
</template>

<script setup>
import { reactive } from 'vue'
import { Plus, X } from 'lucide-vue-next'

defineProps({
  tabs: { type: Array, default: () => [] },
  activeId: String,
})
const emit = defineEmits(['select', 'close', 'new', 'closeOthers', 'closeAll'])

const ctx = reactive({ show: false, x: 0, y: 0, tab: null })

function showContextMenu(event, tab) {
  ctx.show = true
  ctx.x = Math.min(event.clientX, window.innerWidth - 150)
  ctx.y = Math.min(event.clientY, window.innerHeight - 112)
  ctx.tab = tab
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

<style scoped>
.tabbar {
  min-width: 0;
  background: var(--bg-surface);
}

.tab-item {
  position: relative;
  height: 100%;
  min-width: 84px;
  max-width: 190px;
  display: flex;
  align-items: center;
  gap: 7px;
  padding: 0 8px 0 11px;
  border-top: 2px solid transparent;
  outline: none;
  color: var(--fg-secondary);
  font-size: 12px;
  cursor: pointer;
  user-select: none;
  transition: background-color var(--transition-fast), color var(--transition-fast), border-color var(--transition-fast);
}

.tab-item:hover,
.tab-item:focus-visible {
  background: var(--bg-hover);
  color: var(--fg-primary);
}

.tab-item.active {
  border-top-color: var(--accent);
  background: var(--bg-base);
  color: var(--fg-primary);
}

.tab-close {
  width: 20px;
  height: 20px;
  margin-left: auto;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  border: 0;
  border-radius: 4px;
  background: transparent;
  color: var(--fg-muted);
  opacity: 0;
  transition: opacity var(--transition-fast), background-color var(--transition-fast), color var(--transition-fast);
}

.tab-item:hover .tab-close,
.tab-item.active .tab-close,
.tab-close:focus-visible {
  opacity: 1;
}

.tab-close:hover {
  background: var(--bg-elevated);
  color: var(--fg-primary);
}

.new-tab {
  width: 30px;
  height: 100%;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  border: 0;
  background: transparent;
  color: var(--fg-muted);
  transition: background-color var(--transition-fast), color var(--transition-fast);
}

.new-tab:hover {
  background: var(--bg-hover);
  color: var(--fg-primary);
}

.tab-menu {
  min-width: 140px;
  padding: 5px;
  border: 1px solid var(--border);
  border-radius: 8px;
  background: var(--bg-elevated);
  box-shadow: var(--shadow-lg);
}

.tab-menu button {
  width: 100%;
  height: 30px;
  padding: 0 8px;
  border: 0;
  border-radius: 5px;
  background: transparent;
  color: var(--fg-secondary);
  font-size: 12px;
  text-align: left;
  transition: background-color var(--transition-fast), color var(--transition-fast);
}

.tab-menu button:hover {
  background: var(--bg-hover);
  color: var(--fg-primary);
}
</style>
