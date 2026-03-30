<template>
  <div class="flex flex-col h-full bg-gray-900">
    <!-- Toolbar -->
    <div class="h-9 bg-gray-800 border-b border-gray-700 flex items-center px-2 gap-2">
      <button @click="createRemoteDir" :disabled="!sessionId" class="px-2 py-0.5 text-xs rounded bg-gray-700 text-gray-300 hover:bg-gray-600 disabled:opacity-50" title="新建文件夹">📁+</button>
      <button @click="refreshRemote" :disabled="!sessionId" class="px-2 py-0.5 text-xs rounded bg-gray-700 text-gray-300 hover:bg-gray-600 disabled:opacity-50">⟳ 刷新</button>
      <div class="flex-1" />
      <span class="text-xs text-gray-500">{{ connection?.name || '未连接' }}</span>
    </div>

    <!-- SFTP Panels -->
    <div class="flex-1 flex">
      <!-- Local Panel -->
      <div class="flex flex-col border-r border-gray-700" :style="{ width: '50%' }">
        <div class="h-8 bg-gray-800 border-b border-gray-700 flex items-center px-2 gap-1">
          <span class="text-xs text-green-400">🏠 本地</span>
          <input v-model="localPath" @keydown.enter="loadLocal" class="flex-1 bg-gray-900 text-xs text-gray-300 px-2 py-0.5 rounded border border-gray-600 focus:outline-none focus:border-blue-500" />
          <button @click="loadLocal" class="text-xs text-gray-400 hover:text-white">⟳</button>
        </div>
        <div class="flex-1 overflow-y-auto">
          <div v-for="file in localFiles" :key="file.path" @click="selectedLocal = file.path" @dblclick="onLocalDblClick(file)"
            class="flex items-center gap-2 px-3 py-1 cursor-pointer hover:bg-gray-700 text-sm"
            :class="selectedLocal === file.path ? 'bg-blue-600/20' : ''">
            <span class="w-5 text-center">{{ file.is_dir ? '📁' : icon(file.name) }}</span>
            <span class="flex-1 truncate text-gray-300">{{ file.name }}</span>
            <span class="text-xs text-gray-500 w-16 text-right">{{ file.is_dir ? '' : fmtSize(file.size) }}</span>
          </div>
        </div>
      </div>

      <!-- Transfer Buttons -->
      <div class="w-10 bg-gray-800 flex flex-col items-center justify-center gap-2 border-r border-gray-700">
        <button @click="doUpload" :disabled="!selectedLocal || !sessionId" class="w-8 h-8 rounded text-lg disabled:opacity-30 bg-blue-600 hover:bg-blue-500 text-white">→</button>
        <button @click="doDownload" :disabled="!selectedRemote || !sessionId" class="w-8 h-8 rounded text-lg disabled:opacity-30 bg-blue-600 hover:bg-blue-500 text-white">←</button>
      </div>

      <!-- Remote Panel -->
      <div class="flex-1 flex flex-col">
        <div class="h-8 bg-gray-800 border-b border-gray-700 flex items-center px-2 gap-1">
          <span class="text-xs text-blue-400">🌐 远程</span>
          <input v-model="remotePath" @keydown.enter="loadRemote" class="flex-1 bg-gray-900 text-xs text-gray-300 px-2 py-0.5 rounded border border-gray-600 focus:outline-none focus:border-blue-500" />
          <button @click="loadRemote" class="text-xs text-gray-400 hover:text-white">⟳</button>
        </div>
        <div class="flex-1 overflow-y-auto">
          <div v-for="file in remoteFiles" :key="file.path"
            @click="selectedRemote = file.path"
            @dblclick="onRemoteDblClick(file)"
            @contextmenu.prevent="showRemoteMenu($event, file)"
            class="flex items-center gap-2 px-3 py-1 cursor-pointer hover:bg-gray-700 text-sm"
            :class="selectedRemote === file.path ? 'bg-blue-600/20' : ''">
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
      <div class="px-2 py-1 text-xs text-gray-500 border-b border-gray-700">传输队列</div>
      <div v-for="t in transfers" :key="t.id" class="px-2 py-1 flex items-center gap-2 text-xs">
        <span :class="t.status === 'done' ? 'text-green-400' : t.status === 'error' ? 'text-red-400' : 'text-yellow-400'">
          {{ t.status === 'done' ? '✓' : t.status === 'error' ? '✗' : '⏳' }}
        </span>
        <span class="flex-1 truncate text-gray-300">{{ t.label }}</span>
        <button v-if="t.status !== 'pending'" @click="transfers = transfers.filter(x => x.id !== t.id)" class="text-gray-500 hover:text-white">✕</button>
      </div>
    </div>

    <!-- Context Menu -->
    <div v-if="ctxMenu.show" :style="{ left: ctxMenu.x + 'px', top: ctxMenu.y + 'px' }" class="fixed bg-gray-800 border border-gray-600 rounded shadow-lg z-50 py-1 text-xs" @click.stop>
      <div @click="ctxRename" class="px-4 py-1.5 hover:bg-gray-600 cursor-pointer text-gray-300">重命名</div>
      <div @click="ctxDelete" class="px-4 py-1.5 hover:bg-gray-600 cursor-pointer text-red-400">删除</div>
      <div @click="ctxChmod" class="px-4 py-1.5 hover:bg-gray-600 cursor-pointer text-gray-300">修改权限</div>
      <div class="border-t border-gray-700 my-1" />
      <div @click="ctxEdit" class="px-4 py-1.5 hover:bg-gray-600 cursor-pointer text-gray-300">在线编辑</div>
    </div>

    <!-- Edit Modal -->
    <div v-if="editFile.show" class="fixed inset-0 bg-black/60 flex items-center justify-center z-50" @click.self="editFile.show = false">
      <div class="bg-gray-800 rounded-lg shadow-xl w-[700px] max-h-[80vh] flex flex-col">
        <div class="flex items-center justify-between px-4 py-2 border-b border-gray-700">
          <span class="text-sm text-gray-300">编辑: {{ editFile.path }}</span>
          <button @click="editFile.show = false" class="text-gray-400 hover:text-white">✕</button>
        </div>
        <textarea v-model="editFile.content" class="flex-1 bg-gray-900 text-gray-200 text-sm p-4 font-mono resize-none focus:outline-none min-h-[300px]" spellcheck="false" />
        <div class="flex justify-end gap-2 px-4 py-2 border-t border-gray-700">
          <button @click="saveEdit" class="px-4 py-1 text-xs bg-blue-600 hover:bg-blue-500 text-white rounded">保存</button>
          <button @click="editFile.show = false" class="px-4 py-1 text-xs bg-gray-700 hover:bg-gray-600 text-gray-300 rounded">取消</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, watch } from 'vue'
import { invoke } from '../utils/tauri.js'

const props = defineProps({ connection: Object, sessionId: String, active: Boolean })

const localPath = ref('/home')
const remotePath = ref('/')
const localFiles = ref([])
const remoteFiles = ref([])
const selectedLocal = ref(null)
const selectedRemote = ref(null)
const transfers = ref([])

// Context menu
const ctxMenu = ref({ show: false, x: 0, y: 0, file: null })
const editFile = ref({ show: false, path: '', content: '' })

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

async function loadLocal() {
  try { localFiles.value = await invoke('sftp_list_local', { path: localPath.value }) }
  catch { localFiles.value = [] }
}

async function loadRemote() {
  if (!props.sessionId) return
  try { remoteFiles.value = await invoke('sftp_list_remote', { sessionId: props.sessionId, path: remotePath.value }) }
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
  if (!selectedLocal.value || !props.sessionId) return
  const name = selectedLocal.value.split('/').pop()
  const remote = remotePath.value === '/' ? `/${name}` : `${remotePath.value}/${name}`
  const t = { id: Date.now(), label: `↑ ${name}`, status: 'pending' }
  transfers.value.push(t)
  try {
    await invoke('sftp_upload', { sessionId: props.sessionId, local: selectedLocal.value, remote })
    t.status = 'done'
    loadRemote()
  } catch (e) { t.status = 'error'; t.label += ` (${e})` }
}

async function doDownload() {
  if (!selectedRemote.value || !props.sessionId) return
  const name = selectedRemote.value.split('/').pop()
  const local = localPath.value + '/' + name
  const t = { id: Date.now(), label: `↓ ${name}`, status: 'pending' }
  transfers.value.push(t)
  try {
    await invoke('sftp_download', { sessionId: props.sessionId, remote: selectedRemote.value, local })
    t.status = 'done'
    loadLocal()
  } catch (e) { t.status = 'error'; t.label += ` (${e})` }
}

// ─── Context Menu ───

function showRemoteMenu(e, file) {
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
  try {
    await invoke('sftp_rename', { sessionId: props.sessionId, oldPath: file.path, newPath })
    loadRemote()
  } catch (e) { alert('重命名失败: ' + e) }
}

async function ctxDelete() {
  const file = ctxMenu.value.file
  if (!file) return
  if (!confirm(`确认删除 ${file.name}?`)) return
  try {
    await invoke('sftp_delete', { sessionId: props.sessionId, path: file.path, isDir: file.is_dir })
    loadRemote()
  } catch (e) { alert('删除失败: ' + e) }
}

async function ctxChmod() {
  const file = ctxMenu.value.file
  if (!file) return
  const mode = prompt('权限 (如 755):', '644')
  if (!mode) return
  try {
    await invoke('sftp_chmod', { sessionId: props.sessionId, path: file.path, mode })
    loadRemote()
  } catch (e) { alert('修改权限失败: ' + e) }
}

async function ctxEdit() {
  const file = ctxMenu.value.file
  if (file) openEdit(file.path)
}

async function openEdit(path) {
  if (!props.sessionId) return
  try {
    const content = await invoke('sftp_read_file', { sessionId: props.sessionId, path })
    editFile.value = { show: true, path, content }
  } catch (e) { alert('读取文件失败: ' + e) }
}

async function saveEdit() {
  try {
    await invoke('sftp_write_file', { sessionId: props.sessionId, path: editFile.value.path, content: editFile.value.content })
    editFile.value.show = false
    alert('已保存')
  } catch (e) { alert('保存失败: ' + e) }
}

async function createRemoteDir() {
  if (!props.sessionId) return
  const name = prompt('目录名称:')
  if (!name) return
  const path = remotePath.value === '/' ? `/${name}` : `${remotePath.value}/${name}`
  try {
    await invoke('sftp_mkdir', { sessionId: props.sessionId, path })
    loadRemote()
  } catch (e) { alert('创建失败: ' + e) }
}

onMounted(() => loadLocal())
watch(() => props.sessionId, (sid) => { if (sid) loadRemote() })
</script>
