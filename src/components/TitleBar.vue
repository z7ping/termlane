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

    <span class="ml-1 text-sm font-medium" style="color: var(--fg-primary);">Termlane</span>
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
import { onMounted, onUnmounted, ref } from 'vue'
import { Maximize2, Moon, PanelLeft, Settings, Snowflake, Sun } from 'lucide-vue-next'
import { applyTheme, getStoredTheme, nextTheme } from '@/utils/theme-state'

defineEmits(['toggle-sidebar', 'toggle-fullscreen', 'open-settings'])

const themeNames = { dark: '暗色', light: '亮色', nord: 'Nord' }
const themeIcons = { dark: Moon, light: Sun, nord: Snowflake }
const currentTheme = ref(getStoredTheme())

function cycleTheme() {
  currentTheme.value = nextTheme(currentTheme.value)
  applyTheme(currentTheme.value)
}

function handleThemeChanged(event) {
  currentTheme.value = event.detail || getStoredTheme()
}

onMounted(() => window.addEventListener('termlane-theme-changed', handleThemeChanged))
onUnmounted(() => window.removeEventListener('termlane-theme-changed', handleThemeChanged))
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
