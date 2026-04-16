<template>
  <div class="h-9 flex items-center px-3 select-none drag-region" style="background: var(--bg-surface); border-bottom: 1px solid var(--border-subtle);">
    <button @click="$emit('toggle-sidebar')" class="mr-3 p-1 no-drag hover:bg-white/10 rounded" style="color: var(--fg-secondary);">
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16"/></svg>
    </button>
    <span class="text-sm font-medium" style="color: var(--fg-primary);">XTerminal Pro</span>
    <div class="flex-1" />
    <div class="flex gap-1 no-drag">
      <button @click="cycleTheme" class="p-1 hover:bg-white/10 rounded text-xs" style="color: var(--fg-muted);" :title="'主题: ' + themeNames[currentTheme]">
        {{ themeIcons[currentTheme] }}
      </button>
      <button @click="$emit('toggle-fullscreen')" class="p-1 hover:bg-white/10 rounded text-xs" style="color: var(--fg-muted);" title="全屏 (F11)">⛶</button>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue'

defineEmits(['toggle-sidebar', 'toggle-fullscreen'])

const themes = ['dark', 'light', 'nord']
const themeNames = { dark: '暗色', light: '亮色', nord: 'Nord' }
const themeIcons = { dark: '🌙', light: '☀️', nord: '❄️' }
const currentTheme = ref(localStorage.getItem('xterminal-theme') || 'dark')

// Apply theme on mount
document.documentElement.setAttribute('data-theme', currentTheme.value)

function cycleTheme() {
  const idx = themes.indexOf(currentTheme.value)
  currentTheme.value = themes[(idx + 1) % themes.length]
  document.documentElement.setAttribute('data-theme', currentTheme.value)
  localStorage.setItem('xterminal-theme', currentTheme.value)
}
</script>

<style scoped>
.drag-region { -webkit-app-region: drag; }
.no-drag { -webkit-app-region: no-drag; }
</style>
