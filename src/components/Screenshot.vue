<template>
  <div v-if="visible" class="fixed inset-0 bg-black/60 flex items-center justify-center z-50" @click.self="$emit('close')">
    <div class="bg-gray-800 rounded-lg w-96 border border-gray-600 p-4 space-y-3">
      <h3 class="text-sm font-medium text-gray-200">📸 终端截图</h3>

      <div class="space-y-2">
        <button @click="captureFull" class="w-full px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded text-sm text-gray-200 flex items-center gap-2">
          <span>🖥️</span> 截取整个终端
        </button>
        <button @click="captureSelection" class="w-full px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded text-sm text-gray-200 flex items-center gap-2">
          <span>✂️</span> 截取选中区域
        </button>
        <button @click="captureVisible" class="w-full px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded text-sm text-gray-200 flex items-center gap-2">
          <span>👁️</span> 截取可见区域
        </button>
      </div>

      <div v-if="preview" class="mt-3">
        <img :src="preview" class="w-full rounded border border-gray-600" />
        <div class="flex gap-2 mt-2">
          <button @click="copyToClipboard" class="flex-1 px-3 py-1.5 text-sm bg-blue-600 hover:bg-blue-500 rounded text-white">📋 复制到剪贴板</button>
          <button @click="download" class="flex-1 px-3 py-1.5 text-sm bg-gray-700 hover:bg-gray-600 rounded text-gray-200">💾 保存文件</button>
        </div>
      </div>

      <div class="flex justify-end">
        <button @click="$emit('close')" class="px-4 py-1.5 text-sm text-gray-400 hover:text-white">关闭</button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue'

defineProps({ visible: Boolean })
defineEmits(['close'])

const preview = ref(null)

function captureVisible() {
  // In a real app, would capture the terminal canvas
  // For now, create a placeholder
  const canvas = document.createElement('canvas')
  canvas.width = 800
  canvas.height = 400
  const ctx = canvas.getContext('2d')
  ctx.fillStyle = '#1e1e1e'
  ctx.fillRect(0, 0, 800, 400)
  ctx.fillStyle = '#0dbc79'
  ctx.font = '14px monospace'
  ctx.fillText('$ 截图功能演示', 20, 30)
  ctx.fillStyle = '#d4d4d4'
  ctx.fillText('终端内容将在此显示', 20, 50)
  preview.value = canvas.toDataURL('image/png')
}

function captureSelection() { captureVisible() }
function captureAll() { captureVisible() }

function copyToClipboard() {
  if (!preview.value) return
  // Convert data URL to blob and copy
  const data = preview.value.split(',')[1]
  const binary = atob(data)
  const array = new Uint8Array(binary.length)
  for (let i = 0; i < binary.length; i++) array[i] = binary.charCodeAt(i)
  const blob = new Blob([array], { type: 'image/png' })
  navigator.clipboard.write([new ClipboardItem({ 'image/png': blob })]).catch(() => {})
}

function download() {
  if (!preview.value) return
  const a = document.createElement('a')
  a.href = preview.value
  a.download = `xterminal-screenshot-${Date.now()}.png`
  a.click()
}
</script>
