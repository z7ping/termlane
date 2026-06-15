<template>
  <div class="flex flex-col h-full" style="background: var(--bg-base);">
    <!-- Toolbar -->
    <div class="h-9 flex items-center px-2 gap-2" style="background: var(--bg-surface); border-bottom: 1px solid var(--border-subtle);">
      <button @click="createRemoteDir" :disabled="!sessionId" class="px-2 py-0.5 text-xs rounded disabled:opacity-50 flex items-center gap-1" style="background: var(--bg-elevated); color: var(--fg-secondary);"><FolderPlus :size="14" /></button>
      <button @click="batchDelete" :disabled="!sessionId || selectedRemoteSet.size === 0" class="px-2 py-0.5 text-xs rounded disabled:opacity-50 flex items-center gap-1" style="background: var(--bg-elevated); color: var(--fg-secondary);"><Trash2 :size="14" /> {{ selectedRemoteSet.size || '' }}</button>
      <button @click="refreshRemote" :disabled="!sessionId" class="px-2 py-0.5 text-xs rounded disabled:opacity-50 flex items-center gap-1" style="background: var(--bg-elevated); color: var(--fg-secondary);"><RefreshCw :size="14" /></button>
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
              <div @click="onLocalClick($event, file)" @dblclick="onLocalDblClick(file)" @dragstart="onLocalDragStart($event, file)" draggable="true"
                class="flex items-center gap-2 px-3 cursor-pointer hover:bg-white/5 text-xs select-none h-full"
                :style="selectedLocalSet.has(file.path) ? 'background: var(--accent-hover); color: var(--accent);' : ''">
                <span class="w-4 text-center text-xs">{{ file.isDir ? '📁' : icon(file.name) }}</span>
                <span class="flex-1 truncate" style="color: var(--fg-secondary);">{{ file.name }}</span>
                <span class="text-xs w-14 text-right" style="color: var(--fg-muted);">{{ file.isDir ? '' : fmtSize(file.size) }}</span>
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
            <span class="w-5 text-center">{{ file.isDir ? '📁' : icon(file.name) }}</span>
            <span class="flex-1 truncate" style="color: var(--fg-secondary);">{{ file.name }}</span>
            <span class="text-xs w-16 text-right" style="color: var(--fg-muted);">{{ file.isDir ? '' : fmtSize(file.size) }}</span>
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
              <span v-if="editFile.dirty" class="text-xs px-1.5 py-0.5 rounded-full animate-pulse-dot" style="background: var(--warning); color: #000;">未保存</span>
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
                <span class="w-4 text-center text-xs">{{ file.isDir ? '📁' : icon(file.name) }}</span>
                <span class="flex-1 truncate" style="color: var(--fg-secondary);">{{ file.name }}</span>
                <span class="text-xs w-14 text-right" style="color: var(--fg-muted);">{{ file.isDir ? '' : fmtSize(file.size) }}</span>
                <span class="text-xs w-16 text-right" style="color: var(--fg-muted);">{{ file.permissions || '' }}</span>
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
              file._dragOver && file.isDir ? 'background: var(--accent-hover); --tw-ring-color: var(--accent);' : ''
            ]">
            <span class="w-5 text-center">{{ file.isDir ? '📁' : icon(file.name) }}</span>
            <span class="flex-1 truncate" style="color: var(--fg-secondary);">{{ file.name }}</span>
            <span class="text-xs w-16 text-right" style="color: var(--fg-muted);">{{ file.isDir ? '' : fmtSize(file.size) }}</span>
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

    <!-- Loading Overlay -->
    <div v-if="loading" class="absolute inset-0 flex items-center justify-center z-10" style="background: color-mix(in srgb, var(--bg-base) 70%, transparent);">
      <div class="flex items-center gap-2 text-sm" style="color: var(--fg-muted);">
        <span class="animate-spin">⏳</span> 加载中...
      </div>
    </div>

    <!-- Inline Prompt Dialog -->
    <Teleport to="body">
      <div v-if="promptState.show" class="fixed inset-0 flex items-center justify-center z-[60]" style="background: color-mix(in srgb, #000 60%, transparent)" @click.self="promptResolve(null)">
        <div class="rounded-lg p-4 w-80 space-y-3" style="background: var(--bg-surface); border: 1px solid var(--border-subtle);" @keydown.esc="promptResolve(null)">
          <div class="text-sm font-medium" style="color: var(--fg-primary);">{{ promptState.title }}</div>
          <input ref="promptInputRef" v-model="promptState.value" @keydown.enter="promptResolve(promptState.value)" class="w-full rounded px-3 py-1.5 text-sm focus:outline-none" style="background: var(--bg-base); color: var(--fg-primary); border: 1px solid var(--border);" />
          <div class="flex justify-end gap-2">
            <button @click="promptResolve(null)" class="px-3 py-1 text-xs rounded" style="background: var(--bg-elevated); color: var(--fg-muted);">取消</button>
            <button @click="promptResolve(promptState.value)" class="px-3 py-1 text-xs rounded text-white" style="background: var(--accent);">确定</button>
          </div>
        </div>
      </div>
    </Teleport>

    <!-- Inline Confirm Dialog -->
    <Teleport to="body">
      <div v-if="confirmState.show" class="fixed inset-0 flex items-center justify-center z-[60]" style="background: color-mix(in srgb, #000 60%, transparent)" @click.self="confirmResolve(false)">
        <div class="rounded-lg p-4 w-80 space-y-3" style="background: var(--bg-surface); border: 1px solid var(--border-subtle);" @keydown.esc="confirmResolve(false)">
          <div class="text-sm" style="color: var(--fg-primary);">{{ confirmState.message }}</div>
          <div class="flex justify-end gap-2">
            <button @click="confirmResolve(false)" class="px-3 py-1 text-xs rounded" style="background: var(--bg-elevated); color: var(--fg-muted);">取消</button>
            <button @click="confirmResolve(true)" class="px-3 py-1 text-xs rounded text-white" style="background: var(--accent);">确定</button>
          </div>
        </div>
      </div>
    </Teleport>

    <!-- Toast -->
    <div v-if="toastState.show" class="absolute top-2 right-2 px-3 py-1.5 rounded text-xs z-20 transition-opacity" :class="{
      'bg-green-600/90 text-white': toastState.type === 'success',
      'bg-red-600/90 text-white': toastState.type === 'error',
      'bg-gray-700/90 text-gray-200': toastState.type === 'info',
    }">{{ toastState.message }}</div>

  </div>
</template>

<script setup>
import { ref, reactive, onMounted, watch, computed, nextTick } from 'vue'
import { invoke } from '../utils/tauri.js'
import VirtualList from './VirtualList.vue'
import { FolderPlus, Trash2, RefreshCw } from 'lucide-vue-next'
import { formatBytes } from '../utils/format.js'
import { useSplitResize } from '../composables/useSplitResize.js'
import { useFileSelection } from '../composables/useFileSelection.js'

const props = defineProps({ connection: Object, sessionId: String, active: Boolean })

const { startResize } = useSplitResize()

// ─── Toast ───
const toastState = ref({ show: false, message: '', type: 'info' })
function _toast(message, type = 'info', duration = 2500) {
  toastState.value = { show: true, message, type }
  setTimeout(() => { toastState.value.show = false }, duration)
}

// ─── Prompt Dialog ───
const promptInputRef = ref(null)
const promptState = ref({ show: false, title: '', value: '' })
let promptResolveFn = null
function showPrompt(title, defaultVal = '') {
  return new Promise((resolve) => {
    promptState.value = { show: true, title, value: defaultVal }
    promptResolveFn = resolve
    nextTick(() => promptInputRef.value?.focus())
  })
}
function promptResolve(val) {
  promptState.value.show = false
  promptResolveFn?.(val)
  promptResolveFn = null
}

// ─── Confirm Dialog ───
const confirmState = ref({ show: false, message: '' })
let confirmResolveFn = null
function showConfirm(message) {
  return new Promise((resolve) => {
    confirmState.value = { show: true, message }
    confirmResolveFn = resolve
  })
}
function confirmResolve(val) {
  confirmState.value.show = false
  confirmResolveFn?.(val)
  confirmResolveFn = null
}

// ─── Loading ───
const loading = ref(false)

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

const { onClick: onLocalClick, onDragStart: onLocalDragStart } = useFileSelection(selectedLocalSet, localFiles, { onSelect: (p) => { selectedLocal.value = p } })
const { onClick: onRemoteClick, onDragStart: onRemoteDragStart } = useFileSelection(selectedRemoteSet, remoteFiles, { onSelect: (p) => { selectedRemote.value = p } })

// Drag state
let dragPaths = []

// --- SFTP split resize ---
function startLocalResize(e) {
  const container = e.target.parentElement
  startResize(e, container, localWidth)
}

const ctxMenu = ref({ show: false, x: 0, y: 0, file: null })
const editFile = ref({ show: false, path: '', content: '', dirty: false })
const showInlineEditor = ref(false)

function fmtSize(bytes) {
  return formatBytes(bytes)
}

function icon(name) {
  const ext = name.split('.').pop()?.toLowerCase()
  const m = { js:'📜', ts:'📘', py:'🐍', rs:'🦀', go:'🐹', md:'📝', txt:'📄', json:'📋', html:'🌐', css:'🎨', vue:'💚', png:'🖼️', jpg:'🖼️', zip:'📦', tar:'📦', gz:'📦', sh:'⚡', conf:'⚙️' }
  return m[ext] || '📄'
}

// ─── Drag & Drop ───

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
      const t = { id: crypto.randomUUID(), label: `↑ ${name}`, status: 'pending' }
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
  if (!targetFile.isDir || !props.sessionId) return
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
  loading.value = true
  try { localFiles.value = await invoke('sftp_list_local', { path: localPath.value }) }
  catch { localFiles.value = [] }
  finally { loading.value = false }
}

async function loadRemote() {
  if (!props.sessionId) return
  loading.value = true
  try { remoteFiles.value = await invoke('sftp_list_remote', { sessionId: props.sessionId, path: remotePath.value }).then(files => files.map(f => ({ ...f, _dragOver: false }))) }
  catch { remoteFiles.value = [] }
  finally { loading.value = false }
}

function refreshRemote() { loadRemote() }

function onLocalDblClick(file) {
  if (file.isDir) { localPath.value = file.path; loadLocal() }
}

function onRemoteDblClick(file) {
  if (file.isDir) { remotePath.value = file.path; loadRemote() }
  else { openEdit(file.path) }
}

// ─── Upload / Download ───

async function doUpload() {
  if (!props.sessionId) return
  const paths = selectedLocalSet.size > 0 ? [...selectedLocalSet] : (selectedLocal.value ? [selectedLocal.value] : [])
  for (const p of paths) {
    const name = p.split('/').pop()
    const remote = remotePath.value === '/' ? `/${name}` : `${remotePath.value}/${name}`
    const t = { id: crypto.randomUUID(), label: `↑ ${name}`, status: 'pending' }
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
    const t = { id: crypto.randomUUID(), label: `↓ ${name}`, status: 'pending' }
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
  if (!await showConfirm(`确认删除 ${selectedRemoteSet.size} 个文件/目录？`)) return
  for (const p of [...selectedRemoteSet]) {
    const file = remoteFiles.value.find(f => f.path === p)
    if (file) {
      try { await invoke('sftp_delete', { sessionId: props.sessionId, path: p, isDir: file.isDir }) }
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
  const newName = await showPrompt('新名称:', file.name)
  if (!newName || newName === file.name) return
  const dir = file.path.substring(0, file.path.lastIndexOf('/'))
  const newPath = dir ? `${dir}/${newName}` : `/${newName}`
  try { await invoke('sftp_rename', { sessionId: props.sessionId, oldPath: file.path, newPath }); loadRemote() }
  catch (e) { _toast('重命名失败: ' + e, 'error') }
}

async function ctxDelete() {
  const file = ctxMenu.value.file
  if (!file) return
  if (!await showConfirm(`确认删除 ${file.name}?`)) return
  try { await invoke('sftp_delete', { sessionId: props.sessionId, path: file.path, isDir: file.isDir }); loadRemote() }
  catch (e) { _toast('删除失败: ' + e, 'error') }
}

async function ctxChmod() {
  const file = ctxMenu.value.file
  if (!file) return
  const mode = await showPrompt('权限 (如 755):', '644')
  if (!mode) return
  try { await invoke('sftp_chmod', { sessionId: props.sessionId, path: file.path, mode }); loadRemote() }
  catch (e) { _toast('修改权限失败: ' + e, 'error') }
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
  } catch (e) { _toast('读取文件失败: ' + e, 'error') }
}

function closeEditor() {
  showInlineEditor.value = false
}

async function saveEdit() {
  try {
    await invoke('sftp_write_file', { sessionId: props.sessionId, path: editFile.value.path, content: editFile.value.content })
    editFile.value.dirty = false
    _toast('保存成功', 'success')
  } catch (e) { _toast('保存失败: ' + e, 'error') }
}

async function createRemoteDir() {
  if (!props.sessionId) return
  const name = await showPrompt('目录名称:')
  if (!name) return
  const path = remotePath.value === '/' ? `/${name}` : `${remotePath.value}/${name}`
  try { await invoke('sftp_mkdir', { sessionId: props.sessionId, path }); loadRemote() }
  catch (e) { _toast('创建失败: ' + e, 'error') }
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
