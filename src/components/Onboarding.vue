<template>
  <div v-if="show" class="fixed inset-0 bg-black/70 flex items-center justify-center z-50">
    <div class="bg-gray-800 rounded-xl w-[520px] border border-gray-600 shadow-2xl overflow-hidden">
      <!-- Step Content -->
      <div class="p-8 text-center">
        <div class="text-6xl mb-4">{{ steps[current].icon }}</div>
        <h2 class="text-xl font-medium text-gray-100 mb-2">{{ steps[current].title }}</h2>
        <p class="text-sm text-gray-400 leading-relaxed max-w-sm mx-auto">{{ steps[current].desc }}</p>

        <!-- Feature Preview -->
        <div v-if="steps[current].preview" class="mt-4 bg-gray-900 rounded-lg p-4 text-left">
          <div class="text-xs text-gray-500 mb-2">{{ steps[current].preview.title }}</div>
          <div class="flex flex-wrap gap-2">
            <kbd v-for="key in steps[current].preview.keys" :key="key" class="px-2 py-0.5 bg-gray-700 border border-gray-600 rounded text-xs text-gray-300 font-mono">{{ key }}</kbd>
          </div>
        </div>
      </div>

      <!-- Navigation -->
      <div class="px-8 pb-6 flex items-center justify-between">
        <button v-if="current > 0" @click="current--" class="text-sm text-gray-400 hover:text-white">← 上一步</button>
        <div v-else />

        <!-- Dots -->
        <div class="flex gap-1.5">
          <div v-for="(_, i) in steps" :key="i" class="w-2 h-2 rounded-full transition-colors" :class="i === current ? 'bg-blue-500' : 'bg-gray-600'" />
        </div>

        <button v-if="current < steps.length - 1" @click="current++" class="px-4 py-1.5 text-sm bg-blue-600 hover:bg-blue-500 rounded text-white">下一步 →</button>
        <button v-else @click="finish" class="px-4 py-1.5 text-sm bg-green-600 hover:bg-green-500 rounded text-white">开始使用 🚀</button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'

const show = ref(false)
const current = ref(0)

const steps = [
  {
    icon: '👋',
    title: '欢迎使用 XTerminal Pro',
    desc: '轻量级 SSH 终端工具，基于 Tauri 构建，内存占用仅 30MB。',
  },
  {
    icon: '🔌',
    title: '连接你的服务器',
    desc: '点击左侧 + 添加 SSH 连接，支持密码和密钥认证。',
  },
  {
    icon: '⌨️',
    title: '强大的终端体验',
    desc: '支持分屏、搜索、命令历史、自动复制等高级功能。',
    preview: { title: '快捷键', keys: ['Ctrl+Shift+F 搜索', 'Ctrl+L 清屏', 'Ctrl+C 中断', '↑↓ 历史'] },
  },
  {
    icon: '📁',
    title: 'SFTP 文件管理',
    desc: '点击顶部「文件」切换到双栏文件管理器，轻松上传下载。',
  },
  {
    icon: '⚡',
    title: '批量命令 & 端口转发',
    desc: '选中多台服务器同时执行命令，或配置端口转发访问内网服务。',
  },
]

onMounted(() => {
  const guided = localStorage.getItem('xterminal_guided')
  if (!guided) show.value = true
})

function finish() {
  localStorage.setItem('xterminal_guided', 'true')
  show.value = false
}
</script>
