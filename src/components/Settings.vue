<template>
  <div class="h-full flex flex-col" style="background: var(--bg-base);">
    <div class="settings-header h-9 flex items-center px-3">
      <SettingsIcon :size="15" :stroke-width="1.8" />
      <span class="text-sm font-medium">设置</span>
    </div>

    <div class="flex-1 overflow-y-auto p-4">
      <div class="settings-content">
        <section class="settings-section">
          <h3 class="section-title">外观</h3>

          <div class="setting-block">
            <div class="setting-label">主题</div>
            <div class="theme-grid">
              <button
                v-for="theme in themes"
                :key="theme.value"
                type="button"
                class="theme-button"
                :class="{ active: currentTheme === theme.value }"
                :aria-pressed="currentTheme === theme.value"
                @click="setTheme(theme.value)"
              >
                <component :is="theme.icon" :size="15" :stroke-width="1.8" />
                <span>{{ theme.label }}</span>
              </button>
            </div>
          </div>

          <div class="setting-row">
            <div>
              <div class="setting-label">终端字体</div>
              <div class="setting-description">新建终端时生效</div>
            </div>
            <div class="stepper">
              <button type="button" aria-label="减小字体" @click="settings.fontSize = Math.max(13, settings.fontSize - 1)">−</button>
              <span>{{ settings.fontSize }}</span>
              <button type="button" aria-label="增大字体" @click="settings.fontSize = Math.min(24, settings.fontSize + 1)">+</button>
            </div>
          </div>

          <div class="font-preview" :style="{ fontSize: settings.fontSize + 'px' }">
            ssh user@example.com
          </div>
        </section>

        <section class="settings-section">
          <h3 class="section-title">终端</h3>
          <div class="setting-row">
            <div>
              <div class="setting-label">滚动缓冲区</div>
              <div class="setting-description">每个新终端保留的历史行数</div>
            </div>
            <select v-model.number="settings.scrollback" class="setting-select" aria-label="滚动缓冲区行数">
              <option :value="1000">1,000</option>
              <option :value="5000">5,000</option>
              <option :value="10000">10,000</option>
              <option :value="50000">50,000</option>
            </select>
          </div>
        </section>

        <section class="settings-section">
          <h3 class="section-title">快捷键</h3>
          <div class="shortcut-list">
            <div v-for="shortcut in shortcuts" :key="shortcut.name" class="shortcut-row">
              <span>{{ shortcut.name }}</span>
              <div class="flex items-center gap-2">
                <kbd>{{ shortcut.key }}</kbd>
                <button
                  type="button"
                  class="icon-button"
                  :aria-label="`编辑${shortcut.name}快捷键`"
                  :title="`编辑 ${shortcut.name}`"
                  @click="editShortcut(shortcut)"
                >
                  <Pencil :size="14" :stroke-width="1.8" />
                </button>
              </div>
            </div>
          </div>
        </section>

        <section class="settings-section">
          <h3 class="section-title">关于</h3>
          <div class="about-card">
            <div class="flex items-center gap-2">
              <Info :size="16" :stroke-width="1.8" />
              <span class="setting-label">XTerminal Pro{{ appVersion ? ` v${appVersion}` : '' }}</span>
            </div>
            <p>轻量级 SSH 终端与远程文件管理工具。</p>
            <p>基于 Tauri v2、Vue 3 和 xterm.js。</p>
          </div>
        </section>
      </div>
    </div>

    <div
      v-if="editingShortcut"
      class="fixed inset-0 z-50 flex items-center justify-center"
      style="background: rgba(0, 0, 0, 0.7);"
      role="dialog"
      aria-modal="true"
      :aria-label="`编辑${editingShortcut.name}快捷键`"
      @click.self="editingShortcut = null"
    >
      <div class="shortcut-dialog">
        <h3>编辑快捷键：{{ editingShortcut.name }}</h3>
        <label for="shortcut-input">按下新的快捷键组合</label>
        <input
          id="shortcut-input"
          ref="shortcutInput"
          v-model="newShortcutKey"
          class="shortcut-input"
          placeholder="例如 Ctrl+Shift+K"
          autocomplete="off"
          @keydown="captureShortcut"
        />
        <p>Esc 取消，Enter 保存</p>
        <div class="flex justify-end gap-2 mt-4">
          <button type="button" class="secondary-button" @click="editingShortcut = null">取消</button>
          <button type="button" class="primary-button" @click="saveShortcut">保存</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { STORAGE_KEYS } from '@/utils/storage-keys.js'
import { applyTheme, getStoredTheme } from '@/utils/theme-state'
import { invoke } from '@/utils/tauri.js'
import { nextTick, onMounted, reactive, ref, watch } from 'vue'
import {
  Info,
  Moon,
  Pencil,
  Settings as SettingsIcon,
  Snowflake,
  Sun,
} from 'lucide-vue-next'

const themes = [
  { value: 'dark', label: '暗色', icon: Moon },
  { value: 'light', label: '亮色', icon: Sun },
  { value: 'nord', label: 'Nord', icon: Snowflake },
]

const currentTheme = ref(getStoredTheme())
const appVersion = ref('')

const settings = reactive({
  fontSize: parseInt(localStorage.getItem(STORAGE_KEYS.FONT_SIZE)) || 14,
  scrollback: parseInt(localStorage.getItem(STORAGE_KEYS.SCROLLBACK)) || 10000,
})

watch(() => settings.fontSize, value => localStorage.setItem(STORAGE_KEYS.FONT_SIZE, String(value)))
watch(() => settings.scrollback, value => localStorage.setItem(STORAGE_KEYS.SCROLLBACK, String(value)))

const defaultShortcuts = [
  { name: '搜索', key: 'Ctrl+Shift+F' },
  { name: '清屏', key: 'Ctrl+L' },
  { name: '中断', key: 'Ctrl+C' },
  { name: '新建标签', key: 'Ctrl+T' },
  { name: '关闭标签', key: 'Ctrl+W' },
  { name: '全屏', key: 'F11' },
]

const shortcuts = defaultShortcuts.map(shortcut => ({
  name: shortcut.name,
  key: localStorage.getItem(`${STORAGE_KEYS.SHORTCUT_PREFIX}${shortcut.name}`) || shortcut.key,
}))

const editingShortcut = ref(null)
const newShortcutKey = ref('')
const shortcutInput = ref(null)

onMounted(async () => {
  try {
    appVersion.value = await invoke('get_app_version')
  } catch {
    appVersion.value = ''
  }
})

function setTheme(value) {
  currentTheme.value = value
  applyTheme(value)
}

function editShortcut(shortcut) {
  editingShortcut.value = shortcut
  newShortcutKey.value = shortcut.key
  nextTick(() => shortcutInput.value?.focus())
}

function captureShortcut(event) {
  event.preventDefault()

  if (event.key === 'Escape') {
    editingShortcut.value = null
    return
  }
  if (event.key === 'Enter') {
    saveShortcut()
    return
  }

  const modifiers = []
  if (event.ctrlKey) modifiers.push('Ctrl')
  if (event.altKey) modifiers.push('Alt')
  if (event.shiftKey) modifiers.push('Shift')
  if (event.metaKey) modifiers.push('Meta')

  const modifierKeys = ['Control', 'Alt', 'Shift', 'Meta']
  if (modifierKeys.includes(event.key)) return

  const keyName = event.key.length === 1 ? event.key.toUpperCase() : event.key
  newShortcutKey.value = [...modifiers, keyName].join('+')
}

function saveShortcut() {
  if (!editingShortcut.value || !newShortcutKey.value) return

  editingShortcut.value.key = newShortcutKey.value
  localStorage.setItem(
    `${STORAGE_KEYS.SHORTCUT_PREFIX}${editingShortcut.value.name}`,
    newShortcutKey.value,
  )
  window.dispatchEvent(new CustomEvent('shortcut-changed', {
    detail: { name: editingShortcut.value.name, key: newShortcutKey.value },
  }))
  editingShortcut.value = null
}
</script>

<style scoped>
.settings-header {
  gap: 7px;
  color: var(--fg-secondary);
  background: var(--bg-surface);
  border-bottom: 1px solid var(--border-subtle);
}

.settings-content {
  width: min(680px, 100%);
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: 22px;
}

.settings-section {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.section-title {
  margin: 0;
  color: var(--fg-muted);
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.setting-block,
.setting-row,
.shortcut-list,
.about-card {
  border: 1px solid var(--border-subtle);
  border-radius: 8px;
  background: var(--bg-surface);
}

.setting-block {
  padding: 12px;
}

.setting-row {
  min-height: 54px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 9px 12px;
}

.setting-label {
  color: var(--fg-primary);
  font-size: 13px;
  font-weight: 500;
}

.setting-description {
  margin-top: 2px;
  color: var(--fg-muted);
  font-size: 11px;
}

.theme-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 6px;
  margin-top: 8px;
}

.theme-button {
  height: 32px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--bg-elevated);
  color: var(--fg-secondary);
  font-size: 12px;
  transition: background-color var(--transition-fast), border-color var(--transition-fast), color var(--transition-fast);
}

.theme-button:hover {
  background: var(--bg-hover);
  color: var(--fg-primary);
}

.theme-button.active {
  border-color: var(--accent);
  background: var(--accent-hover);
  color: var(--accent);
}

.stepper {
  display: flex;
  align-items: center;
  border: 1px solid var(--border);
  border-radius: 6px;
  overflow: hidden;
}

.stepper button,
.stepper span {
  width: 30px;
  height: 28px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-elevated);
  color: var(--fg-secondary);
  font-size: 12px;
}

.stepper button:hover {
  background: var(--bg-hover);
  color: var(--fg-primary);
}

.font-preview {
  padding: 10px 12px;
  border: 1px solid var(--border-subtle);
  border-radius: 8px;
  background: var(--bg-base);
  color: var(--fg-secondary);
  font-family: 'Cascadia Code', 'Cascadia Mono', 'JetBrains Mono', Consolas, monospace;
}

.setting-select,
.shortcut-input {
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--bg-elevated);
  color: var(--fg-primary);
  outline: none;
}

.setting-select {
  min-width: 92px;
  padding: 5px 8px;
  font-size: 12px;
}

.shortcut-list {
  overflow: hidden;
}

.shortcut-row {
  min-height: 40px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 6px 10px 6px 12px;
  color: var(--fg-secondary);
  font-size: 12px;
  border-bottom: 1px solid var(--border-subtle);
}

.shortcut-row:last-child {
  border-bottom: 0;
}

.shortcut-row kbd {
  padding: 2px 7px;
  border: 1px solid var(--border);
  border-radius: 5px;
  background: var(--bg-base);
  color: var(--fg-muted);
  font-family: monospace;
  font-size: 11px;
}

.icon-button {
  width: 26px;
  height: 26px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 0;
  border-radius: 5px;
  background: transparent;
  color: var(--fg-muted);
}

.icon-button:hover {
  background: var(--bg-hover);
  color: var(--fg-primary);
}

.about-card {
  padding: 12px;
  color: var(--fg-muted);
  font-size: 11px;
  line-height: 1.7;
}

.about-card p {
  margin: 4px 0 0;
}

.shortcut-dialog {
  width: 320px;
  padding: 18px;
  border: 1px solid var(--border);
  border-radius: 10px;
  background: var(--bg-elevated);
  box-shadow: var(--shadow-lg);
}

.shortcut-dialog h3 {
  margin: 0 0 14px;
  color: var(--fg-primary);
  font-size: 13px;
  font-weight: 600;
}

.shortcut-dialog label,
.shortcut-dialog p {
  color: var(--fg-muted);
  font-size: 11px;
}

.shortcut-input {
  width: 100%;
  margin-top: 6px;
  padding: 8px 10px;
  font-family: monospace;
  font-size: 12px;
}

.shortcut-dialog p {
  margin: 6px 0 0;
}

.secondary-button,
.primary-button {
  padding: 6px 11px;
  border: 0;
  border-radius: 6px;
  font-size: 12px;
}

.secondary-button {
  background: var(--bg-hover);
  color: var(--fg-secondary);
}

.primary-button {
  background: var(--accent);
  color: white;
}
</style>
