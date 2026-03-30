<template>
  <div class="fixed inset-0 bg-black/60 flex items-center justify-center z-50" @click.self="$emit('close')">
    <div class="bg-gray-800 rounded-lg w-[400px] border border-gray-600 p-4 space-y-4">
      <h3 class="text-sm font-medium text-gray-200">连接配置导入/导出</h3>

      <div class="space-y-3">
        <button
          @click="exportConnections"
          class="w-full px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded text-sm text-gray-200 flex items-center gap-2"
        >
          <span>📤</span> 导出连接配置（JSON）
        </button>

        <div
          @drop.prevent="handleDrop"
          @dragover.prevent="isDragging = true"
          @dragleave="isDragging = false"
          class="w-full px-4 py-6 border-2 border-dashed rounded text-center cursor-pointer transition-colors"
          :class="isDragging ? 'border-blue-500 bg-blue-500/10' : 'border-gray-600 hover:border-gray-500'"
          @click="$refs.fileInput.click()"
        >
          <input ref="fileInput" type="file" accept=".json" class="hidden" @change="handleFileSelect" />
          <span class="text-2xl block mb-2">📥</span>
          <span class="text-sm text-gray-400">拖拽 JSON 文件到此处或点击选择</span>
        </div>

        <div v-if="importResult" class="text-xs px-3 py-2 rounded" :class="importResult.success ? 'bg-green-900/50 text-green-300' : 'bg-red-900/50 text-red-300'">
          {{ importResult.message }}
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
import { invoke } from '../utils/tauri.js'

const emit = defineEmits(['close', 'imported'])

const isDragging = ref(false)
const importResult = ref(null)

async function exportConnections() {
  try {
    const connections = await invoke('load_connections')
    const blob = new Blob([JSON.stringify(connections, null, 2)], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `xterminal-connections-${new Date().toISOString().slice(0, 10)}.json`
    a.click()
    URL.revokeObjectURL(url)
    importResult.value = { success: true, message: `✓ 已导出 ${connections.length} 个连接配置` }
  } catch (err) {
    importResult.value = { success: false, message: `✗ 导出失败: ${err}` }
  }
}

function handleDrop(e) {
  isDragging.value = false
  const file = e.dataTransfer.files[0]
  if (file) processFile(file)
}

function handleFileSelect(e) {
  const file = e.target.files[0]
  if (file) processFile(file)
}

async function processFile(file) {
  if (!file.name.endsWith('.json')) {
    importResult.value = { success: false, message: '✗ 请选择 JSON 文件' }
    return
  }

  try {
    const text = await file.text()
    const connections = JSON.parse(text)

    if (!Array.isArray(connections)) {
      throw new Error('无效的连接配置格式')
    }

    for (const conn of connections) {
      await invoke('save_connection', { conn })
    }

    importResult.value = { success: true, message: `✓ 已导入 ${connections.length} 个连接配置` }
    emit('imported', connections)
  } catch (err) {
    importResult.value = { success: false, message: `✗ 导入失败: ${err.message}` }
  }
}
</script>
