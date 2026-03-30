<template>
  <div class="flex flex-col h-full bg-gray-900">
    <!-- Toolbar -->
    <div class="h-9 bg-gray-800 border-b border-gray-700 flex items-center px-2 gap-2">
      <button
        @click="mode = mode === 'sftp' ? 'terminal' : 'sftp'"
        class="px-2 py-0.5 text-xs rounded"
        :class="mode === 'sftp' ? 'bg-blue-600 text-white' : 'bg-gray-700 text-gray-300 hover:bg-gray-600'"
      >{{ mode === 'sftp' ? '📁 SFTP' : '⌨️ 终端' }}</button>
      <div class="flex-1" />
      <span v-if="mode === 'sftp'" class="text-xs text-gray-500">
        {{ connection?.name || '未连接' }}
      </span>
    </div>

    <!-- SFTP Mode -->
    <div v-if="mode === 'sftp'" class="flex-1 flex">
      <!-- Local Panel -->
      <div class="flex-1 flex flex-col border-r border-gray-700">
        <div class="h-8 bg-gray-800 border-b border-gray-700 flex items-center px-2 gap-1">
          <span class="text-xs text-green-400">🏠 本地</span>
          <input
            v-model="localPath"
            @keydown.enter="loadLocal"
            class="flex-1 bg-gray-900 text-xs text-gray-300 px-2 py-0.5 rounded border border-gray-600 focus:outline-none focus:border-blue-500"
          />
          <button @click="loadLocal" class="text-xs text-gray-400 hover:text-white">⟳</button>
        </div>
        <div class="flex-1 overflow-y-auto">
          <div
            v-for="file in localFiles"
            :key="file.path"
            @click="onLocalClick(file)"
            @dblclick="onLocalDblClick(file)"
            class="flex items-center gap-2 px-3 py-1 cursor-pointer hover:bg-gray-700 text-sm"
            :class="selectedLocal === file.path ? 'bg-blue-600/20' : ''"
          >
            <span class="w-5 text-center">{{ file.is_dir ? '📁' : getFileIcon(file.name) }}</span>
            <span class="flex-1 truncate text-gray-300">{{ file.name }}</span>
            <span class="text-xs text-gray-500 w-16 text-right">{{ file.is_dir ? '' : formatSize(file.size) }}</span>
            <span class="text-xs text-gray-600 w-28 text-right">{{ file.modified || '' }}</span>
          </div>
          <div v-if="localFiles.length === 0" class="p-4 text-center text-gray-500 text-sm">空目录</div>
        </div>
      </div>

      <!-- Action Buttons -->
      <div class="w-10 bg-gray-800 flex flex-col items-center justify-center gap-2 border-r border-gray-700">
        <button
          @click="uploadSelected"
          :disabled="!selectedLocal"
          class="w-8 h-8 rounded flex items-center justify-center text-lg"
          :class="selectedLocal ? 'bg-blue-600 hover:bg-blue-500 text-white' : 'bg-gray-700 text-gray-500'"
          title="上传 →"
        >→</button>
        <button
          @click="downloadSelected"
          :disabled="!selectedRemote"
          class="w-8 h-8 rounded flex items-center justify-center text-lg"
          :class="selectedRemote ? 'bg-blue-600 hover:bg-blue-500 text-white' : 'bg-gray-700 text-gray-500'"
          title="← 下载"
        >←</button>
      </div>

      <!-- Remote Panel -->
      <div class="flex-1 flex flex-col">
        <div class="h-8 bg-gray-800 border-b border-gray-700 flex items-center px-2 gap-1">
          <span class="text-xs text-blue-400">🌐 远程</span>
          <input
            v-model="remotePath"
            @keydown.enter="loadRemote"
            class="flex-1 bg-gray-900 text-xs text-gray-300 px-2 py-0.5 rounded border border-gray-600 focus:outline-none focus:border-blue-500"
          />
          <button @click="loadRemote" class="text-xs text-gray-400 hover:text-white">⟳</button>
        </div>
        <div class="flex-1 overflow-y-auto">
          <div
            v-for="file in remoteFiles"
            :key="file.path"
            @click="onRemoteClick(file)"
            @dblclick="onRemoteDblClick(file)"
            class="flex items-center gap-2 px-3 py-1 cursor-pointer hover:bg-gray-700 text-sm"
            :class="selectedRemote === file.path ? 'bg-blue-600/20' : ''"
          >
            <span class="w-5 text-center">{{ file.is_dir ? '📁' : getFileIcon(file.name) }}</span>
            <span class="flex-1 truncate text-gray-300">{{ file.name }}</span>
            <span class="text-xs text-gray-500 w-16 text-right">{{ file.is_dir ? '' : formatSize(file.size) }}</span>
            <span class="text-xs text-gray-600 w-28 text-right">{{ file.modified || '' }}</span>
          </div>
          <div v-if="remoteFiles.length === 0" class="p-4 text-center text-gray-500 text-sm">
            {{ sessionId ? '空目录' : '未连接' }}
          </div>
        </div>
      </div>
    </div>

    <!-- Transfer Queue -->
    <div v-if="mode === 'sftp' && transfers.length > 0" class="h-24 bg-gray-850 border-t border-gray-700 overflow-y-auto" style="background: #1a1a1a;">
      <div class="px-2 py-1 text-xs text-gray-500 border-b border-gray-700">传输队列 ({{ transfers.length }})</div>
      <div v-for="t in transfers" :key="t.id" class="px-2 py-1 flex items-center gap-2 text-xs">
        <span :class="t.status === 'completed' ? 'text-green-400' : t.status === 'failed' ? 'text-red-400' : 'text-yellow-400'">
          {{ t.status === 'completed' ? '✓' : t.status === 'failed' ? '✗' : '⏳' }}
        </span>
        <span class="flex-1 truncate text-gray-300">{{ t.filename }}</span>
        <span class="text-gray-500">{{ formatSize(t.transferred) }}/{{ formatSize(t.total) }}</span>
        <div v-if="t.status === 'transferring'" class="w-20 h-1.5 bg-gray-700 rounded-full overflow-hidden">
          <div class="h-full bg-blue-500 rounded-full" :style="{ width: (t.transferred / t.total * 100) + '%' }" />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, watch } from 'vue'
import { invoke } from '../utils/tauri.js'

const props = defineProps({
  connection: Object,
  sessionId: String,
  active: Boolean,
})

const mode = ref('sftp')
const localPath = ref('/home/user')
const remotePath = ref('/')
const localFiles = ref([])
const remoteFiles = ref([])
const selectedLocal = ref(null)
const selectedRemote = ref(null)
const transfers = ref([])

function formatSize(bytes) {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i]
}

function getFileIcon(name) {
  const ext = name.split('.').pop()?.toLowerCase()
  const icons = {
    js: '📜', ts: '📘', py: '🐍', rs: '🦀', go: '🐹',
    md: '📝', txt: '📄', json: '📋', yaml: '📋', yml: '📋',
    html: '🌐', css: '🎨', vue: '💚',
    png: '🖼️', jpg: '🖼️', jpeg: '🖼️', gif: '🖼️', svg: '🖼️',
    zip: '📦', tar: '📦', gz: '📦', '7z': '📦',
    mp3: '🎵', mp4: '🎬', avi: '🎬',
    sh: '⚡', bat: '⚡', exe: '⚙️',
  }
  return icons[ext] || '📄'
}

async function loadLocal() {
  try {
    localFiles.value = await invoke('sftp_list_local', { path: localPath.value })
  } catch (e) {
    localFiles.value = []
  }
}

async function loadRemote() {
  if (!props.sessionId) return
  try {
    remoteFiles.value = await invoke('sftp_list_remote', {
      sessionId: props.sessionId,
      path: remotePath.value,
    })
  } catch (e) {
    remoteFiles.value = []
  }
}

function onLocalClick(file) {
  selectedLocal.value = file.path
}

function onLocalDblClick(file) {
  if (file.is_dir) {
    localPath.value = file.path
    loadLocal()
  }
}

function onRemoteClick(file) {
  selectedRemote.value = file.path
}

function onRemoteDblClick(file) {
  if (file.is_dir) {
    remotePath.value = file.path
    loadRemote()
  }
}

function uploadSelected() {
  if (!selectedLocal.value) return
  const filename = selectedLocal.value.split('/').pop()
  const transfer = {
    id: Date.now().toString(),
    filename,
    total: 1024 * 1024,
    transferred: 0,
    status: 'transferring',
  }
  transfers.value.push(transfer)

  // Simulate transfer
  const interval = setInterval(() => {
    transfer.transferred += 1024 * 64
    if (transfer.transferred >= transfer.total) {
      transfer.transferred = transfer.total
      transfer.status = 'completed'
      clearInterval(interval)
    }
  }, 100)
}

function downloadSelected() {
  if (!selectedRemote.value) return
  const filename = selectedRemote.value.split('/').pop()
  const transfer = {
    id: Date.now().toString(),
    filename,
    total: 512 * 1024,
    transferred: 0,
    status: 'transferring',
  }
  transfers.value.push(transfer)

  const interval = setInterval(() => {
    transfer.transferred += 1024 * 32
    if (transfer.transferred >= transfer.total) {
      transfer.transferred = transfer.total
      transfer.status = 'completed'
      clearInterval(interval)
    }
  }, 100)
}

onMounted(() => {
  loadLocal()
})

watch(() => props.sessionId, (sid) => {
  if (sid) loadRemote()
})
</script>
