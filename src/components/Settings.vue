<template>
  <div class="h-full flex flex-col" style="background: var(--bg-base);">
    <div class="h-9 flex items-center px-3" style="background: var(--bg-surface); border-bottom: 1px solid var(--border);">
      <span class="text-sm font-medium" style="color: var(--fg-secondary);">设置</span>
    </div>

    <div class="flex-1 overflow-y-auto p-4 space-y-6">
      <!-- 外观 -->
      <section>
        <h3 class="text-xs uppercase mb-3" style="color: var(--fg-muted);">外观</h3>
        <div class="space-y-3">
          <!-- 主题选择 -->
          <div>
            <div class="text-sm mb-2" style="color: var(--fg-primary);">主题</div>
            <div class="flex gap-1">
              <button
                v-for="t in themes"
                :key="t.value"
                @click="setTheme(t.value)"
                class="flex-1 px-2 py-1.5 rounded text-xs font-medium transition-colors border"
                :style="currentTheme === t.value
                  ? 'background: var(--accent); color: white; border-color: var(--accent);'
                  : 'background: var(--bg-surface); color: var(--fg-secondary); border-color: var(--border);'"
              >
                {{ t.label }}
              </button>
            </div>
          </div>

          <!-- 字体大小 -->
          <div class="flex items-center justify-between">
            <div>
              <div class="text-sm" style="color: var(--fg-primary);">字体大小</div>
              <div class="text-xs" style="color: var(--fg-muted);">终端字体大小</div>
            </div>
            <div class="flex items-center gap-2">
              <button @click="settings.fontSize = Math.max(10, settings.fontSize - 1)" class="w-6 h-6 rounded text-sm" style="background: var(--bg-elevated); color: var(--fg-secondary);">-</button>
              <span class="text-sm w-8 text-center" style="color: var(--fg-secondary);">{{ settings.fontSize }}</span>
              <button @click="settings.fontSize = Math.min(24, settings.fontSize + 1)" class="w-6 h-6 rounded text-sm" style="background: var(--bg-elevated); color: var(--fg-secondary);">+</button>
            </div>
          </div>
          <!-- 字体预览 -->
          <div
            class="rounded px-3 py-2 text-center italic"
            :style="{ fontSize: settings.fontSize + 'px', fontFamily: 'monospace', background: 'var(--bg-surface)', color: 'var(--fg-secondary)' }"
          >
            The quick brown fox
          </div>

          <!-- 光标样式 -->
          <div class="flex items-center justify-between">
            <div>
              <div class="text-sm" style="color: var(--fg-primary);">光标样式</div>
              <div class="text-xs" style="color: var(--fg-muted);">终端光标闪烁</div>
            </div>
            <button
              @click="settings.cursorBlink = !settings.cursorBlink"
              class="w-10 h-5 rounded-full relative transition-colors"
              :style="{ background: settings.cursorBlink ? 'var(--accent)' : 'var(--border)' }"
            >
              <span
                class="absolute top-0.5 w-4 h-4 rounded-full bg-white transition-all"
                :class="settings.cursorBlink ? 'left-5' : 'left-0.5'"
              />
            </button>
          </div>
        </div>
      </section>

      <!-- 终端 -->
      <section>
        <h3 class="text-xs uppercase mb-3" style="color: var(--fg-muted);">终端</h3>
        <div class="space-y-3">
          <div class="flex items-center justify-between">
            <div>
              <div class="text-sm" style="color: var(--fg-primary);">滚动缓冲区</div>
              <div class="text-xs" style="color: var(--fg-muted);">保留的行数</div>
            </div>
            <select v-model.number="settings.scrollback" class="border rounded px-2 py-1 text-sm" style="background: var(--bg-surface); color: var(--fg-secondary); border-color: var(--border);">
              <option :value="1000">1000</option>
              <option :value="5000">5000</option>
              <option :value="10000">10000</option>
              <option :value="50000">50000</option>
            </select>
          </div>

          <div class="flex items-center justify-between">
            <div>
              <div class="text-sm" style="color: var(--fg-primary);">SSH 连接超时</div>
              <div class="text-xs" style="color: var(--fg-muted);">秒</div>
            </div>
            <input v-model.number="settings.sshTimeout" type="number" min="5" max="120" class="w-16 border rounded px-2 py-1 text-sm text-center" style="background: var(--bg-surface); color: var(--fg-secondary); border-color: var(--border);" />
          </div>
        </div>
      </section>

      <!-- 快捷键 -->
      <section>
        <h3 class="text-xs uppercase mb-3" style="color: var(--fg-muted);">快捷键</h3>
        <div class="space-y-2 text-sm">
          <div v-for="key in shortcuts" :key="key.name" class="flex items-center justify-between py-1">
            <span style="color: var(--fg-secondary);">{{ key.name }}</span>
            <div class="flex items-center gap-2">
              <kbd class="px-2 py-0.5 border rounded text-xs font-mono" style="background: var(--bg-surface); border-color: var(--border); color: var(--fg-muted);">{{ key.key }}</kbd>
              <button
                @click="editShortcut(key)"
                class="px-1.5 py-0.5 rounded text-xs transition-colors"
                style="color: var(--accent);"
                title="编辑快捷键"
              >
                ✏️
              </button>
            </div>
          </div>
        </div>
      </section>

      <!-- 快捷键编辑对话框 -->
      <div v-if="editingShortcut" class="fixed inset-0 z-50 flex items-center justify-center" style="background: rgba(0,0,0,0.7);" @click.self="editingShortcut = null">
        <div class="rounded-lg p-6 w-80" style="background: var(--bg-surface); border: 1px solid var(--border);">
'          <h3 class="text-sm font-medium mb-4" style="color: var(--fg-primary);">编辑快捷键: {{ editingShortcut.name }}</h3>
          <div class="mb-4">
            <label class="text-xs block mb-2" style="color: var(--fg-secondary);">按下新的快捷键组合</label>
            <input
              ref="shortcutInput"
              v-model="newShortcutKey"
              @keydown="captureShortcut"
              class="w-full px-3 py-2 rounded text-sm font-mono"
              style="background: var(--bg-elevated); border: 1px solid var(--border); color: var(--fg-primary);"
              placeholder="例如: Ctrl+Shift+K"
              autocomplete="off"
            />
            <p class="text-[10px] mt-2" style="color: var(--fg-muted);">按 Esc 取消，按 Enter 确认</p>
          </div>
          <div class="flex justify-end gap-2">
            <button @click="editingShortcut = null" class="px-3 py-1.5 rounded text-xs" style="background: var(--bg-hover); color: var(--fg-secondary);">取消</button>
            <button @click="saveShortcut" class="px-3 py-1.5 rounded text-xs" style="background: var(--accent); color: white;">保存</button>
          </div>
        </div>
      </div>

      <!-- 代理设置 -->
      <section>
        <h3 class="text-xs uppercase mb-3" style="color: var(--fg-muted);">网络</h3>
        <button
          @click="$emit('open-proxy-settings')"
          class="w-full px-3 py-2 rounded text-sm font-medium transition-colors border"
          style="background: var(--bg-surface); color: var(--fg-secondary); border-color: var(--border);"
        >
          🌐 打开代理设置
        </button>
      </section>

      <!-- 关于 -->
      <section>
        <h3 class="text-xs uppercase mb-3" style="color: var(--fg-muted);">关于</h3>
        <div class="rounded p-3 space-y-2" style="background: var(--bg-surface);">
          <div class="text-sm" style="color: var(--fg-primary);">XTerminal Pro v0.1.0</div>
          <div class="text-xs" style="color: var(--fg-muted);">轻量级SSH终端 + SFTP + 多服务器管理</div>
          <div class="text-xs" style="color: var(--fg-muted);">基于 Tauri v2 + Vue 3 + xterm.js</div>
          <div class="text-xs" style="color: var(--fg-muted);">内存占用: ~30MB | 安装包: ~8MB</div>
          <div class="text-xs mt-2" style="color: var(--fg-muted);">开发者: 龙虾003 🦞</div>
        </div>
      </section>
    </div>
  </div>
</template>

<script setup>
import { reactive, ref, nextTick } from 'vue'

defineEmits(['open-proxy-settings'])

const themes = [
  { value: 'dark', label: '🌙 暗色' },
  { value: 'light', label: '☀️ 亮色' },
  { value: 'nord', label: '❄️ Nord' },
]

const currentTheme = ref(localStorage.getItem('xterminal_theme') || 'dark')

const settings = reactive({
  darkMode: currentTheme.value !== 'light',
  fontSize: 14,
  cursorBlink: true,
  scrollback: 10000,
  sshTimeout: 30,
})

const shortcuts = [
  { name: '搜索', key: 'Ctrl+Shift+F' },
  { name: '清屏', key: 'Ctrl+L' },
  { name: '中断', key: 'Ctrl+C' },
  { name: '新建标签', key: 'Ctrl+T' },
  { name: '关闭标签', key: 'Ctrl+W' },
  { name: '分屏', key: 'Ctrl+Shift+D' },
  { name: '全屏', key: 'F11' },
]

function setTheme(value) {
  currentTheme.value = value
  document.documentElement.setAttribute('data-theme', value)
  localStorage.setItem('xterminal_theme', value)
  settings.darkMode = value !== 'light'
}

const editingShortcut = ref(null)
const newShortcutKey = ref('')
const shortcutInput = ref(null)

function editShortcut(key) {
  editingShortcut.value = key
  newShortcutKey.value = key.key
  nextTick(() => shortcutInput.value?.focus())
}

function captureShortcut(e) {
  e.preventDefault()
  const modifiers = []
  if (e.ctrlKey) modifiers.push('Ctrl')
  if (e.altKey) modifiers.push('Alt')
  if (e.shiftKey) modifiers.push('Shift')
  if (e.metaKey) modifiers.push('Meta')
  
  // Ignore modifier-only combos
  if (!modifiers.length && e.key.length > 1) return
  
  const keyName = e.key.length === 1 ? e.key.toUpperCase() : e.key
  newShortcutKey.value = [...modifiers, keyName].join('+')
  
  if (e.key === 'Escape') {
    editingShortcut.value = null
  } else if (e.key === 'Enter') {
    saveShortcut()
  }
}

function saveShortcut() {
  if (editingShortcut.value && newShortcutKey.value) {
    editingShortcut.value.key = newShortcutKey.value
    localStorage.setItem(`shortcut_${editingShortcut.value.name}`, newShortcutKey.value)
  }
  editingShortcut.value = null
}
</script>
