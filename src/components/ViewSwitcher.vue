<template>
  <div ref="rootRef" class="view-switcher" aria-label="工作区切换">
    <button
      v-for="item in primaryViews"
      :key="item.value"
      type="button"
      class="view-button"
      :class="{ active: modelValue === item.value }"
      :aria-pressed="modelValue === item.value"
      @click="selectView(item.value)"
    >
      <component :is="item.icon" :size="14" :stroke-width="1.8" />
      <span>{{ item.label }}</span>
    </button>

    <div class="tool-menu-wrap">
      <button
        type="button"
        class="view-button tool-button"
        :class="{ active: activeTool != null }"
        aria-haspopup="menu"
        :aria-expanded="menuOpen"
        @click="menuOpen = !menuOpen"
      >
        <Wrench :size="14" :stroke-width="1.8" />
        <span>{{ activeTool ? activeTool.label : '工具' }}</span>
        <ChevronDown :size="12" :stroke-width="1.8" :class="{ rotated: menuOpen }" />
      </button>

      <Transition name="menu-fade">
        <div v-if="menuOpen" class="tool-menu" role="menu" aria-label="更多工具">
          <button
            v-for="item in toolViews"
            :key="item.value"
            type="button"
            role="menuitemradio"
            :aria-checked="modelValue === item.value"
            class="tool-menu-item"
            :class="{ active: modelValue === item.value }"
            @click="selectView(item.value)"
          >
            <component :is="item.icon" :size="15" :stroke-width="1.8" />
            <span>{{ item.label }}</span>
          </button>
        </div>
      </Transition>
    </div>
  </div>
</template>

<script setup>
import { computed, onMounted, onUnmounted, ref } from 'vue'
import {
  Activity,
  Bookmark,
  ChevronDown,
  Circle,
  Clock,
  Command,
  FileText,
  Folder,
  Gauge,
  Globe,
  Network,
  Terminal,
  Wrench,
  Zap,
} from 'lucide-vue-next'

const props = defineProps({
  modelValue: {
    type: String,
    required: true,
  },
})

const emit = defineEmits(['update:modelValue'])

const rootRef = ref(null)
const menuOpen = ref(false)

const primaryViews = [
  { value: 'terminal', label: '终端', icon: Terminal },
  { value: 'sftp', label: '文件', icon: Folder },
  { value: 'batch', label: '批量', icon: Zap },
]

const toolViews = [
  { value: 'monitor', label: '监控', icon: Activity },
  { value: 'speed', label: '测速', icon: Gauge },
  { value: 'recorder', label: '录制', icon: Circle },
  { value: 'notes', label: '笔记', icon: FileText },
  { value: 'bookmarks', label: '书签', icon: Bookmark },
  { value: 'proxy', label: '代理', icon: Globe },
  { value: 'commands', label: '快捷命令', icon: Command },
  { value: 'forward', label: '端口转发', icon: Network },
  { value: 'tasks', label: '定时任务', icon: Clock },
  { value: 'macro', label: '宏', icon: Wrench },
]

const activeTool = computed(() => toolViews.find(item => item.value === props.modelValue) || null)

function selectView(value) {
  emit('update:modelValue', value)
  menuOpen.value = false
}

function handlePointerDown(event) {
  if (!rootRef.value?.contains(event.target)) menuOpen.value = false
}

function handleKeydown(event) {
  if (event.key === 'Escape') menuOpen.value = false
}

onMounted(() => {
  document.addEventListener('pointerdown', handlePointerDown)
  document.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  document.removeEventListener('pointerdown', handlePointerDown)
  document.removeEventListener('keydown', handleKeydown)
})
</script>

<style scoped>
.view-switcher {
  display: flex;
  align-items: center;
  gap: 2px;
  margin-left: 8px;
  flex-shrink: 0;
}

.view-button {
  height: 26px;
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 0 8px;
  border: 0;
  border-radius: 6px;
  background: transparent;
  color: var(--fg-muted);
  font-size: 12px;
  line-height: 1;
  white-space: nowrap;
  transition: background-color var(--transition-fast), color var(--transition-fast);
}

.view-button:hover {
  background: var(--bg-hover);
  color: var(--fg-primary);
}

.view-button.active {
  background: var(--accent-hover);
  color: var(--accent);
}

.tool-menu-wrap {
  position: relative;
}

.tool-button svg:last-child {
  transition: transform var(--transition-fast);
}

.tool-button svg.rotated {
  transform: rotate(180deg);
}

.tool-menu {
  position: absolute;
  top: calc(100% + 6px);
  right: 0;
  z-index: 60;
  width: 176px;
  padding: 5px;
  border: 1px solid var(--border);
  border-radius: 8px;
  background: var(--bg-elevated);
  box-shadow: var(--shadow-lg);
}

.tool-menu-item {
  width: 100%;
  height: 32px;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 9px;
  border: 0;
  border-radius: 6px;
  background: transparent;
  color: var(--fg-secondary);
  font-size: 12px;
  text-align: left;
  transition: background-color var(--transition-fast), color var(--transition-fast);
}

.tool-menu-item:hover,
.tool-menu-item.active {
  background: var(--bg-hover);
  color: var(--fg-primary);
}

.tool-menu-item.active svg {
  color: var(--accent);
}

.menu-fade-enter-active,
.menu-fade-leave-active {
  transition: opacity var(--transition-fast), transform var(--transition-fast);
  transform-origin: top right;
}

.menu-fade-enter-from,
.menu-fade-leave-to {
  opacity: 0;
  transform: translateY(-3px) scale(0.98);
}

@media (max-width: 920px) {
  .view-button span {
    display: none;
  }

  .view-button {
    width: 28px;
    justify-content: center;
    padding: 0;
  }

  .tool-button svg:last-child {
    display: none;
  }
}
</style>
