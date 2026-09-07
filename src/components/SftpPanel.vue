<template>
  <div class="file-workspace flex flex-col h-full">
    <div class="toolbar">
      <button class="toolbar-button" type="button" :disabled="!sessionId" title="新建远程目录" aria-label="新建远程目录" @click="createRemoteDir">
        <FolderPlus :size="14" :stroke-width="1.8" />
      </button>
      <button class="toolbar-button" type="button" :disabled="!sessionId || selectedRemoteSet.size === 0" title="删除选中项" aria-label="删除选中项" @click="batchDelete">
        <Trash2 :size="14" :stroke-width="1.8" />
        <span v-if="selectedRemoteSet.size" class="button-count">{{ selectedRemoteSet.size }}</span>
      </button>
      <button class="toolbar-button" type="button" :disabled="!sessionId" title="刷新远程目录" aria-label="刷新远程目录" @click="loadRemote">
        <RefreshCw :size="14" :stroke-width="1.8" />
      </button>
      <div class="flex-1" />
      <span v-if="selectedRemoteSet.size" class="toolbar-meta">已选 {{ selectedRemoteSet.size }} 个</span>
      <span class="toolbar-meta truncate max-w-44">{{ connection?.name || '未连接' }}</span>
    </div>

    <div class="flex-1 flex min-h-0">
      <section class="file-pane" :style="{ width: localWidth + '%' }" aria-label="本地文件">
        <div class="pane-header">
          <Home :size="14" :stroke-width="1.8" class="pane-icon local" />
          <span class="pane-title">本地</span>
          <input v-model="localPath" class="path-input" aria-label="本地路径" @keydown.enter="loadLocal" />
          <button class="icon-button" type="button" aria-label="刷新本地目录" title="刷新" @click="loadLocal">
            <RefreshCw :size="13" :stroke-width="1.8" />
          </button>
        </div>

        <div class="file-list" @click.self="clearLocalSelection">
          <div v-if="localLoading" class="pane-state">
            <LoaderCircle :size="20" :stroke-width="1.7" class="spin" />
            <span>正在读取本地目录</span>
          </div>
          <div v-else-if="localError" class="pane-state error">
            <CircleAlert :size="20" :stroke-width="1.7" />
            <span>{{ localError }}</span>
            <button type="button" @click="loadLocal">重试</button>
          </div>
          <div v-else-if="localFiles.length === 0" class="pane-state">
            <FolderOpen :size="20" :stroke-width="1.7" />
            <span>空目录</span>
          </div>

          <VirtualList v-else-if="localVirtual" :items="localFiles" :item-height="28" :height="400">
            <template #default="{ item: file }">
              <FileRow
                :file="file"
                :selected="selectedLocalSet.has(file.path)"
                compact
                @click="onLocalClick($event, file)"
                @dblclick="onLocalDblClick(file)"
                @dragstart="onLocalDragStart($event, file)"
              />
            </template>
          </VirtualList>

          <template v-else>
            <FileRow
              v-for="file in localFiles"
              :key="file.path"
              :file="file"
              :selected="selectedLocalSet.has(file.path)"
              @click="onLocalClick($event, file)"
              @dblclick="onLocalDblClick(file)"
              @dragstart="onLocalDragStart($event, file)"
            />
          </template>
        </div>
      </section>

      <div class="resize-divider" aria-hidden="true" @mousedown="startLocalResize" />

      <div class="transfer-actions" aria-label="文件传输操作">
        <button
          type="button"
          class="transfer-button"
          :disabled="!sessionId || localTransferPaths.length === 0"
          :title="localTransferPaths.length > 1 ? `上传 ${localTransferPaths.length} 个文件` : '上传到远程'"
          aria-label="上传到远程"
          @click="doUpload"
        >
          <Upload :size="16" :stroke-width="1.9" />
        </button>
        <button
          type="button"
          class="transfer-button"
          :disabled="!sessionId || remoteTransferPaths.length === 0"
          :title="remoteTransferPaths.length > 1 ? `下载 ${remoteTransferPaths.length} 个文件` : '下载到本地'"
          aria-label="下载到本地"
          @click="doDownload"
        >
          <Download :size="16" :stroke-width="1.9" />
        </button>
      </div>

      <section
        class="file-pane flex-1"
        :class="{ 'drag-target': remoteDragOver }"
        aria-label="远程文件"
        @dragover.prevent="remoteDragOver = true"
        @dragleave="handleRemoteDragLeave"
        @drop.prevent="onRemoteDrop"
      >
        <div class="pane-header">
          <Server :size="14" :stroke-width="1.8" class="pane-icon remote" />
          <span class="pane-title">远程</span>
          <input v-model="remotePath" class="path-input" aria-label="远程路径" :disabled="!sessionId" @keydown.enter="loadRemote" />
          <button class="icon-button" type="button" :disabled="!sessionId" aria-label="刷新远程目录" title="刷新" @click="loadRemote">
            <RefreshCw :size="13" :stroke-width="1.8" />
          </button>
        </div>

        <Transition name="slide-right">
          <div v-if="showInlineEditor" class="editor-pane">
            <div class="editor-header">
              <FilePenLine :size="15" :stroke-width="1.8" />
              <span class="editor-name">{{ editFile.path.split('/').pop() }}</span>
              <span v-if="editFile.dirty" class="dirty-badge">未保存</span>
              <div class="flex-1" />
              <button class="secondary-action" type="button" :disabled="savingEdit" @click="saveEdit">
                <Save :size="13" :stroke-width="1.8" />
                <span>{{ savingEdit ? '保存中' : '保存' }}</span>
              </button>
              <button class="icon-button" type="button" aria-label="关闭编辑器" title="关闭" @click="closeEditor">
                <X :size="14" :stroke-width="1.8" />
              </button>
            </div>
            <textarea
              v-model="editFile.content"
              class="editor-textarea"
              spellcheck="false"
              @input="editFile.dirty = true"
            />
          </div>
        </Transition>

        <div v-if="!showInlineEditor" class="file-list" @click.self="clearRemoteSelection">
          <div v-if="!sessionId" class="pane-state">
            <Unplug :size="20" :stroke-width="1.7" />
            <span>请先连接服务器</span>
          </div>
          <div v-else-if="remoteLoading" class="pane-state">
            <LoaderCircle :size="20" :stroke-width="1.7" class="spin" />
            <span>正在读取远程目录</span>
          </div>
          <div v-else-if="remoteError" class="pane-state error">
            <CircleAlert :size="20" :stroke-width="1.7" />
            <span>{{ remoteError }}</span>
            <button type="button" @click="loadRemote">重试</button>
          </div>
          <div v-else-if="remoteFiles.length === 0" class="pane-state">
            <FolderOpen :size="20" :stroke-width="1.7" />
            <span>空目录</span>
          </div>

          <VirtualList v-else-if="remoteVirtual" :items="remoteFiles" :item-height="28" :height="400">
            <template #default="{ item: file }">
              <FileRow
                :file="file"
                :selected="selectedRemoteSet.has(file.path)"
                :show-permissions="true"
                compact
                @click="onRemoteClick($event, file)"
                @dblclick="onRemoteDblClick(file)"
                @contextmenu.prevent="showRemoteMenu($event, file)"
                @dragstart="onRemoteDragStart($event, file)"
              />
            </template>
          </VirtualList>

          <template v-else>
            <FileRow
              v-for="file in remoteFiles"
              :key="file.path"
              :file="file"
              :selected="selectedRemoteSet.has(file.path)"
              :show-permissions="true"
              :drag-over="file._dragOver && file.isDir"
              @click="onRemoteClick($event, file)"
              @dblclick="onRemoteDblClick(file)"
              @contextmenu.prevent="showRemoteMenu($event, file)"
              @dragstart="onRemoteDragStart($event, file)"
              @dragover.prevent="file._dragOver = true"
              @dragleave="file._dragOver = false"
              @drop.prevent.stop="onRemoteDirDrop($event, file)"
            />
          </template>
        </div>
      </section>
    </div>

    <div v-if="transfers.length > 0" class="transfer-queue">
      <div class="queue-header">
        <span>传输队列</span>
        <span v-if="pendingTransferCount">{{ pendingTransferCount }} 进行中</span>
        <div class="flex-1" />
        <button type="button" @click="clearFinishedTransfers">清除已完成</button>
      </div>
      <div v-for="transfer in transfers" :key="transfer.id" class="queue-row">
        <Upload v-if="transfer.direction === 'upload'" :size="13" :stroke-width="1.8" />
        <Download v-else :size="13" :stroke-width="1.8" />
        <span class="flex-1 truncate">{{ transfer.name }}</span>
        <LoaderCircle v-if="transfer.status === 'pending'" :size="13" :stroke-width="1.8" class="spin pending" />
        <CheckCircle2 v-else-if="transfer.status === 'done'" :size="13" :stroke-width="1.8" class="done" />
        <CircleAlert v-else :size="13" :stroke-width="1.8" class="failed" :title="transfer.error" />
        <button v-if="transfer.status !== 'pending'" class="queue-remove" type="button" aria-label="移除传输记录" @click="removeTransfer(transfer.id)">
          <X :size="12" :stroke-width="1.8" />
        </button>
      </div>
    </div>

    <div v-if="ctxMenu.show" class="context-backdrop" @pointerdown="closeContextMenu" />
    <div v-if="ctxMenu.show" class="context-menu" role="menu" :style="{ left: ctxMenu.x + 'px', top: ctxMenu.y + 'px' }" @pointerdown.stop>
      <button v-if="!ctxMenu.file?.isDir" type="button" role="menuitem" @click="ctxEdit">
        <FilePenLine :size="14" :stroke-width="1.8" /><span>在线编辑</span>
      </button>
      <button type="button" role="menuitem" @click="ctxRename">
        <Pencil :size="14" :stroke-width="1.8" /><span>重命名</span>
      </button>
      <button v-if="!ctxMenu.file?.isDir" type="button" role="menuitem" @click="ctxDownload">
        <Download :size="14" :stroke-width="1.8" /><span>下载</span>
      </button>
      <div class="menu-separator" />
      <button type="button" role="menuitem" @click="ctxChmod">
        <Shield :size="14" :stroke-width="1.8" /><span>修改权限</span>
      </button>
      <button type="button" role="menuitem" @click="ctxCopyPath">
        <Copy :size="14" :stroke-width="1.8" /><span>复制路径</span>
      </button>
      <div class="menu-separator" />
      <button type="button" role="menuitem" class="danger" @click="ctxDelete">
        <Trash2 :size="14" :stroke-width="1.8" /><span>删除</span>
      </button>
    </div>

    <Teleport to="body">
      <div v-if="promptState.show" class="dialog-backdrop" @click.self="promptResolve(null)">
        <div class="mini-dialog" role="dialog" aria-modal="true" :aria-label="promptState.title" @keydown.esc="promptResolve(null)">
          <div class="dialog-title">{{ promptState.title }}</div>
          <input ref="promptInputRef" v-model="promptState.value" class="dialog-input" @keydown.enter="promptResolve(promptState.value)" />
          <div class="dialog-actions">
            <button type="button" class="secondary-button" @click="promptResolve(null)">取消</button>
            <button type="button" class="primary-button" @click="promptResolve(promptState.value)">确定</button>
          </div>
        </div>
      </div>

      <div v-if="confirmState.show" class="dialog-backdrop" @click.self="confirmResolve(false)">
        <div class="mini-dialog" role="alertdialog" aria-modal="true" :aria-label="confirmState.title" @keydown.esc="confirmResolve(false)">
          <div class="dialog-title">{{ confirmState.title }}</div>
          <div class="dialog-message">{{ confirmState.message }}</div>
          <div class="dialog-actions">
            <button type="button" class="secondary-button" @click="confirmResolve(false)">取消</button>
            <button type="button" :class="confirmState.danger ? 'danger-button' : 'primary-button'" @click="confirmResolve(true)">
              {{ confirmState.confirmLabel }}
            </button>
          </div>
        </div>
      </div>
    </Teleport>

    <div v-if="toastState.show" class="file-toast" :class="toastState.type">{{ toastState.message }}</div>
  </div>
</template>

<script setup>
import { computed, defineComponent, h, nextTick, onMounted, onUnmounted, reactive, ref, watch } from 'vue'
import {
  Archive,
  CheckCircle2,
  CircleAlert,
  Copy,
  Download,
  File,
  FileCode2,
  FileJson,
  FilePenLine,
  FileText,
  Folder,
  FolderOpen,
  FolderPlus,
  Home,
  Image,
  LoaderCircle,
  Pencil,
  RefreshCw,
  Save,
  Server,
  Settings2,
  Shield,
  Trash2,
  Unplug,
  Upload,
  X,
} from 'lucide-vue-next'
import { invoke, isTauri } from '../utils/tauri.js'
import { formatBytes } from '../utils/format.js'
import { useFileSelection } from '../composables/useFileSelection.js'
import { useSplitResize } from '../composables/useSplitResize.js'
import VirtualList from './VirtualList.vue'

const props = defineProps({
  connection: Object,
  sessionId: String,
  active: Boolean,
  navigationTarget: Object,
})
const { startResize } = useSplitResize()

const toastState = ref({ show: false, message: '', type: 'info' })
let toastTimer = null
function toast(message, type = 'info', duration = 2500) {
  if (toastTimer) clearTimeout(toastTimer)
  toastState.value = { show: true, message, type }
  toastTimer = setTimeout(() => { toastState.value.show = false }, duration)
}

const promptInputRef = ref(null)
const promptState = ref({ show: false, title: '', value: '' })
let promptResolveFn = null
function showPrompt(title, defaultValue = '') {
  return new Promise(resolve => {
    promptState.value = { show: true, title, value: defaultValue }
    promptResolveFn = resolve
    nextTick(() => promptInputRef.value?.focus())
  })
}
function promptResolve(value) {
  promptState.value.show = false
  promptResolveFn?.(value)
  promptResolveFn = null
}

const confirmState = ref({ show: false, title: '确认操作', message: '', danger: false, confirmLabel: '确定' })
let confirmResolveFn = null
function showConfirm(message, { title = '确认操作', danger = false, confirmLabel = '确定' } = {}) {
  return new Promise(resolve => {
    confirmState.value = { show: true, title, message, danger, confirmLabel }
    confirmResolveFn = resolve
  })
}
function confirmResolve(value) {
  confirmState.value.show = false
  confirmResolveFn?.(value)
  confirmResolveFn = null
}

const localPath = ref('')
const remotePath = ref('/')
const localFiles = ref([])
const remoteFiles = ref([])
const localLoading = ref(false)
const remoteLoading = ref(false)
const localError = ref('')
const remoteError = ref('')
const selectedLocal = ref(null)
const selectedRemote = ref(null)
const selectedLocalSet = reactive(new Set())
const selectedRemoteSet = reactive(new Set())
const localVirtual = computed(() => localFiles.value.length > 100)
const remoteVirtual = computed(() => remoteFiles.value.length > 100)
const remoteDragOver = ref(false)
const localWidth = ref(50)
const transfers = ref([])
const editFile = ref({ path: '', content: '', dirty: false })
const showInlineEditor = ref(false)
const savingEdit = ref(false)
const ctxMenu = ref({ show: false, x: 0, y: 0, file: null })
let appliedNavigationRequestId = null

const { onClick: onLocalClick, onDragStart: onLocalDragStart } = useFileSelection(selectedLocalSet, localFiles, {
  dragSource: 'local',
  onSelect: path => { selectedLocal.value = path },
})
const { onClick: onRemoteClick, onDragStart: onRemoteDragStart } = useFileSelection(selectedRemoteSet, remoteFiles, {
  dragSource: 'remote',
  onSelect: path => { selectedRemote.value = path },
})

const localTransferPaths = computed(() => transferPaths(selectedLocalSet, selectedLocal.value, localFiles.value))
const remoteTransferPaths = computed(() => transferPaths(selectedRemoteSet, selectedRemote.value, remoteFiles.value))
const pendingTransferCount = computed(() => transfers.value.filter(item => item.status === 'pending').length)

const codeExtensions = new Set(['js', 'jsx', 'ts', 'tsx', 'vue', 'py', 'rs', 'go', 'java', 'kt', 'kts', 'c', 'h', 'cpp', 'hpp', 'cs', 'php', 'rb', 'sh', 'bash', 'zsh', 'fish', 'html', 'css', 'scss', 'less', 'sql'])
const textExtensions = new Set(['md', 'txt', 'log', 'csv'])
const imageExtensions = new Set(['png', 'jpg', 'jpeg', 'gif', 'webp', 'svg', 'ico'])
const archiveExtensions = new Set(['zip', 'tar', 'gz', 'tgz', 'bz2', 'xz', '7z', 'rar'])
const configExtensions = new Set(['yaml', 'yml', 'toml', 'ini', 'conf', 'env', 'properties'])

function fileIcon(file) {
  if (file.isDir) return Folder
  const extension = file.name.split('.').pop()?.toLowerCase() || ''
  if (codeExtensions.has(extension)) return FileCode2
  if (extension === 'json') return FileJson
  if (textExtensions.has(extension)) return FileText
  if (imageExtensions.has(extension)) return Image
  if (archiveExtensions.has(extension)) return Archive
  if (configExtensions.has(extension)) return Settings2
  return File
}

const FileRow = defineComponent({
  name: 'SftpFileRow',
  inheritAttrs: false,
  props: {
    file: { type: Object, required: true },
    selected: Boolean,
    compact: Boolean,
    showPermissions: Boolean,
    dragOver: Boolean,
  },
  setup(rowProps, { attrs }) {
    return () => h('div', {
      ...attrs,
      class: ['file-row', { selected: rowProps.selected, compact: rowProps.compact, 'drag-over': rowProps.dragOver }],
      draggable: true,
    }, [
      h(fileIcon(rowProps.file), { size: 14, strokeWidth: 1.7, class: 'file-row-icon' }),
      h('span', { class: 'file-name' }, rowProps.file.name),
      h('span', { class: 'file-size' }, rowProps.file.isDir ? '' : formatBytes(rowProps.file.size)),
      rowProps.showPermissions ? h('span', { class: 'file-permissions' }, rowProps.file.permissions || '') : null,
    ])
  },
})

function transferPaths(selectedSet, selectedPath, files) {
  const requested = selectedSet.size > 0 ? [...selectedSet] : (selectedPath ? [selectedPath] : [])
  return requested.filter(path => {
    const file = files.find(item => item.path === path)
    return file && !file.isDir && file.name !== '..'
  })
}

function clearLocalSelection() {
  selectedLocalSet.clear()
  selectedLocal.value = null
}
function clearRemoteSelection() {
  selectedRemoteSet.clear()
  selectedRemote.value = null
}

function startLocalResize(event) {
  startResize(event, event.target.parentElement, localWidth)
}

async function resolveInitialLocalPath() {
  if (isTauri) {
    try {
      const { homeDir } = await import('@tauri-apps/api/path')
      return await homeDir()
    } catch {}
  }
  return '.'
}

async function loadLocal() {
  if (!localPath.value) return
  localLoading.value = true
  localError.value = ''
  try {
    localFiles.value = await invoke('sftp_list_local', { path: localPath.value })
    clearLocalSelection()
  } catch (error) {
    localFiles.value = []
    localError.value = `无法读取本地目录：${error}`
  } finally {
    localLoading.value = false
  }
}

async function loadRemote() {
  if (!props.sessionId) {
    remoteFiles.value = []
    remoteError.value = ''
    clearRemoteSelection()
    return
  }

  remoteLoading.value = true
  remoteError.value = ''
  try {
    const files = await invoke('sftp_list_remote', { sessionId: props.sessionId, path: remotePath.value })
    remoteFiles.value = files.map(file => ({ ...file, _dragOver: false }))
    clearRemoteSelection()
  } catch (error) {
    remoteFiles.value = []
    remoteError.value = `无法读取远程目录：${error}`
  } finally {
    remoteLoading.value = false
  }
}

async function applyNavigationTarget() {
  const target = props.navigationTarget
  if (!props.sessionId || !target?.path) return false
  if (target.requestId && target.requestId === appliedNavigationRequestId) return true

  showInlineEditor.value = false
  editFile.value = { path: '', content: '', dirty: false }
  remotePath.value = target.path
  await loadRemote()
  appliedNavigationRequestId = target.requestId || null
  return true
}

function onLocalDblClick(file) {
  if (!file.isDir) return
  localPath.value = file.path
  loadLocal()
}

function onRemoteDblClick(file) {
  if (file.isDir) {
    remotePath.value = file.path
    loadRemote()
  } else {
    openEdit(file.path)
  }
}

function readDragPayload(event) {
  const raw = event.dataTransfer?.getData('application/x-xterminal-files')
  if (!raw) return null
  try {
    const payload = JSON.parse(raw)
    if (!['local', 'remote'].includes(payload.source) || !Array.isArray(payload.paths)) return null
    return payload
  } catch {
    return null
  }
}

function handleRemoteDragLeave(event) {
  if (!event.currentTarget.contains(event.relatedTarget)) remoteDragOver.value = false
}

async function onRemoteDrop(event) {
  remoteDragOver.value = false
  if (!props.sessionId) return
  const payload = readDragPayload(event)
  if (!payload) return

  if (payload.source === 'remote') return
  await uploadPaths(payload.paths, remotePath.value)
}

async function onRemoteDirDrop(event, targetFile) {
  targetFile._dragOver = false
  if (!targetFile.isDir || !props.sessionId) return
  const payload = readDragPayload(event)
  if (!payload) return

  if (payload.source === 'local') {
    await uploadPaths(payload.paths, targetFile.path)
    return
  }

  for (const path of payload.paths) {
    const name = path.split('/').pop()
    const destination = targetFile.path === '/' ? `/${name}` : `${targetFile.path}/${name}`
    if (path === destination) continue
    try {
      await invoke('sftp_rename', { sessionId: props.sessionId, oldPath: path, newPath: destination })
    } catch (error) {
      toast(`移动失败：${error}`, 'error')
      break
    }
  }
  await loadRemote()
}

function createTransfer(direction, name) {
  const transfer = { id: crypto.randomUUID(), direction, name, status: 'pending', error: '' }
  transfers.value.push(transfer)
  return transfer
}

async function uploadPaths(paths, targetDirectory) {
  if (!props.sessionId) return
  const filePaths = paths.filter(path => {
    const file = localFiles.value.find(item => item.path === path)
    return file && !file.isDir && file.name !== '..'
  })

  if (filePaths.length === 0) {
    toast('当前上传仅支持文件，不支持目录', 'info')
    return
  }

  for (const path of filePaths) {
    const name = path.split(/[\\/]/).pop()
    const remote = targetDirectory === '/' ? `/${name}` : `${targetDirectory.replace(/\/$/, '')}/${name}`
    const transfer = createTransfer('upload', name)
    try {
      await invoke('sftp_upload', { sessionId: props.sessionId, local: path, remote })
      transfer.status = 'done'
    } catch (error) {
      transfer.status = 'error'
      transfer.error = String(error)
    }
  }
  await loadRemote()
}

async function doUpload() {
  await uploadPaths(localTransferPaths.value, remotePath.value)
}

async function doDownload() {
  if (!props.sessionId) return
  if (remoteTransferPaths.value.length === 0) {
    toast('当前下载仅支持文件，不支持目录', 'info')
    return
  }

  for (const path of remoteTransferPaths.value) {
    const name = path.split('/').pop()
    const separator = localPath.value.endsWith('/') || localPath.value.endsWith('\\') ? '' : '/'
    const local = `${localPath.value}${separator}${name}`
    const transfer = createTransfer('download', name)
    try {
      await invoke('sftp_download', { sessionId: props.sessionId, remote: path, local })
      transfer.status = 'done'
    } catch (error) {
      transfer.status = 'error'
      transfer.error = String(error)
    }
  }
  await loadLocal()
}

function clearFinishedTransfers() {
  transfers.value = transfers.value.filter(item => item.status === 'pending')
}
function removeTransfer(id) {
  transfers.value = transfers.value.filter(item => item.id !== id)
}

async function batchDelete() {
  if (!props.sessionId || selectedRemoteSet.size === 0) return
  const confirmed = await showConfirm(`确认删除选中的 ${selectedRemoteSet.size} 个文件或目录？`, {
    title: '删除远程文件',
    danger: true,
    confirmLabel: '删除',
  })
  if (!confirmed) return

  let failed = 0
  for (const path of [...selectedRemoteSet]) {
    const file = remoteFiles.value.find(item => item.path === path)
    if (!file || file.name === '..') continue
    try {
      await invoke('sftp_delete', { sessionId: props.sessionId, path, isDir: file.isDir })
    } catch {
      failed++
    }
  }
  clearRemoteSelection()
  await loadRemote()
  if (failed) toast(`${failed} 项删除失败`, 'error')
}

function showRemoteMenu(event, file) {
  if (file.name === '..') return
  if (!selectedRemoteSet.has(file.path)) {
    selectedRemoteSet.clear()
    selectedRemoteSet.add(file.path)
    selectedRemote.value = file.path
  }
  ctxMenu.value = {
    show: true,
    x: Math.min(event.clientX, window.innerWidth - 170),
    y: Math.min(event.clientY, window.innerHeight - 230),
    file,
  }
}

function closeContextMenu() {
  ctxMenu.value.show = false
}

async function ctxRename() {
  const file = ctxMenu.value.file
  closeContextMenu()
  if (!file) return
  const newName = await showPrompt('重命名', file.name)
  if (!newName || newName === file.name) return
  const dir = file.path.substring(0, file.path.lastIndexOf('/'))
  const newPath = dir ? `${dir}/${newName}` : `/${newName}`
  try {
    await invoke('sftp_rename', { sessionId: props.sessionId, oldPath: file.path, newPath })
    await loadRemote()
  } catch (error) {
    toast(`重命名失败：${error}`, 'error')
  }
}

async function ctxDelete() {
  const file = ctxMenu.value.file
  closeContextMenu()
  if (!file) return
  const confirmed = await showConfirm(`确认删除“${file.name}”？${file.isDir ? ' 目录内容也会一并删除。' : ''}`, {
    title: '删除远程文件',
    danger: true,
    confirmLabel: '删除',
  })
  if (!confirmed) return
  try {
    await invoke('sftp_delete', { sessionId: props.sessionId, path: file.path, isDir: file.isDir })
    await loadRemote()
  } catch (error) {
    toast(`删除失败：${error}`, 'error')
  }
}

async function ctxChmod() {
  const file = ctxMenu.value.file
  closeContextMenu()
  if (!file) return
  const mode = await showPrompt('修改权限', file.isDir ? '755' : '644')
  if (!mode) return
  try {
    await invoke('sftp_chmod', { sessionId: props.sessionId, path: file.path, mode })
    await loadRemote()
  } catch (error) {
    toast(`修改权限失败：${error}`, 'error')
  }
}

function ctxEdit() {
  const file = ctxMenu.value.file
  closeContextMenu()
  if (file && !file.isDir) openEdit(file.path)
}

function ctxDownload() {
  const file = ctxMenu.value.file
  closeContextMenu()
  if (!file || file.isDir) return
  selectedRemoteSet.clear()
  selectedRemoteSet.add(file.path)
  selectedRemote.value = file.path
  doDownload()
}

async function ctxCopyPath() {
  const file = ctxMenu.value.file
  closeContextMenu()
  if (!file) return
  try {
    await navigator.clipboard.writeText(file.path)
    toast('路径已复制', 'success', 1200)
  } catch {
    toast('复制路径失败', 'error')
  }
}

async function openEdit(path) {
  if (!props.sessionId) return
  remoteLoading.value = true
  try {
    const content = await invoke('sftp_read_file', { sessionId: props.sessionId, path })
    editFile.value = { path, content, dirty: false }
    showInlineEditor.value = true
  } catch (error) {
    toast(`读取文件失败：${error}`, 'error')
  } finally {
    remoteLoading.value = false
  }
}

async function closeEditor() {
  if (editFile.value.dirty) {
    const confirmed = await showConfirm('当前文件有未保存的修改，确定放弃并关闭？', {
      title: '放弃未保存修改',
      danger: true,
      confirmLabel: '放弃修改',
    })
    if (!confirmed) return
  }
  showInlineEditor.value = false
  editFile.value = { path: '', content: '', dirty: false }
}

async function saveEdit() {
  if (!props.sessionId || !editFile.value.path || savingEdit.value) return
  savingEdit.value = true
  try {
    await invoke('sftp_write_file', {
      sessionId: props.sessionId,
      path: editFile.value.path,
      content: editFile.value.content,
    })
    editFile.value.dirty = false
    toast('保存成功', 'success')
  } catch (error) {
    toast(`保存失败：${error}`, 'error')
  } finally {
    savingEdit.value = false
  }
}

async function createRemoteDir() {
  if (!props.sessionId) return
  const name = await showPrompt('新建目录')
  if (!name?.trim()) return
  const cleanName = name.trim()
  if (cleanName.includes('/') || cleanName === '.' || cleanName === '..') {
    toast('目录名称不能包含 /、. 或 ..', 'error')
    return
  }
  const path = remotePath.value === '/' ? `/${cleanName}` : `${remotePath.value.replace(/\/$/, '')}/${cleanName}`
  try {
    await invoke('sftp_mkdir', { sessionId: props.sessionId, path })
    await loadRemote()
  } catch (error) {
    toast(`创建目录失败：${error}`, 'error')
  }
}

function handleGlobalKeydown(event) {
  if (event.key !== 'Escape') return
  if (ctxMenu.value.show) {
    closeContextMenu()
  } else if (showInlineEditor.value && !promptState.value.show && !confirmState.value.show) {
    closeEditor()
  }
}

onMounted(async () => {
  localPath.value = await resolveInitialLocalPath()
  await loadLocal()
  window.addEventListener('keydown', handleGlobalKeydown)

  if (props.sessionId) {
    const navigated = await applyNavigationTarget()
    if (!navigated) await loadRemote()
  }
})

onUnmounted(() => {
  if (toastTimer) clearTimeout(toastTimer)
  window.removeEventListener('keydown', handleGlobalKeydown)
  promptResolveFn?.(null)
  confirmResolveFn?.(false)
})

watch(() => props.navigationTarget, async () => {
  await applyNavigationTarget()
})

watch(() => props.sessionId, async sessionId => {
  showInlineEditor.value = false
  editFile.value = { path: '', content: '', dirty: false }
  if (sessionId) {
    const navigated = await applyNavigationTarget()
    if (!navigated) {
      remotePath.value = '/'
      await loadRemote()
    }
  } else {
    remotePath.value = '/'
    remoteFiles.value = []
    remoteError.value = ''
    clearRemoteSelection()
  }
})
</script>

<style scoped>
.file-workspace {
  position: relative;
  background: var(--bg-base);
  color: var(--fg-secondary);
}

.toolbar,
.pane-header,
.editor-header,
.queue-header {
  display: flex;
  align-items: center;
  flex-shrink: 0;
  background: var(--bg-surface);
  border-bottom: 1px solid var(--border-subtle);
}

.toolbar {
  height: 36px;
  gap: 4px;
  padding: 0 8px;
}

.toolbar-button,
.icon-button,
.queue-remove {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 0;
  border-radius: 5px;
  background: transparent;
  color: var(--fg-muted);
  transition: background-color var(--transition-fast), color var(--transition-fast);
}

.toolbar-button {
  position: relative;
  min-width: 27px;
  height: 27px;
  padding: 0 6px;
  gap: 4px;
}

.toolbar-button:hover:not(:disabled),
.icon-button:hover:not(:disabled),
.queue-remove:hover {
  background: var(--bg-hover);
  color: var(--fg-primary);
}

.toolbar-button:disabled,
.icon-button:disabled,
.transfer-button:disabled,
.secondary-action:disabled {
  cursor: not-allowed;
  opacity: 0.38;
}

.button-count {
  color: var(--fg-secondary);
  font-size: 10px;
}

.toolbar-meta {
  color: var(--fg-muted);
  font-size: 11px;
}

.file-pane {
  min-width: 160px;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  transition: box-shadow var(--transition-fast);
}

.file-pane.drag-target {
  box-shadow: inset 0 0 0 2px var(--accent);
}

.pane-header {
  height: 32px;
  gap: 6px;
  padding: 0 7px;
}

.pane-icon.local { color: var(--success); }
.pane-icon.remote { color: var(--accent); }

.pane-title {
  color: var(--fg-muted);
  font-size: 11px;
  font-weight: 600;
}

.path-input {
  flex: 1;
  min-width: 0;
  height: 25px;
  padding: 0 7px;
  border: 1px solid transparent;
  border-radius: 5px;
  outline: none;
  background: transparent;
  color: var(--fg-secondary);
  font-family: monospace;
  font-size: 11px;
}

.path-input:hover,
.path-input:focus {
  border-color: var(--border);
  background: var(--bg-base);
}

.icon-button {
  width: 25px;
  height: 25px;
  flex-shrink: 0;
}

.file-list {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
}

.file-row {
  height: 30px;
  display: flex;
  align-items: center;
  gap: 7px;
  padding: 0 9px;
  color: var(--fg-secondary);
  font-size: 12px;
  cursor: default;
  user-select: none;
  transition: background-color var(--transition-fast), color var(--transition-fast);
}

.file-row.compact { height: 28px; }
.file-row:hover { background: var(--bg-hover); }
.file-row.selected { background: var(--accent-hover); color: var(--accent); }
.file-row.drag-over { box-shadow: inset 0 0 0 1px var(--accent); background: var(--accent-hover); }
.file-row-icon { flex-shrink: 0; color: var(--fg-muted); }
.file-row.selected .file-row-icon { color: var(--accent); }
.file-name { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.file-size { width: 58px; flex-shrink: 0; color: var(--fg-muted); font-size: 10px; text-align: right; }
.file-permissions { width: 72px; flex-shrink: 0; overflow: hidden; color: var(--fg-muted); font-family: monospace; font-size: 10px; text-align: right; }

.pane-state {
  height: 100%;
  min-height: 140px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 20px;
  color: var(--fg-muted);
  font-size: 11px;
  text-align: center;
}

.pane-state.error { color: var(--danger); }
.pane-state button {
  padding: 4px 8px;
  border: 0;
  border-radius: 5px;
  background: var(--bg-hover);
  color: var(--fg-secondary);
  font-size: 11px;
}

.resize-divider {
  width: 3px;
  flex-shrink: 0;
  cursor: col-resize;
  background: var(--border-subtle);
  transition: background-color var(--transition-fast);
}
.resize-divider:hover { background: var(--accent); }

.transfer-actions {
  width: 38px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 5px;
  flex-shrink: 0;
  background: var(--bg-surface);
  border-right: 1px solid var(--border-subtle);
}

.transfer-button {
  width: 28px;
  height: 28px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 0;
  border-radius: 6px;
  background: transparent;
  color: var(--fg-muted);
  transition: background-color var(--transition-fast), color var(--transition-fast);
}
.transfer-button:hover:not(:disabled) { background: var(--accent-hover); color: var(--accent); }

.editor-pane {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  background: var(--bg-base);
}

.editor-header {
  min-height: 38px;
  gap: 7px;
  padding: 0 9px;
  color: var(--fg-muted);
}
.editor-name { min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; color: var(--fg-primary); font-size: 12px; font-weight: 500; }
.dirty-badge { padding: 1px 5px; border-radius: 4px; background: color-mix(in srgb, var(--warning) 16%, transparent); color: var(--warning); font-size: 10px; }

.secondary-action {
  height: 27px;
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 0 8px;
  border: 0;
  border-radius: 5px;
  background: var(--accent-hover);
  color: var(--accent);
  font-size: 11px;
}

.editor-textarea {
  flex: 1;
  min-height: 0;
  padding: 12px;
  resize: none;
  border: 0;
  outline: none;
  background: var(--bg-base);
  color: var(--fg-primary);
  font-family: 'Cascadia Code', 'Cascadia Mono', 'JetBrains Mono', Consolas, monospace;
  font-size: 12px;
  line-height: 1.55;
}

.transfer-queue {
  max-height: 120px;
  overflow-y: auto;
  flex-shrink: 0;
  border-top: 1px solid var(--border-subtle);
  background: var(--bg-base);
}
.queue-header { height: 28px; gap: 8px; padding: 0 8px; color: var(--fg-muted); font-size: 10px; }
.queue-header button { border: 0; background: transparent; color: var(--fg-muted); font-size: 10px; }
.queue-header button:hover { color: var(--fg-primary); }
.queue-row { min-height: 27px; display: flex; align-items: center; gap: 7px; padding: 0 8px; color: var(--fg-secondary); font-size: 11px; }
.queue-row > svg:first-child { color: var(--fg-muted); }
.queue-row .pending { color: var(--warning); }
.queue-row .done { color: var(--success); }
.queue-row .failed { color: var(--danger); }
.queue-remove { width: 20px; height: 20px; }

.context-backdrop { position: fixed; inset: 0; z-index: 40; }
.context-menu { position: fixed; z-index: 50; min-width: 164px; padding: 5px; border: 1px solid var(--border); border-radius: 8px; background: var(--bg-elevated); box-shadow: var(--shadow-lg); }
.context-menu button { width: 100%; height: 30px; display: flex; align-items: center; gap: 8px; padding: 0 8px; border: 0; border-radius: 5px; background: transparent; color: var(--fg-secondary); font-size: 12px; text-align: left; }
.context-menu button:hover { background: var(--bg-hover); color: var(--fg-primary); }
.context-menu button.danger { color: var(--danger); }
.menu-separator { height: 1px; margin: 4px 2px; background: var(--border-subtle); }

.dialog-backdrop { position: fixed; inset: 0; z-index: 60; display: flex; align-items: center; justify-content: center; padding: 20px; background: rgba(0, 0, 0, 0.62); }
.mini-dialog { width: min(340px, 100%); padding: 14px; border: 1px solid var(--border); border-radius: 9px; background: var(--bg-elevated); box-shadow: var(--shadow-lg); }
.dialog-title { color: var(--fg-primary); font-size: 13px; font-weight: 600; }
.dialog-message { margin-top: 8px; color: var(--fg-secondary); font-size: 12px; line-height: 1.5; }
.dialog-input { width: 100%; height: 31px; margin-top: 10px; padding: 0 9px; border: 1px solid var(--border); border-radius: 6px; outline: none; background: var(--bg-base); color: var(--fg-primary); font-size: 12px; }
.dialog-input:focus { border-color: var(--accent); }
.dialog-actions { display: flex; justify-content: flex-end; gap: 7px; margin-top: 13px; }
.secondary-button, .primary-button, .danger-button { height: 30px; padding: 0 11px; border: 0; border-radius: 6px; font-size: 12px; }
.secondary-button { background: var(--bg-hover); color: var(--fg-secondary); }
.primary-button { background: var(--accent); color: white; }
.danger-button { background: var(--danger); color: white; }

.file-toast { position: absolute; top: 8px; right: 8px; z-index: 55; max-width: min(420px, calc(100% - 16px)); padding: 7px 9px; border-radius: 6px; box-shadow: var(--shadow-sm); font-size: 11px; }
.file-toast.success { background: color-mix(in srgb, var(--success) 18%, var(--bg-elevated)); color: var(--success); }
.file-toast.error { background: color-mix(in srgb, var(--danger) 18%, var(--bg-elevated)); color: var(--danger); }
.file-toast.info { background: var(--bg-elevated); color: var(--fg-secondary); }

.spin { animation: spin 0.8s linear infinite; }
@keyframes spin { to { transform: rotate(360deg); } }

.slide-right-enter-active,
.slide-right-leave-active { transition: opacity var(--transition-fast), transform var(--transition-fast); }
.slide-right-enter-from,
.slide-right-leave-to { opacity: 0; transform: translateX(10px); }

@media (max-width: 900px) {
  .toolbar-meta:first-of-type { display: none; }
  .file-size { width: 48px; }
  .file-permissions { display: none; }
  .file-pane { min-width: 120px; }
}
</style>
