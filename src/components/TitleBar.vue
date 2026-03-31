<template>
  <div class="h-9 flex items-center px-3 select-none drag-region" style="background: var(--bg-surface); border-bottom: 1px solid var(--border-subtle);">
    <button @click="$emit('toggle-sidebar')" class="mr-3 p-1 no-drag hover:bg-white/10 rounded" style="color: var(--fg-secondary);">
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16"/></svg>
    </button>
    <span class="text-sm font-medium" style="color: var(--fg-primary);">XTerminal Pro</span>
    <div class="flex-1" />
    <div class="flex gap-2 no-drag">
      <button @click="$emit('toggle-fullscreen')" class="text-xs p-1 hover:bg-white/10 rounded" style="color: var(--fg-muted);" title="全屏 (F11)">⛶</button>
    </div>
  </div>
</template>

<script setup>
defineEmits(['toggle-sidebar', 'toggle-fullscreen'])
</script>

<style scoped>
.drag-region { -webkit-app-region: drag; }
.no-drag { -webkit-app-region: no-drag; }
</style>
