<template>
  <div class="h-full flex flex-col bg-gray-900">
    <div class="h-9 bg-gray-800 border-b border-gray-700 flex items-center px-3">
      <span class="text-sm font-medium text-gray-300">设置</span>
    </div>

    <div class="flex-1 overflow-y-auto p-4 space-y-6">
      <!-- 外观 -->
      <section>
        <h3 class="text-xs text-gray-500 uppercase mb-3">外观</h3>
        <div class="space-y-3">
          <div class="flex items-center justify-between">
            <div>
              <div class="text-sm text-gray-200">暗色主题</div>
              <div class="text-xs text-gray-500">切换暗色/亮色主题</div>
            </div>
            <button
              @click="settings.darkMode = !settings.darkMode"
              class="w-10 h-5 rounded-full relative transition-colors"
              :class="settings.darkMode ? 'bg-blue-600' : 'bg-gray-600'"
            >
              <span
                class="absolute top-0.5 w-4 h-4 rounded-full bg-white transition-all"
                :class="settings.darkMode ? 'left-5' : 'left-0.5'"
              />
            </button>
          </div>

          <div class="flex items-center justify-between">
            <div>
              <div class="text-sm text-gray-200">字体大小</div>
              <div class="text-xs text-gray-500">终端字体大小</div>
            </div>
            <div class="flex items-center gap-2">
              <button @click="settings.fontSize = Math.max(10, settings.fontSize - 1)" class="w-6 h-6 bg-gray-700 rounded text-gray-300 text-sm">-</button>
              <span class="text-sm text-gray-300 w-8 text-center">{{ settings.fontSize }}</span>
              <button @click="settings.fontSize = Math.min(24, settings.fontSize + 1)" class="w-6 h-6 bg-gray-700 rounded text-gray-300 text-sm">+</button>
            </div>
          </div>

          <div class="flex items-center justify-between">
            <div>
              <div class="text-sm text-gray-200">光标样式</div>
              <div class="text-xs text-gray-500">终端光标闪烁</div>
            </div>
            <button
              @click="settings.cursorBlink = !settings.cursorBlink"
              class="w-10 h-5 rounded-full relative transition-colors"
              :class="settings.cursorBlink ? 'bg-blue-600' : 'bg-gray-600'"
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
        <h3 class="text-xs text-gray-500 uppercase mb-3">终端</h3>
        <div class="space-y-3">
          <div class="flex items-center justify-between">
            <div>
              <div class="text-sm text-gray-200">滚动缓冲区</div>
              <div class="text-xs text-gray-500">保留的行数</div>
            </div>
            <select v-model.number="settings.scrollback" class="bg-gray-800 border border-gray-600 rounded px-2 py-1 text-sm text-gray-300">
              <option :value="1000">1000</option>
              <option :value="5000">5000</option>
              <option :value="10000">10000</option>
              <option :value="50000">50000</option>
            </select>
          </div>

          <div class="flex items-center justify-between">
            <div>
              <div class="text-sm text-gray-200">SSH 连接超时</div>
              <div class="text-xs text-gray-500">秒</div>
            </div>
            <input v-model.number="settings.sshTimeout" type="number" min="5" max="120" class="w-16 bg-gray-800 border border-gray-600 rounded px-2 py-1 text-sm text-gray-300 text-center" />
          </div>
        </div>
      </section>

      <!-- 快捷键 -->
      <section>
        <h3 class="text-xs text-gray-500 uppercase mb-3">快捷键</h3>
        <div class="space-y-2 text-sm">
          <div v-for="key in shortcuts" :key="key.name" class="flex items-center justify-between py-1">
            <span class="text-gray-300">{{ key.name }}</span>
            <kbd class="px-2 py-0.5 bg-gray-800 border border-gray-600 rounded text-xs text-gray-400 font-mono">{{ key.key }}</kbd>
          </div>
        </div>
      </section>

      <!-- 关于 -->
      <section>
        <h3 class="text-xs text-gray-500 uppercase mb-3">关于</h3>
        <div class="bg-gray-800 rounded p-3 space-y-2">
          <div class="text-sm text-gray-200">XTerminal Pro v0.1.0</div>
          <div class="text-xs text-gray-500">轻量级SSH终端 + SFTP + 多服务器管理</div>
          <div class="text-xs text-gray-500">基于 Tauri v2 + Vue 3 + xterm.js</div>
          <div class="text-xs text-gray-500">内存占用: ~30MB | 安装包: ~8MB</div>
          <div class="text-xs text-gray-500 mt-2">开发者: 龙虾003 🦞</div>
        </div>
      </section>
    </div>
  </div>
</template>

<script setup>
import { reactive } from 'vue'

const settings = reactive({
  darkMode: true,
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
</script>
