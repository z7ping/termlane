<template>
  <div class="recorder-view h-full flex flex-col">
    <div class="view-header">
      <CircleDot :size="15" :stroke-width="1.8" />
      <span>会话录制</span>
      <div class="flex-1" />
      <span v-if="recording" class="recording-state">
        <span class="recording-dot" />
        {{ formatDuration(recDuration) }}
      </span>
      <button
        type="button"
        class="record-button"
        :class="{ active: recording }"
        :disabled="!sessionId && !recording"
        @click="toggleRecording"
      >
        <Square v-if="recording" :size="12" fill="currentColor" />
        <Circle v-else :size="12" fill="currentColor" />
        <span>{{ recording ? '停止' : '录制' }}</span>
      </button>
    </div>

    <div class="flex-1 overflow-y-auto p-3">
      <div v-if="recordings.length === 0" class="empty-state">
        <CircleDot :size="28" :stroke-width="1.4" />
        <div>暂无录制</div>
        <span>{{ sessionId ? '点击右上角开始记录当前终端输出。' : '先打开并连接一个终端会话。' }}</span>
      </div>

      <div v-for="recordingItem in recordings" v-else :key="recordingItem.id" class="recording-card">
        <div class="min-w-0 flex-1">
          <div class="recording-title">{{ recordingItem.name }}</div>
          <div class="recording-meta">
            {{ recordingItem.connectionName || '本地' }} · {{ formatDate(recordingItem.startedAt) }} · {{ formatDuration(recordingItem.durationSecs) }}
          </div>
        </div>
        <div class="recording-actions">
          <button type="button" title="回放" aria-label="回放" @click="playRecording(recordingItem)">
            <Play :size="14" :stroke-width="1.8" />
          </button>
          <button type="button" title="导出" aria-label="导出" @click="exportRecording(recordingItem)">
            <Download :size="14" :stroke-width="1.8" />
          </button>
          <button type="button" class="danger" title="删除" aria-label="删除" @click="requestDeleteRecording(recordingItem.id)">
            <Trash2 :size="14" :stroke-width="1.8" />
          </button>
        </div>
      </div>
    </div>

    <div v-if="playing" class="playback-backdrop" @click.self="closePlayback">
      <div class="playback-dialog" role="dialog" aria-modal="true" :aria-label="`回放 ${playing.name}`">
        <div class="playback-header">
          <Play :size="14" :stroke-width="1.8" />
          <span class="truncate">{{ playing.name }}</span>
          <div class="flex-1" />
          <button type="button" class="icon-button" aria-label="关闭回放" @click="closePlayback">
            <X :size="14" :stroke-width="1.8" />
          </button>
        </div>

        <div ref="playbackRef" class="playback-output">
          <div
            v-for="(line, index) in playbackLines"
            :key="index"
            class="whitespace-pre-wrap break-all"
            v-html="ansiToHtml(line)"
          />
        </div>

        <div class="playback-controls">
          <button type="button" class="playback-button" @click="togglePlayback">
            <Pause v-if="playbackActive" :size="13" fill="currentColor" />
            <Play v-else :size="13" fill="currentColor" />
            <span>{{ playbackActive ? '暂停' : '播放' }}</span>
          </button>
          <input
            v-model.number="playbackIdx"
            type="range"
            class="flex-1"
            min="0"
            :max="playbackData.length"
            @input="renderPlayback"
          />
          <span class="playback-count">{{ playbackIdx }}/{{ playbackData.length }}</span>
        </div>
      </div>
    </div>

    <BaseModal
      :show="deleteRecordingId != null"
      width="360px"
      title="删除录制"
      @close="deleteRecordingId = null"
    >
      <div class="confirm-dialog">
        <div class="dialog-title">删除录制</div>
        <p>此操作会删除录制文件和对应元数据，无法撤销。</p>
        <div class="confirm-actions">
          <button type="button" class="secondary-button" @click="deleteRecordingId = null">取消</button>
          <button type="button" class="danger-button" @click="confirmDeleteRecording">删除</button>
        </div>
      </div>
    </BaseModal>

    <div v-if="toastState.show" class="recorder-toast" :class="toastState.type">{{ toastState.message }}</div>
  </div>
</template>

<script setup>
import { nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import {
  Circle,
  CircleDot,
  Download,
  Pause,
  Play,
  Square,
  Trash2,
  X,
} from 'lucide-vue-next'
import { invoke, listen } from '../utils/tauri.js'
import BaseModal from './BaseModal.vue'

const props = defineProps({
  sessionId: String,
  connectionName: String,
  isLocal: Boolean,
})

const recording = ref(false)
const recDuration = ref(0)
const recordings = ref([])
const playing = ref(null)
const playbackRef = ref(null)
const playbackLines = ref([])
const playbackData = ref([])
const playbackIdx = ref(0)
const playbackActive = ref(false)
const deleteRecordingId = ref(null)
const toastState = ref({ show: false, message: '', type: 'info' })

const recData = []
let recTimer = null
let unlistenOutput = null
let recordingStartedAt = null
let playbackTimer = null
let toastTimer = null

function toast(message, type = 'info', duration = 2800) {
  if (toastTimer) clearTimeout(toastTimer)
  toastState.value = { show: true, message, type }
  toastTimer = setTimeout(() => { toastState.value.show = false }, duration)
}

function formatDuration(seconds) {
  if (seconds == null) return '-'
  const value = Math.max(0, Math.floor(Number(seconds) || 0))
  const minutes = Math.floor(value / 60)
  return `${minutes}:${(value % 60).toString().padStart(2, '0')}`
}

function formatDate(value) {
  if (!value) return '-'
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return String(value)
  return date.toLocaleString('zh-CN', {
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  })
}

async function loadRecordings() {
  try {
    recordings.value = await invoke('list_recordings') || []
  } catch (error) {
    recordings.value = []
    toast(`读取录制列表失败：${error}`, 'error')
  }
}

async function startRecording() {
  if (!props.sessionId || recording.value) return

  recData.length = 0
  recDuration.value = 0
  recordingStartedAt = new Date()
  const startTime = recordingStartedAt.getTime()
  const eventName = `${props.isLocal ? 'local-output' : 'ssh-output'}:${props.sessionId}`

  try {
    unlistenOutput = await listen(eventName, event => {
      if (!recording.value) return
      const elapsed = (Date.now() - startTime) / 1000
      recData.push([Number(elapsed.toFixed(3)), 'o', String(event.payload ?? '')])
    })
  } catch (error) {
    recordingStartedAt = null
    toast(`无法监听当前会话输出：${error}`, 'error')
    return
  }

  recording.value = true
  recTimer = setInterval(() => {
    recDuration.value = Math.floor((Date.now() - startTime) / 1000)
  }, 500)
}

async function stopRecording({ save = true } = {}) {
  if (!recording.value) return

  clearInterval(recTimer)
  recTimer = null
  unlistenOutput?.()
  unlistenOutput = null
  recording.value = false

  if (!save || !recordingStartedAt) {
    recData.length = 0
    recDuration.value = 0
    recordingStartedAt = null
    return
  }

  const id = `rec_${Date.now()}`
  const filename = `${id}.cast`
  const duration = recDuration.value
  const startedAt = recordingStartedAt.toISOString()
  const displayTime = recordingStartedAt.toLocaleString('zh-CN')
  const header = JSON.stringify({
    version: 2,
    width: 80,
    height: 24,
    timestamp: Math.floor(recordingStartedAt.getTime() / 1000),
    duration,
    title: `录制 ${displayTime}`,
    env: { TERM: 'xterm-256color' },
  })
  const content = `${header}\n${recData.map(entry => JSON.stringify(entry)).join('\n')}\n`
  const meta = {
    id,
    name: `录制 ${displayTime}`,
    connectionName: props.connectionName || (props.isLocal ? '本地' : '未知'),
    startedAt,
    durationSecs: duration,
    filePath: filename,
    tags: [],
  }

  try {
    await invoke('save_recording', { filename, content, meta })
    toast('录制已保存', 'success')
    await loadRecordings()
  } catch (error) {
    toast(`保存录制失败：${error}`, 'error', 4500)
  } finally {
    recData.length = 0
    recDuration.value = 0
    recordingStartedAt = null
  }
}

async function toggleRecording() {
  if (recording.value) await stopRecording()
  else await startRecording()
}

async function playRecording(recordingItem) {
  stopPlaybackTimer()
  try {
    const content = await invoke('read_recording_file', { filename: recordingItem.filePath })
    const lines = content.split('\n').filter(Boolean)
    playbackData.value = lines
      .slice(1)
      .map(line => {
        try { return JSON.parse(line) } catch { return null }
      })
      .filter(Boolean)
    playbackIdx.value = 0
    playbackLines.value = []
    playbackActive.value = false
    playing.value = recordingItem
  } catch (error) {
    toast(`读取录制失败：${error}`, 'error')
  }
}

function stopPlaybackTimer() {
  if (playbackTimer) clearTimeout(playbackTimer)
  playbackTimer = null
  playbackActive.value = false
}

function closePlayback() {
  stopPlaybackTimer()
  playing.value = null
  playbackData.value = []
  playbackLines.value = []
  playbackIdx.value = 0
}

function appendPlaybackEntry(entry) {
  if (entry?.[1] !== 'o') return
  playbackLines.value.push(String(entry[2] ?? ''))
  nextTick(() => {
    if (playbackRef.value) playbackRef.value.scrollTop = playbackRef.value.scrollHeight
  })
}

function scheduleNextPlaybackEntry() {
  if (!playbackActive.value) return
  if (playbackIdx.value >= playbackData.value.length) {
    stopPlaybackTimer()
    return
  }

  const currentIndex = playbackIdx.value
  const current = playbackData.value[currentIndex]
  appendPlaybackEntry(current)
  playbackIdx.value = currentIndex + 1

  if (playbackIdx.value >= playbackData.value.length) {
    stopPlaybackTimer()
    return
  }

  const next = playbackData.value[playbackIdx.value]
  const currentTime = Number(current?.[0]) || 0
  const nextTime = Number(next?.[0]) || currentTime
  const delay = Math.max(10, Math.min(5000, Math.round((nextTime - currentTime) * 1000)))
  playbackTimer = setTimeout(scheduleNextPlaybackEntry, delay)
}

function togglePlayback() {
  if (playbackActive.value) {
    stopPlaybackTimer()
    return
  }
  if (playbackIdx.value >= playbackData.value.length) {
    playbackIdx.value = 0
    playbackLines.value = []
  }
  playbackActive.value = true
  scheduleNextPlaybackEntry()
}

function renderPlayback() {
  stopPlaybackTimer()
  playbackLines.value = []
  for (let index = 0; index < playbackIdx.value && index < playbackData.value.length; index++) {
    const entry = playbackData.value[index]
    if (entry?.[1] === 'o') playbackLines.value.push(String(entry[2] ?? ''))
  }
}

async function exportRecording(recordingItem) {
  try {
    const content = await invoke('read_recording_file', { filename: recordingItem.filePath })
    const blob = new Blob([content], { type: 'application/x-asciicast' })
    const url = URL.createObjectURL(blob)
    const anchor = document.createElement('a')
    anchor.href = url
    anchor.download = `${recordingItem.name}.cast`
    anchor.click()
    setTimeout(() => URL.revokeObjectURL(url), 0)
  } catch (error) {
    toast(`导出失败：${error}`, 'error')
  }
}

function requestDeleteRecording(id) {
  deleteRecordingId.value = id
}

async function confirmDeleteRecording() {
  const id = deleteRecordingId.value
  if (!id) return
  deleteRecordingId.value = null
  try {
    await invoke('delete_recording', { id })
    toast('录制已删除', 'success')
    await loadRecordings()
  } catch (error) {
    toast(`删除失败：${error}`, 'error')
  }
}

function ansiToHtml(text) {
  if (!text) return ''
  return String(text)
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;')
    .replace(/\x1b\[1;32m/g, '<span style="color:#0dbc79">')
    .replace(/\x1b\[1;31m/g, '<span style="color:#cd3131">')
    .replace(/\x1b\[1;33m/g, '<span style="color:#e5e510">')
    .replace(/\x1b\[1;36m/g, '<span style="color:#11a8cd">')
    .replace(/\x1b\[0m/g, '</span>')
    .replace(/\r\n/g, '\n')
    .replace(/\r/g, '\n')
}

watch(() => props.sessionId, async (sessionId, previousSessionId) => {
  if (recording.value && sessionId !== previousSessionId) {
    await stopRecording({ save: true })
    toast('会话已切换，上一段录制已自动保存', 'info')
  }
})

onMounted(loadRecordings)

onUnmounted(() => {
  clearInterval(recTimer)
  unlistenOutput?.()
  stopPlaybackTimer()
  if (toastTimer) clearTimeout(toastTimer)
})
</script>

<style scoped>
.recorder-view {
  position: relative;
  background: var(--bg-base);
  color: var(--fg-secondary);
}

.view-header {
  height: 36px;
  display: flex;
  align-items: center;
  gap: 7px;
  padding: 0 10px;
  flex-shrink: 0;
  border-bottom: 1px solid var(--border-subtle);
  background: var(--bg-surface);
  color: var(--fg-secondary);
  font-size: 12px;
  font-weight: 500;
}

.recording-state {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  color: var(--danger);
  font-family: monospace;
  font-size: 11px;
}

.recording-dot {
  width: 7px;
  height: 7px;
  border-radius: 999px;
  background: var(--danger);
  animation: pulse 1.2s ease-in-out infinite;
}

.record-button,
.playback-button {
  height: 28px;
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 0 9px;
  border: 0;
  border-radius: var(--radius-sm);
  background: var(--accent);
  color: white;
  font-size: 11px;
}

.record-button.active {
  background: var(--danger);
}

.empty-state {
  min-height: 220px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 7px;
  color: var(--fg-muted);
  font-size: 12px;
  text-align: center;
}

.empty-state span {
  font-size: 11px;
}

.recording-card {
  min-height: 58px;
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 6px;
  padding: 9px 10px;
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius);
  background: var(--bg-surface);
}

.recording-title {
  overflow: hidden;
  color: var(--fg-primary);
  font-size: 12px;
  font-weight: 500;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.recording-meta {
  margin-top: 3px;
  overflow: hidden;
  color: var(--fg-muted);
  font-size: 10px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.recording-actions {
  display: flex;
  gap: 2px;
}

.recording-actions button,
.icon-button {
  width: 27px;
  height: 27px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 0;
  border-radius: 5px;
  background: transparent;
  color: var(--fg-muted);
}

.recording-actions button:hover,
.icon-button:hover {
  background: var(--bg-hover);
  color: var(--fg-primary);
}

.recording-actions button.danger:hover {
  color: var(--danger);
}

.playback-backdrop {
  position: fixed;
  inset: 0;
  z-index: 60;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
  background: rgba(0, 0, 0, 0.7);
}

.playback-dialog {
  width: min(760px, 100%);
  max-height: min(620px, 86vh);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  background: var(--bg-elevated);
  box-shadow: var(--shadow-lg);
}

.playback-header {
  height: 38px;
  display: flex;
  align-items: center;
  gap: 7px;
  padding: 0 10px;
  border-bottom: 1px solid var(--border-subtle);
  color: var(--fg-primary);
  font-size: 12px;
}

.playback-output {
  height: 380px;
  overflow: auto;
  padding: 12px;
  background: #0b0d10;
  color: #d4d4d4;
  font-family: var(--font-mono);
  font-size: 12px;
  line-height: 1.45;
}

.playback-controls {
  min-height: 42px;
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 7px 10px;
  border-top: 1px solid var(--border-subtle);
}

.playback-button {
  background: var(--bg-hover);
  color: var(--fg-primary);
}

.playback-count {
  min-width: 54px;
  color: var(--fg-muted);
  font-family: var(--font-mono);
  font-size: 10px;
  text-align: right;
}

.confirm-dialog p {
  margin: 8px 0 0;
  color: var(--fg-secondary);
  font-size: 12px;
  line-height: 1.6;
}

.dialog-title {
  color: var(--fg-primary);
  font-size: 14px;
  font-weight: 600;
}

.confirm-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 18px;
}

.secondary-button,
.danger-button {
  height: 30px;
  padding: 0 11px;
  border: 0;
  border-radius: var(--radius-sm);
  font-size: 11px;
}

.secondary-button {
  background: var(--bg-hover);
  color: var(--fg-secondary);
}

.danger-button {
  background: var(--danger);
  color: white;
}

.recorder-toast {
  position: absolute;
  top: 8px;
  right: 8px;
  z-index: 70;
  max-width: min(420px, calc(100% - 16px));
  padding: 7px 9px;
  border-radius: var(--radius-sm);
  box-shadow: var(--shadow);
  font-size: 11px;
}

.recorder-toast.success {
  background: color-mix(in srgb, var(--success) 18%, var(--bg-elevated));
  color: var(--success);
}

.recorder-toast.error {
  background: color-mix(in srgb, var(--danger) 18%, var(--bg-elevated));
  color: var(--danger);
}

.recorder-toast.info {
  background: var(--bg-elevated);
  color: var(--fg-secondary);
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.35; }
}
</style>
