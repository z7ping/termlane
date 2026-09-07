<template>
  <div class="titlebar h-9 flex items-center px-2 select-none drag-region">
    <button
      type="button"
      class="titlebar-button no-drag"
      aria-label="切换侧边栏"
      title="切换侧边栏"
      @click="$emit('toggle-sidebar')"
    >
      <PanelLeft :size="16" :stroke-width="1.8" />
    </button>

    <span class="ml-1 text-sm font-medium" style="color: var(--fg-primary);">XTerminal Pro</span>
    <div class="flex-1" />

    <div class="flex items-center gap-0.5 no-drag">
      <button
        type="button"
        class="titlebar-button"
        :aria-label="`切换主题，当前${themeNames[currentTheme]}`"
        :title="`主题: ${themeNames[currentTheme]}`"
        @click="cycleTheme"
      >
        <component :is="themeIcons[currentTheme]" :size="15" :stroke-width="1.8" />
      </button>

      <button
        type="button"
        class="titlebar-button"
        aria-label="打开设置"
        title="设置"
        @click="$emit('open-settings')"
      >
        <Settings :size="15" :stroke-width="1.8" />
      </button>

      <button
        type="button"
        class="titlebar-button"
        aria-label="切换全屏"
        title="全屏 (F11)"
        @click="$emit('toggle-fullscreen')"
      >
        <Maximize2 :size="15" :stroke-width="1.8" />
      </button>
    </div>
  </div>
</template>

<script setup>
import { STORAGE_KEYS } from '@/utils/storage-keys.js'
import { ref } from 'vue'
import { Maximize2, Moon, PanelLeft, Settings, Snowflake, Sun } from 'lucide-vue-next'

defineEmits(['toggle-sidebar', 'toggle-fullscreen', 'open-settings'])

const themes = ['dark', 'light', 'nord']
const themeNames = { dark: '暗色', light: '亮色', nord: 'Nord' }
const themeIcons = { dark: Moon, light: Sun, nord: Snowflake }
const currentTheme = ref(localStorage.getItem(STORAGE_KEYS.THEME) || 'dark')

document.documentElement.setAttribute('data-theme', currentTheme.value)

function cycleTheme() {
  const idx = themes.indexOf(currentTheme.value)
  currentTheme.value = themes[(idx + 1) % themes.length]
  document.documentElement.setAttribute('data-theme', currentTheme.value)
  localStorage.setItem(STORAGE_KEYS.THEME, currentTheme.value)
}
</script>

<style scoped>
.titlebar {
  background: var(--bg-surface);
  border-bottom: 1px solid var(--border-subtle);
}

.titlebar-button {
  width: 28px;
  height: 28px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 0;
  border-radius: 6px;
  background: transparent;
  color: var(--fg-muted);
  transition: background-color var(--transition-fast), color var(--transition-fast);
}

.titlebar-button:hover {
  background: var(--bg-hover);
  color: var(--fg-primary);
}

.drag-region { -webkit-app-region: drag; }
.no-drag { -webkit-app-region: no-drag; }
</style>
