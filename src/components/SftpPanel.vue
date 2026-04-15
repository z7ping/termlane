<template>
  <div class="flex flex-col h-full" style="background: var(--bg-base);">
    <!-- Toolbar -->
    <div class="h-9 flex items-center px-2 gap-2" style="background: var(--bg-surface); border-bottom: 1px solid var(--border-subtle);">
      <button @click="createRemoteDir" :disabled="!sessionId" class="px-2 py-0.5 text-xs rounded disabled:opacity-50 flex items-center gap-1" style="background: var(--bg-elevated); color:; hover:background: var(--bg-hover);"><FolderPlus :size="14" /></button>
      <button @click="batchDelete" :disabled="!sessionId || selectedRemoteSet.size === 0" class="px-2 py-0.5 text-xs rounded disabled:opacity-50 flex items-center gap-1" style="background: var(--bg-elevated); color:; hover:background: rgba(220, 38, 38, 0.2);"><Trash2 :size="14" /> {{ selectedRemoteSet.size || '' }}</button>
      <button @click="refreshRemote" :disabled="!sessionId" class="px-2 py-0.5 text-xs rounded disabled:opacity-50 flex items-center gap-1" style="background: var(--bg-elevated); color:; hover:background: var(--bg-hover);"><RefreshCw :size="14" /></button>
      <div class="flex-1" />
      <span class="text-xs" style="color: var(--fg-muted);">{{ selectedRemoteSet.size ? `已选 ${selectedRemoteSet.size} 个` : '' }}</span>
      <span class="text-xs" style="color: var(--fg-muted);">{{ connection?.name || '未连接' }}</span>
    </div>

    <!-- SFTP Panels -->
    <div class="flex-1 flex">
      <!-- Local Panel -->
      <div class="flex flex-col overflow-hidden" :style="{ width: localWidth + '%' }">
        <div class="h-8 flex items-center px-2 gap-1" style="background: var(--bg-surface); border-bottom: 1px solid var(--border-subtle);">
          <span class="text-xs" style="color: var(--success);">🏠 本地</span>
          <input v-model="localPath" @keydown.enter="loadLocal" class="flex-1 text-xs px-2 py-0.5 rounded border focus:outline-none" style="background: var(--bg-base); color: var(--fg-secondary); border-color: var(--border);" />
          <button @click="loadLocal" class="text-xs" style="color: var(--fg-muted);">⟳</button>
        </div>
        <div class="flex-1 overflow-y-auto" @click.self="selectedLocalSet.clear()">
          <!-- Virtual list for large directories -->
          <VirtualList v-if="localVirtual" :items="localFiles" :item-height="28" :height="400">
            <template #default="{ item: file }">
              <div @click="onLocalClick($event, file)" @dblclick="onLocalDblClick(file)" @dragEvent="onLocalDragStart($event, file)" draggable="true"
                class="flex items-center gap-2 px-3 cursor-pointer hover:bg-white/5 text-xs select-none h-full"
                :style="selectedLocalSet.has(file.path) ? 'background: var(--accent-hover); color: var(--accent);' : ''">
                <span class="w-4 text-center text-xs">{{ file.is_dir ? '📁' : icon(file.name) }}</span>
                <span class="flex-1 truncate" style="color: var(--fg-secondary);">{{ file.name }}</span>
                <span class="text-[10px] w-14 text-right" style="color: var(--fg-muted);">{{ file.is_dir ? '' : fmtSize(file.size) }}</span>
              </div>
            </template>
          </VirtualList>
          <!-- Regular list for small directories -->
          <template v-else>
          <div v-for="file in localFiles" :key="file.path"
            @click="onLocalClick($event, file)"
            @dblclick="onLocalDblClick(file)"
            @dragstart="onLocalDragStart($event, file)"
            draggable="true"
            class="flex items-center gap-2 px-3 py-1 cursor-pointer text-sm select-none hover:bg-white/5"
            :style="selectedLocalSet.has(file.path) ? 'background: var(--accent-hover); color: var(--accent);' : ''">
            <span class="w-5 text-center">{{ file.is_dir ? '📁' : icon(file.name) }}</span>
            <span class="flex-1 truncate" style="color: var(--fg-secondary);">{{ file.name }}</span>
            <span class="text-xs w-16 text-right" style="color: var(--fg-muted);">{{ file.is_dir ? '' : fmtSize(file.size) }}</span>
          </div>
          </template>
        </div>
      </div>

      <!-- Resize Divider -->
      <div class="w-1 cursor-col-resize hover:bg-blue-500/50 transition-colors flex-shrink-0" @mousedown="startLocalResize" />

      <!-- Transfer Buttons -->
      <div class="w-10 flex flex-col items-center justify-center gap-2" style="background: var(--bg-surface); border-right: 1px solid var(--border-subtle);">
        <button @click="doUpload" :disabled="selectedLocalSet.size === 0 && !selectedLocal || !sessionId"
          class="w-8 h-8 rounded text-lg disabled:opacity-30 text-white"
          style="background: var(--accent); hover:background: var(--accent-hover);"
          :title="selectedLocalSet.size > 1 ? `上传 ${selectedLocalSet.size} 个文件` : '上传'">→</button>
        <button @click="doDownload" :disabled="selectedRemoteSet.size === 0 && !selectedRemote || !sessionId"
          class="w-8 h-8 rounded text-lg disabled:opacity-30 text-white"
          style="background: var(--accent); hover:background: var(--accent-hover);"
          :title="selectedRemoteSet.size > 1 ? `下载 ${selectedRemoteSet.size} 个文件` : '下载'">←</button>
      </div>

      <!-- Remote Panel -->
      <div class="flex-1 flex flex-col"
        @dragover.prevent="remoteDragOver = true"
        @dragleave="remoteDragOver = false"
        @drop.prevent="onRemoteDrop"
        :class="remoteDragOver ? 'ring-2' : ''"
        :style="remoteDragOver ? '--tw-ring-color: var(--accent-hover);' : ''">
        <div class="h-8 flex items-center px-2 gap-1" style="background: var(--bg-surface); border-bottom: 1px solid var(--border-subtle);">
          <span class="text-xs" style="color: var(--accent);">🌐 远程</span>
          <input v-model="remotePath" @keydown.enter="loadRemote" class="flex-1 text-xs px-2 py-0.5 rounded border focus:outline-none" style="background: var(--bg-base); color: var(--fg-secondary); border-color: var(--border);" />
          <button @click="loadRemote" class="text-xs" style="color: var(--fg-muted);">⟳</button>
        </div>

        <!-- Inline Editor (replaces file list when editing) -->
        <Transition name="slide-right">
        <div v-if="showInlineEditor" class="flex-1 flex flex-col" style="background: var(--bg-base);">
          <div class="flex items-center justify-between px-4 py-2" style="border-bottom: 1px solid var(--border-subtle);">
            <div class="flex items-center gap-2">
              <span class="text-sm font-medium" style="color: var(--fg-primary);">📝 {{ editFile.path.split('/').pop() }}</span>
              <span v-if="editFile.dirty" class="text-[10px] px-1.5 py-0.5 rounded-full animate-pulse-dot" style="background: var(--warning); color: #000;">未保存</span>
            </div>
            <div class="flex items-center gap-1">
              <button @click="saveEdit" class="px-3 py-1 text-xs rounded-lg font-medium" style="background: var(--accent); color: white;">保存</button>
              <button @click="closeEditor" class="p-1 rounded hover:bg-white/10" style="color: var(--fg-muted);">✕</button>
            </div>
          </div>
          <textarea v-model="editFile.content" @input="editFile.dirty = true" class="flex-1 text-sm p-4 font-mono resize-none focus:outline-none" style="background: var(--bg-base); color: var(--fg-primary);" spellcheck="false" />
        </div>
        </Transition>

        <!-- File List -->
        <div v-if="!showInlineEditor" class="flex-1 overflow-y-auto" @click.self="selectedRemoteSet.clear()">
          <!-- Virtual list for large directories -->
          <VirtualList v-if="remoteVirtual" :items="remoteFiles" :item-height="28" :height="400">
            <template #default="{ item: file }">
              <div @click="onRemoteClick($event, file)" @dblclick="onRemoteDblClick(file)" @contextmenu.prevent="showRemoteMenu($event, file)"
                @dragstart="onRemoteDragStart($event, file)" :draggable="true"
                class="flex items-center gap-2 px-3 cursor-pointer hover:bg-white/5 text-xs select-none h-full"
                :style="selectedRemoteSet.has(file.path) ? 'background: var(--accent-hover); color: var(--accent);' : ''">
                <span class="w-4 text-center text-xs">{{ file.is_dir ? '📁' : icon(file.name) }}</span>
                <span class="flex-1 truncate" style="color: var(--fg-secondary);">{{ file.name }}</span>
                <span class="text-[10px] w-14 text-right" style="color: var(--fg-muted);">{{ file.is_dir ? '' : fmtSize(file.size) }}</span>
                <span class="text-[10px] w-16 text-right" style="color: var(--fg-muted);">{{ file.permissions || '' }}</span>
              </div>
            </template>
          </VirtualList>
          <!-- Regular list for small directories -->
          <template v-else>
          <div v-for="file in remoteFiles" :key="file.path"
            @click="onRemoteClick($event, file)"
            @dblclick="onRemoteDblClick(file)"
            @contextmenu.prevent="showRemoteMenu($event, file)"
            @dragstart="onRemoteDragStart($event, file)"
            @dragover.prevent="file._dragOver = true"
            @dragleave="file._dragOver = false"
            @drop.prevent.stop="onRemoteDirDrop($event, file)"
            :draggable="true"
            class="flex items-center gap-2 px-3 py-1 cursor-pointer text-sm select-none hover:bg-white/5"
            :style="[
              selectedRemoteSet.has(file.path) ? 'background: var(--accent-hover); color: var(--accent);' : '',
              file._dragOver && file.is_dir ? 'background: var(--accent-hover); --tw-ring-color: var(--accent);' : ''
            ]">
            <span class="w-5 text-center">{{ file.is_dir ? '📁' : icon(file.name) }}</span>
            <span class="flex-1 truncate" style="color: var(--fg-secondary);">{{ file.name }}</span>
            <span class="text-xs w-16 text-right" style="color: var(--fg-muted);">{{ file.is_dir ? '' : fmtSize(file.size) }}</span>
            <span class="text-xs w-20 text-right" style="color: var(--fg-muted);">{{ file.permissions || '' }}</span>
          </div>
          <div v-if="remoteFiles.length === 0" class="p-4 text-center text-sm" style="color: var(--fg-muted);">{{ sessionId ? '空目录' : '未连接' }}</div>
          </template>
        </div>
      </div>
    </div>

    <!-- Transfer Queue -->
    <div v-if="transfers.length > 0" class="h-20 overflow-y-auto" style="background: var(--bg-base); border-top: 1px solid var(--border-subtle);">
      <div class="px-2 py-1 text-xs flex items-center justify-between" style="color: var(--fg-muted); border-bottom: 1px solid var(--border-subtle);">
        <span>传输队列 ({{ transfers.filter(t => t.status === 'pending').length }} 进行中)</span>
        <button @click="transfers = transfers.filter(t => t.status === 'pending')" style="color: var(--fg-muted);">清除已完成</button>
      </div>
      <div v-for="t in transfers" :key="t.id" class="px-2 py-1 flex items-center gap-2 text-xs">
        <span :style="t.status === 'done' ? 'color: var(--success);' : t.status === 'error' ? 'color: var(--danger);' : 'color: var(--warning);'">
          {{ t.status === 'done' ? '✓' : t.status === 'error' ? '✗' : '⏳' }}
        </span>
        <span class="flex-1 truncate" style="color: var(--fg-secondary);">{{ t.label }}</span>
        <span v-if="t.status === 'pending'" style="color: var(--fg-muted);">{{ t.progress || '' }}</span>
        <button v-if="t.status !== 'pending'" @click="transfers = transfers.filter(x => x.id !== t.id)" style="color: var(--fg-muted);">✕</button>
      </div>
    </div>

    <!-- Context Menu -->
    <div v-if="ctxMenu.show" :style="{ left: ctxMenu.x + 'px', top: ctxMenu.y + 'px' }" class="fixed rounded shadow-lg z-50 py-1 text-xs min-w-[140px]" style="background: var(--bg-surface); border: 1px solid var(--border);" @click.stop>
      <div @click="ctxEdit" class="px-4 py-1.5 cursor-pointer" style="color: var(--fg-secondary); hover:background: var(--bg-hover);">📝 在线编辑</div>
      <div @click="ctxRename" class="px-4 py-1.5 cursor-pointer" style="color: var(--fg-secondary); hover:background: var(--bg-hover);">✏️ 重命名</div>
      <div @click="ctxDownload" class="px-4 py-1.5 cursor-pointer" style="color: var(--fg-secondary); hover:background: var(--bg-hover);">📥 下载</div>
      <div class="my-1" style="border-top: 1px solid var(--border);" />
      <div @click="ctxChmod" class="px-4 py-1.5 cursor-pointer" style="color: var(--fg-secondary); hover:background: var(--bg-hover);">🔒 修改权限</div>
      <div @click="ctxCopyPath" class="px-4 py-1.5 cursor-pointer" style="color: var(--fg-secondary); hover:background: var(--bg-hover);">📋 复制路径</div>
      <div class="my-1" style="border-top: 1px solid var(--border);" />
      <div @click="ctxDelete" class="px-4 py-1.5 cursor-pointer" style="color: var(--danger); hover:background: var(--bg-hover);">🗑 删除</div>
    </div>

  </div>
</template>

<script setup>
import { ref, reactive, onMounted, watch, computed } from 'vue'
import { invoke } from '../utils/tauri.js'
import VirtualList from './VirtualList.vue'
import { FolderPlus, Trash2, RefreshCw } from 'lucide-vue-next'

const props = defineProps({ connection: Object, sessionId: String, active: Boolean })

const localPath = ref('/home')
const remotePath = ref('/')
const localFiles = ref([])
const remoteFiles = ref([])
const selectedLocal = ref(null)
const selectedRemote = ref(null)
const selectedLocalSet = reactive(new Set())
const selectedRemoteSet = reactive(new Set())
const localVirtual = computed(() => localFiles.value.length > 100)
const remoteVirtual = computed(() => remoteFiles.value.length > 100)
const remoteDragOver = ref(false)
const localWidth = ref(50) // percentage for resizable split
let lastClickedLocal = null
let lastClickedRemote = null

// Drag state
let dragPaths = []

// --- SFTP split resize ---
function startLocalResize(e) {
  e.preventDefault()
  const container = e.target.parentElement
  const rect = container.getBoundingClientRect()
  const startX = e.clientX
  const startW = localWidth.value
  document.body.style.cursor = 'col-resize'
  document.body.style.userSelect = 'none'
  const onMove = (ev) => {
    const pct = startW + ((ev.clientX - startX) / rect.width) * 100
    localWidth.value = Math.max(20, Math.min(80, pct))
  }
  const onUp = () => {
    document.removeEventListener('mousemove', onMove)
    document.removeEventListener('mouseup', onUp)
    document.body.style.cursor = ''
    document.body.style.userSelect = ''
  }
  document.addEventListener('mousemove', onMove)
  document.addEventListener('mouseup', onUp)
}

const ctxMenu = ref({ show: false, x: 0, y: 0, file: null })
const editFile = ref({ show: false, path: '', content: '', dirty: false })
const showInlineEditor = ref(false)

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
      catch { /* ignore */ }
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
    showInlineEditor.value = true
  } catch (e) { alert('读取文件失败: ' + e) }
}

function closeEditor() {
  showInlineEditor.value = false
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

<style scoped>
.slide-right-enter-active,
.slide-right-leave-active {
  transition: opacity 0.2s ease, transform 0.2s ease;
}
.slide-right-enter-from {
  opacity: 0;
  transform: translateX(20px);
}
.slide-right-leave-to {
  opacity: 0;
  transform: translateX(20px);
}
</style>
