<template>
  <div class="flex flex-col h-full bg-gray-900">
    <!-- Toolbar -->
    <div class="h-9 bg-gray-800 border-b border-gray-700 flex items-center px-2 gap-2">
      <button @click="createRemoteDir" :disabled="!sessionId" class="px-2 py-0.5 text-xs rounded bg-gray-700 text-gray-300 hover:bg-gray-600 disabled:opacity-50">📁+</button>
      <button @click="batchDelete" :disabled="!sessionId || selectedRemoteSet.size === 0" class="px-2 py-0.5 text-xs rounded bg-gray-700 text-gray-300 hover:bg-red-600/50 disabled:opacity-50">🗑 {{ selectedRemoteSet.size || '' }}</button>
      <button @click="refreshRemote" :disabled="!sessionId" class="px-2 py-0.5 text-xs rounded bg-gray-700 text-gray-300 hover:bg-gray-600 disabled:opacity-50">⟳</button>
      <div class="flex-1" />
      <span class="text-xs text-gray-500">{{ selectedRemoteSet.size ? `已选 ${selectedRemoteSet.size} 个` : '' }}</span>
      <span class="text-xs text-gray-500">{{ connection?.name || '未连接' }}</span>
    </div>

    <!-- SFTP Panels -->
    <div class="flex-1 flex">
      <!-- Local Panel -->
      <div class="flex flex-col border-r border-gray-700" style="width: 50%;">
        <div class="h-8 bg-gray-800 border-b border-gray-700 flex items-center px-2 gap-1">
          <span class="text-xs text-green-400">🏠 本地</span>
          <input v-model="localPath" @keydown.enter="loadLocal" class="flex-1 bg-gray-900 text-xs text-gray-300 px-2 py-0.5 rounded border border-gray-600 focus:outline-none focus:border-blue-500" />
          <button @click="loadLocal" class="text-xs text-gray-400 hover:text-white">⟳</button>
        </div>
        <div class="flex-1 overflow-y-auto" @click.self="selectedLocalSet.clear()">
          <div v-for="file in localFiles" :key="file.path"
            @click="onLocalClick($event, file)"
            @dblclick="onLocalDblClick(file)"
            @dragstart="onLocalDragStart($event, file)"
            draggable="true"
            class="flex items-center gap-2 px-3 py-1 cursor-pointer hover:bg-gray-700 text-sm select-none"
            :class="selectedLocalSet.has(file.path) ? 'bg-blue-600/20 text-blue-300' : ''">
            <span class="w-5 text-center">{{ file.is_dir ? '📁' : icon(file.name) }}</span>
            <span class="flex-1 truncate text-gray-300">{{ file.name }}</span>
            <span class="text-xs text-gray-500 w-16 text-right">{{ file.is_dir ? '' : fmtSize(file.size) }}</span>
          </div>
        </div>
      </div>

      <!-- Transfer Buttons -->
      <div class="w-10 bg-gray-800 flex flex-col items-center justify-center gap-2 border-r border-gray-700">
        <button @click="doUpload" :disabled="selectedLocalSet.size === 0 && !selectedLocal || !sessionId"
          class="w-8 h-8 rounded text-lg disabled:opacity-30 bg-blue-600 hover:bg-blue-500 text-white"
          :title="selectedLocalSet.size > 1 ? `上传 ${selectedLocalSet.size} 个文件` : '上传'">→</button>
        <button @click="doDownload" :disabled="selectedRemoteSet.size === 0 && !selectedRemote || !sessionId"
          class="w-8 h-8 rounded text-lg disabled:opacity-30 bg-blue-600 hover:bg-blue-500 text-white"
          :title="selectedRemoteSet.size > 1 ? `下载 ${selectedRemoteSet.size} 个文件` : '下载'">←</button>
      </div>

      <!-- Remote Panel -->
      <div class="flex-1 flex flex-col"
        @dragover.prevent="remoteDragOver = true"
        @dragleave="remoteDragOver = false"
        @drop.prevent="onRemoteDrop"
        :class="remoteDragOver ? 'ring-2 ring-blue-500/50' : ''">
        <div class="h-8 bg-gray-800 border-b border-gray-700 flex items-center px-2 gap-1">
          <span class="text-xs text-blue-400">🌐 远程</span>
          <input v-model="remotePath" @keydown.enter="loadRemote" class="flex-1 bg-gray-900 text-xs text-gray-300 px-2 py-0.5 rounded border border-gray-600 focus:outline-none focus:border-blue-500" />
          <button @click="loadRemote" class="text-xs text-gray-400 hover:text-white">⟳</button>
        </div>
        <div class="flex-1 overflow-y-auto" @click.self="selectedRemoteSet.clear()">
          <div v-for="file in remoteFiles" :key="file.path"
            @click="onRemoteClick($event, file)"
            @dblclick="onRemoteDblClick(file)"
            @contextmenu.prevent="showRemoteMenu($event, file)"
            @dragstart="onRemoteDragStart($event, file)"
            @dragover.prevent="file._dragOver = true"
            @dragleave="file._dragOver = false"
            @drop.prevent.stop="onRemoteDirDrop($event, file)"
            :draggable="true"
            class="flex items-center gap-2 px-3 py-1 cursor-pointer hover:bg-gray-700 text-sm select-none"
            :class="[
              selectedRemoteSet.has(file.path) ? 'bg-blue-600/20 text-blue-300' : '',
              file._dragOver && file.is_dir ? 'bg-blue-500/20 ring-1 ring-blue-400' : ''
            ]">
            <span class="w-5 text-center">{{ file.is_dir ? '📁' : icon(file.name) }}</span>
            <span class="flex-1 truncate text-gray-300">{{ file.name }}</span>
            <span class="text-xs text-gray-500 w-16 text-right">{{ file.is_dir ? '' : fmtSize(file.size) }}</span>
            <span class="text-xs text-gray-600 w-20 text-right">{{ file.permissions || '' }}</span>
          </div>
          <div v-if="remoteFiles.length === 0" class="p-4 text-center text-gray-500 text-sm">{{ sessionId ? '空目录' : '未连接' }}</div>
        </div>
      </div>
    </div>

    <!-- Transfer Queue -->
    <div v-if="transfers.length > 0" class="h-20 border-t border-gray-700 overflow-y-auto" style="background: #1a1a1a;">
      <div class="px-2 py-1 text-xs text-gray-500 border-b border-gray-700 flex items-center justify-between">
        <span>传输队列 ({{ transfers.filter(t => t.status === 'pending').length }} 进行中)</span>
        <button @click="transfers = transfers.filter(t => t.status === 'pending')" class="text-gray-500 hover:text-white">清除已完成</button>
      </div>
      <div v-for="t in transfers" :key="t.id" class="px-2 py-1 flex items-center gap-2 text-xs">
        <span :class="t.status === 'done' ? 'text-green-400' : t.status === 'error' ? 'text-red-400' : 'text-yellow-400'">
          {{ t.status === 'done' ? '✓' : t.status === 'error' ? '✗' : '⏳' }}
        </span>
        <span class="flex-1 truncate text-gray-300">{{ t.label }}</span>
        <span v-if="t.status === 'pending'" class="text-gray-500">{{ t.progress || '' }}</span>
        <button v-if="t.status !== 'pending'" @click="transfers = transfers.filter(x => x.id !== t.id)" class="text-gray-500 hover:text-white">✕</button>
      </div>
    </div>

    <!-- Context Menu -->
    <div v-if="ctxMenu.show" :style="{ left: ctxMenu.x + 'px', top: ctxMenu.y + 'px' }" class="fixed bg-gray-800 border border-gray-600 rounded shadow-lg z-50 py-1 text-xs min-w-[140px]" @click.stop>
      <div @click="ctxEdit" class="px-4 py-1.5 hover:bg-gray-600 cursor-pointer text-gray-300">📝 在线编辑</div>
      <div @click="ctxRename" class="px-4 py-1.5 hover:bg-gray-600 cursor-pointer text-gray-300">✏️ 重命名</div>
      <div @click="ctxDownload" class="px-4 py-1.5 hover:bg-gray-600 cursor-pointer text-gray-300">📥 下载</div>
      <div class="border-t border-gray-700 my-1" />
      <div @click="ctxChmod" class="px-4 py-1.5 hover:bg-gray-600 cursor-pointer text-gray-300">🔒 修改权限</div>
      <div @click="ctxCopyPath" class="px-4 py-1.5 hover:bg-gray-600 cursor-pointer text-gray-300">📋 复制路径</div>
      <div class="border-t border-gray-700 my-1" />
      <div @click="ctxDelete" class="px-4 py-1.5 hover:bg-gray-600 cursor-pointer text-red-400">🗑 删除</div>
    </div>

    <!-- Edit Modal -->
    <div v-if="editFile.show" class="fixed inset-0 bg-black/60 flex items-center justify-center z-50" @click.self="editFile.show = false">
      <div class="bg-gray-800 rounded-lg shadow-xl w-[700px] max-h-[80vh] flex flex-col">
        <div class="flex items-center justify-between px-4 py-2 border-b border-gray-700">
          <span class="text-sm text-gray-300">编辑: {{ editFile.path }}</span>
          <span v-if="editFile.dirty" class="text-xs text-yellow-400">● 未保存</span>
          <button @click="editFile.show = false" class="text-gray-400 hover:text-white">✕</button>
        </div>
        <textarea v-model="editFile.content" @input="editFile.dirty = true" class="flex-1 bg-gray-900 text-gray-200 text-sm p-4 font-mono resize-none focus:outline-none min-h-[300px]" spellcheck="false" />
        <div class="flex justify-end gap-2 px-4 py-2 border-t border-gray-700">
          <button @click="saveEdit" class="px-4 py-1 text-xs bg-blue-600 hover:bg-blue-500 text-white rounded">💾 保存</button>
          <button @click="editFile.show = false" class="px-4 py-1 text-xs bg-gray-700 hover:bg-gray-600 text-gray-300 rounded">关闭</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, reactive, onMounted, watch } from 'vue'
import { invoke } from '../utils/tauri.js'

const props = defineProps({ connection: Object, sessionId: String, active: Boolean })

const localPath = ref('/home')
const remotePath = ref('/')
const localFiles = ref([])
const remoteFiles = ref([])
const selectedLocal = ref(null)
const selectedRemote = ref(null)
const selectedLocalSet = reactive(new Set())
const selectedRemoteSet = reactive(new Set())
const transfers = ref([])
const remoteDragOver = ref(false)
let lastClickedLocal = null
let lastClickedRemote = null

// Drag state
let dragPaths = []

const ctxMenu = ref({ show: false, x: 0, y: 0, file: null })
const editFile = ref({ show: false, path: '', content: '', dirty: false })

function fmtSize(bytes) {
  if (!bytes) return '0 B'
  const k = 1024, s = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return (bytes / Math.pow(k, i)).toFixed(1) + ' ' + s[i]
}

function icon(name) {
  const ext = name.split('.').pop()?.toLowerCase()
  const m = { js:'📜', ts:'📘', py:'🐍', rs:'🦀', go:'🐹', md:'📝', txt:'📄', json:'📋', html:'🌐', css:'🎨', vue:'💚', png:'🖼️', jpg:'🖼️', zip:'📦', tar:'📦', gz:'📦', sh:'⚡', conf:'⚙️' }
  return m[ext] || '📄'
}

// ─── Batch Selection ───

function onLocalClick(e, file) {
  if (e.ctrlKey || e.metaKey) {
    if (selectedLocalSet.has(file.path)) selectedLocalSet.delete(file.path)
    else selectedLocalSet.add(file.path)
  } else if (e.shiftKey && lastClickedLocal) {
    const paths = localFiles.value.map(f => f.path)
    const start = paths.indexOf(lastClickedLocal)
    const end = paths.indexOf(file.path)
    if (start >= 0 && end >= 0) {
      const [lo, hi] = start < end ? [start, end] : [end, start]
      selectedLocalSet.clear()
      for (let i = lo; i <= hi; i++) selectedLocalSet.add(paths[i])
    }
  } else {
    selectedLocalSet.clear()
    selectedLocalSet.add(file.path)
  }
  selectedLocal.value = file.path
  lastClickedLocal = file.path
}

function onRemoteClick(e, file) {
  if (e.ctrlKey || e.metaKey) {
    if (selectedRemoteSet.has(file.path)) selectedRemoteSet.delete(file.path)
    else selectedRemoteSet.add(file.path)
  } else if (e.shiftKey && lastClickedRemote) {
    const paths = remoteFiles.value.map(f => f.path)
    const start = paths.indexOf(lastClickedRemote)
    const end = paths.indexOf(file.path)
    if (start >= 0 && end >= 0) {
      const [lo, hi] = start < end ? [start, end] : [end, start]
      selectedRemoteSet.clear()
      for (let i = lo; i <= hi; i++) selectedRemoteSet.add(paths[i])
    }
  } else {
    selectedRemoteSet.clear()
    selectedRemoteSet.add(file.path)
  }
  selectedRemote.value = file.path
  lastClickedRemote = file.path
}

// ─── Drag & Drop ───

function onLocalDragStart(e, file) {
  const paths = selectedLocalSet.size > 0 && selectedLocalSet.has(file.path)
    ? [...selectedLocalSet]
    : [file.path]
  dragPaths = paths
  e.dataTransfer.setData('text/plain', JSON.stringify(paths))
  e.dataTransfer.effectAllowed = 'move'
}

function onRemoteDragStart(e, file) {
  const paths = selectedRemoteSet.size > 0 && selectedRemoteSet.has(file.path)
    ? [...selectedRemoteSet]
    : [file.path]
  dragPaths = paths
  e.dataTransfer.setData('text/plain', JSON.stringify(paths))
  e.dataTransfer.effectAllowed = 'move'
}

async function onRemoteDrop(e) {
  remoteDragOver.value = false
  if (!props.sessionId) return

  // Handle local files dropped → upload
  try {
    const data = e.dataTransfer.getData('text/plain')
    const paths = JSON.parse(data)
    for (const p of paths) {
      const name = p.split('/').pop()
      const remote = remotePath.value === '/' ? `/${name}` : `${remotePath.value}/${name}`
      const t = { id: Date.now() + Math.random(), label: `↑ ${name}`, status: 'pending' }
      transfers.value.push(t)
      try {
        await invoke('sftp_upload', { sessionId: props.sessionId, local: p, remote })
        t.status = 'done'
      } catch (err) { t.status = 'error'; t.label += ` (${err})` }
    }
    loadRemote()
  } catch {}
}

async function onRemoteDirDrop(e, targetFile) {
  if (!targetFile.is_dir || !props.sessionId) return
  targetFile._dragOver = false

  try {
    const data = e.dataTransfer.getData('text/plain')
    const paths = JSON.parse(data)
    for (const p of paths) {
      const name = p.split('/').pop()
      const newPath = targetFile.path === '/' ? `/${name}` : `${targetFile.path}/${name}`
      if (p !== newPath) {
        await invoke('sftp_rename', { sessionId: props.sessionId, oldPath: p, newPath })
      }
    }
    loadRemote()
  } catch {}
}

// ─── File Operations ───

async function loadLocal() {
  try { localFiles.value = await invoke('sftp_list_local', { path: localPath.value }) }
  catch { localFiles.value = [] }
}

async function loadRemote() {
  if (!props.sessionId) return
  try { remoteFiles.value = await invoke('sftp_list_remote', { sessionId: props.sessionId, path: remotePath.value }).then(files => files.map(f => ({ ...f, _dragOver: false }))) }
  catch { remoteFiles.value = [] }
}

function refreshRemote() { loadRemote() }

function onLocalDblClick(file) {
  if (file.is_dir) { localPath.value = file.path; loadLocal() }
}

function onRemoteDblClick(file) {
  if (file.is_dir) { remotePath.value = file.path; loadRemote() }
  else { openEdit(file.path) }
}

// ─── Upload / Download ───

async function doUpload() {
  if (!props.sessionId) return
  const paths = selectedLocalSet.size > 0 ? [...selectedLocalSet] : (selectedLocal.value ? [selectedLocal.value] : [])
  for (const p of paths) {
    const name = p.split('/').pop()
    const remote = remotePath.value === '/' ? `/${name}` : `${remotePath.value}/${name}`
    const t = { id: Date.now() + Math.random(), label: `↑ ${name}`, status: 'pending' }
    transfers.value.push(t)
    try {
      await invoke('sftp_upload', { sessionId: props.sessionId, local: p, remote })
      t.status = 'done'
    } catch (e) { t.status = 'error'; t.label += ` (${e})` }
  }
  loadRemote()
}

async function doDownload() {
  if (!props.sessionId) return
  const paths = selectedRemoteSet.size > 0 ? [...selectedRemoteSet] : (selectedRemote.value ? [selectedRemote.value] : [])
  for (const p of paths) {
    const name = p.split('/').pop()
    const local = localPath.value + '/' + name
    const t = { id: Date.now() + Math.random(), label: `↓ ${name}`, status: 'pending' }
    transfers.value.push(t)
    try {
      await invoke('sftp_download', { sessionId: props.sessionId, remote: p, local })
      t.status = 'done'
    } catch (e) { t.status = 'error'; t.label += ` (${e})` }
  }
  loadLocal()
}

async function batchDelete() {
  if (!props.sessionId || selectedRemoteSet.size === 0) return
  if (!confirm(`确认删除 ${selectedRemoteSet.size} 个文件/目录？`)) return
  for (const p of [...selectedRemoteSet]) {
    const file = remoteFiles.value.find(f => f.path === p)
    if (file) {
      try { await invoke('sftp_delete', { sessionId: props.sessionId, path: p, isDir: file.is_dir }) }
      catch (e) { console.error('Delete failed:', e) }
    }
  }
  selectedRemoteSet.clear()
  loadRemote()
}

// ─── Context Menu ───

function showRemoteMenu(e, file) {
  if (!selectedRemoteSet.has(file.path)) {
    selectedRemoteSet.clear()
    selectedRemoteSet.add(file.path)
    selectedRemote.value = file.path
  }
  ctxMenu.value = { show: true, x: e.clientX, y: e.clientY, file }
  const close = () => { ctxMenu.value.show = false; document.removeEventListener('click', close) }
  setTimeout(() => document.addEventListener('click', close), 10)
}

async function ctxRename() {
  const file = ctxMenu.value.file
  if (!file) return
  const newName = prompt('新名称:', file.name)
  if (!newName || newName === file.name) return
  const dir = file.path.substring(0, file.path.lastIndexOf('/'))
  const newPath = dir ? `${dir}/${newName}` : `/${newName}`
  try { await invoke('sftp_rename', { sessionId: props.sessionId, oldPath: file.path, newPath }); loadRemote() }
  catch (e) { alert('重命名失败: ' + e) }
}

async function ctxDelete() {
  const file = ctxMenu.value.file
  if (!file) return
  if (!confirm(`确认删除 ${file.name}?`)) return
  try { await invoke('sftp_delete', { sessionId: props.sessionId, path: file.path, isDir: file.is_dir }); loadRemote() }
  catch (e) { alert('删除失败: ' + e) }
}

async function ctxChmod() {
  const file = ctxMenu.value.file
  if (!file) return
  const mode = prompt('权限 (如 755):', '644')
  if (!mode) return
  try { await invoke('sftp_chmod', { sessionId: props.sessionId, path: file.path, mode }); loadRemote() }
  catch (e) { alert('修改权限失败: ' + e) }
}

function ctxEdit() { if (ctxMenu.value.file) openEdit(ctxMenu.value.file.path) }

function ctxDownload() {
  if (ctxMenu.value.file) {
    selectedRemoteSet.clear()
    selectedRemoteSet.add(ctxMenu.value.file.path)
    doDownload()
  }
}

function ctxCopyPath() {
  if (ctxMenu.value.file) {
    navigator.clipboard.writeText(ctxMenu.value.file.path).catch(() => {})
  }
}

async function openEdit(path) {
  if (!props.sessionId) return
  try {
    const content = await invoke('sftp_read_file', { sessionId: props.sessionId, path })
    editFile.value = { show: true, path, content, dirty: false }
  } catch (e) { alert('读取文件失败: ' + e) }
}

async function saveEdit() {
  try {
    await invoke('sftp_write_file', { sessionId: props.sessionId, path: editFile.value.path, content: editFile.value.content })
    editFile.value.dirty = false
  } catch (e) { alert('保存失败: ' + e) }
}

async function createRemoteDir() {
  if (!props.sessionId) return
  const name = prompt('目录名称:')
  if (!name) return
  const path = remotePath.value === '/' ? `/${name}` : `${remotePath.value}/${name}`
  try { await invoke('sftp_mkdir', { sessionId: props.sessionId, path }); loadRemote() }
  catch (e) { alert('创建失败: ' + e) }
}

onMounted(() => loadLocal())
watch(() => props.sessionId, (sid) => { if (sid) loadRemote() })
</script>
