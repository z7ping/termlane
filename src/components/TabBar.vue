<template>
  <div class="h-9 flex items-center overflow-x-auto" style="background: var(--bg-surface); border-bottom: 1px solid var(--border-subtle);">
    <div v-for="tab in tabs" :key="tab.id"
      @click="$emit('select', tab.id)"
      @dblclick="$emit('close', tab.id)"
      @contextmenu.prevent="showContextMenu($event, tab)"
      class="flex items-center gap-1.5 px-3 h-full cursor-pointer text-xs select-none min-w-0 max-w-48 transition-colors"
      :style="activeId === tab.id
        ? 'background: var(--bg-base); color: var(--fg-primary); border-top: 2px solid var(--accent);'
        : 'color: var(--fg-secondary); border-top: 2px solid transparent;'"
    >
      <span class="truncate">{{ tab.name }}</span>
      <button @click.stop="$emit('close', tab.id)" class="rounded w-4 h-4 flex items-center justify-center leading-none hover:bg-white/10" style="color: var(--fg-muted);">✕</button>
    </div>
    <button @click="$emit('new')" class="px-2 h-full flex items-center justify-center hover:bg-white/5" style="color: var(--fg-muted);" title="新终端"><Plus :size="16" /></button>

    <div v-if="ctx.show" class="fixed z-50 rounded-lg shadow-xl py-1 min-w-[140px]" style="background: var(--bg-elevated); border: 1px solid var(--border);" :style="{ left: ctx.x + 'px', top: ctx.y + 'px' }">
      <button @click="onCtx('close')" class="w-full px-3 py-1.5 text-xs hover:bg-white/10 text-left" style="color: var(--fg-primary);">关闭</button>
      <button @click="onCtx('closeOthers')" class="w-full px-3 py-1.5 text-xs hover:bg-white/10 text-left" style="color: var(--fg-primary);">关闭其他</button>
      <button @click="onCtx('closeAll')" class="w-full px-3 py-1.5 text-xs hover:bg-white/10 text-left" style="color: var(--fg-primary);">关闭全部</button>
    </div>
    <div v-if="ctx.show" class="fixed inset-0 z-40" @click="ctx.show = false" />
  </div>
</template>

<script setup>
import { reactive } from 'vue'
import { Plus } from 'lucide-vue-next'

defineProps({ tabs: Array, activeId: String })
const emit = defineEmits(['select', 'close', 'new', 'closeOthers', 'closeAll'])

const ctx = reactive({ show: false, x: 0, y: 0, tab: null })

function showContextMenu(e, tab) { ctx.show = true; ctx.x = e.clientX; ctx.y = e.clientY; ctx.tab = tab }

function onCtx(action) {
  const tab = ctx.tab; ctx.show = false; if (!tab) return
  if (action === 'close') emit('close', tab.id)
  else if (action === 'closeOthers') emit('closeOthers', tab.id)
  else if (action === 'closeAll') emit('closeAll')
}
</script>
