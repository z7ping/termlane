<template>
  <div class="h-full flex flex-col" style="background: var(--bg-base)">
    <div class="h-9 border-b flex items-center px-3 gap-2" style="background: var(--bg-surface); border-color: var(--border)">
      <span class="text-sm font-medium" style="color: var(--fg-secondary)">⏺ 会话录制</span>
      <div class="flex-1" />
      <span v-if="recording" class="flex items-center gap-1 text-xs" style="color: var(--danger)">
        <span class="w-2 h-2 rounded-full animate-pulse" style="background: var(--danger)" />
        {{ formatDuration(recDuration) }}
      </span>
      <button @click="toggleRecording" :disabled="!sessionId && !recording" class="text-xs px-2 py-0.5 rounded"
        :style="recording ? { background: 'var(--danger)', color: 'white' } : { background: 'var(--accent)', color: 'white' }">
        {{ recording ? '⏹ 停止' : '⏺ 录制' }}
      </button>
    </div>

    <div class="flex-1 overflow-y-auto p-3">
      <div v-for="rec in recordings" :key="rec.id" class="rounded-lg mb-2 p-3" style="background: var(--bg-surface)">
        <div class="flex items-center justify-between">
          <div>
            <div class="text-sm" style="color: var(--fg-primary)">{{ rec.name }}</div>
            <div class="text-xs mt-0.5" style="color: var(--fg-muted)">
              {{ rec.connectionName }} · {{ rec.startedAt }} · {{ formatDuration(rec.durationSecs) }}
            </div>
          </div>
          <div class="flex gap-1">
            <button @click="playRecording(rec)" class="text-xs px-2 py-0.5 rounded" style="background: color-mix(in srgb, var(--success) 30%, transparent); color: var(--success)">▶</button>
            <button @click="exportRecording(rec)" class="text-xs px-2 py-0.5 rounded" style="background: color-mix(in srgb, var(--accent) 30%, transparent); color: var(--accent)">📤</button>
            <button @click="delRecording(rec.id)" class="text-xs px-2 py-0.5 rounded" style="background: var(--bg-elevated); color: var(--fg-muted)">🗑</button>
          </div>
        </div>
        <div v-if="rec.tags?.length" class="flex gap-1 mt-2">
          <span v-for="tag in rec.tags" :key="tag" class="text-xs px-1.5 py-0.5 rounded" style="background: var(--bg-elevated); color: var(--fg-muted)">{{ tag }}</span>
        </div>
      </div>
      <div v-if="recordings.length === 0" class="text-center text-sm mt-10" style="color: var(--fg-muted)">
        暂无录制<br/><span class="text-xs">连接服务器后点击 ⏺ 开始录制</span>
      </div>
    </div>

    <!-- Playback Modal -->
    <div v-if="playing" class="fixed inset-0 flex items-center justify-center z-50" style="background: color-mix(in srgb, #000 80%, transparent)" @click.self="playing = null">
      <div class="rounded-lg w-[720px] flex flex-col max-h-[80vh]" style="background: var(--bg-surface); border-color: var(--border-subtle)">
        <div class="p-3 border-b flex items-center justify-between" style="border-color: var(--border)">
          <span class="text-sm" style="color: var(--fg-primary)">▶ {{ playing.name }}</span>
          <button @click="playing = null" class="" style="color: var(--fg-muted)">✕</button>
        </div>
        <div ref="playbackRef" class="h-80 bg-black overflow-auto font-mono text-sm p-3" style="color: #d4d4d4;">
          <div v-for="(line, i) in playbackLines" :key="i" class="whitespace-pre-wrap break-all" v-html="ansiToHtml(line)" @click.prevent @keydown.prevent />
        </div>
        <div class="p-3 border-t flex items-center gap-3" style="border-color: var(--border)">
          <button @click="togglePlayback" class="text-sm px-3 py-1 rounded text-white" :style="playbackActive ? { background: 'var(--danger)' } : { background: 'var(--accent)' }">
            {{ playbackActive ? '⏸ 暂停' : '▶ 播放' }}
          </button>
          <input type="range" class="flex-1" min="0" :max="playbackData.length" v-model.number="playbackIdx" @input="renderPlayback" />
          <span class="text-xs" style="color: var(--fg-muted)">{{ playbackIdx }}/{{ playbackData.length }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted, nextTick } from 'vue'
import { invoke, listen } from '../utils/tauri.js'
import DOMPurify from 'dompurify'

const props = defineProps({ sessionId: String, connectionName: String })

const recording = ref(false)
const recDuration = ref(0)
const recData = []
const recordings = ref([])
const playing = ref(null)
const playbackRef = ref(null)
const playbackLines = ref([])
const playbackData = ref([])
const playbackIdx = ref(0)
const playbackActive = ref(false)
let recTimer = null
let unlistenOutput = null

function formatDuration(s) {
  if (!s && s !== 0) return '-'
  const m = Math.floor(s / 60)
  return `${m}:${(s % 60).toString().padStart(2, '0')}`
}

async function loadRecordings() {
  try { recordings.value = await invoke('list_recordings') || [] }
  catch { recordings.value = [] }
}

async function toggleRecording() {
  if (recording.value) {
    // Stop recording
    clearInterval(recTimer)
    unlistenOutput?.()
    unlistenOutput = null
    recording.value = false

    // Save as asciinema v2
    const id = `rec_${Date.now()}`
    const filename = `${id}.cast`
    const header = JSON.stringify({
      version: 2, width: 80, height: 24,
      timestamp: Math.floor(Date.now() / 1000),
      duration: recDuration.value,
      title: `录制 ${new Date().toLocaleString('zh-CN')}`,
      env: { SHELL: '/bin/bash', TERM: 'xterm-256color' }
    })
    const body = recData.map(e => JSON.stringify(e)).join('\n')
    const content = header + '\n' + body + '\n'

    try {
      await invoke('save_recording_file', { filename, content })
      await invoke('save_recording_meta', {
        meta: {
          id, name: `录制 ${new Date().toLocaleString('zh-CN')}`,
          connectionName: props.connectionName || '未知',
          startedAt: new Date().toISOString(),
          durationSecs: recDuration.value,
          filePath: filename,
          tags: [],
        }
      })
    } catch { /* save failed silently */ }

    recData.length = 0
    recDuration.value = 0
    await loadRecordings()
  } else {
    // Start recording
    if (!props.sessionId) return
    recData.length = 0
    recDuration.value = 0
    recording.value = true

    const startTime = Date.now()

    // Listen for terminal output
    unlistenOutput = await listen(`ssh-output:${props.sessionId}`, (event) => {
      if (recording.value) {
        const elapsed = ((Date.now() - startTime) / 1000).toFixed(3)
        recData.push([parseFloat(elapsed), 'o', event.payload])
      }
    })

    recTimer = setInterval(() => { recDuration.value++ }, 1000)
  }
}

async function playRecording(rec) {
  try {
    const content = await invoke('read_recording_file', { filename: rec.filePath })
    const lines = content.split('\n').filter(Boolean)
    // Skip header line
    const dataLines = lines.slice(1)
    playbackData.value = dataLines.map(l => { try { return JSON.parse(l) } catch { return null } }).filter(Boolean)
    playbackIdx.value = 0
    playbackLines.value = []
    playing.value = rec
  } catch (e) { alert('读取录制失败: ' + e) }
}

let playbackTimer = null
function togglePlayback() {
  if (playbackActive.value) {
    clearInterval(playbackTimer)
    playbackActive.value = false
  } else {
    playbackActive.value = true
    playbackTimer = setInterval(() => {
      if (playbackIdx.value >= playbackData.value.length) {
        clearInterval(playbackTimer)
        playbackActive.value = false
        return
      }
      const entry = playbackData.value[playbackIdx.value]
      if (entry && entry[1] === 'o') {
        playbackLines.value.push(entry[2])
        nextTick(() => { if (playbackRef.value) playbackRef.value.scrollTop = playbackRef.value.scrollHeight })
      }
      playbackIdx.value++
    }, 50)
  }
}

function renderPlayback() {
  playbackLines.value = []
  for (let i = 0; i < playbackIdx.value && i < playbackData.value.length; i++) {
    const entry = playbackData.value[i]
    if (entry && entry[1] === 'o') playbackLines.value.push(entry[2])
  }
}

async function exportRecording(rec) {
  try {
    const content = await invoke('read_recording_file', { filename: rec.filePath })
    const blob = new Blob([content], { type: 'application/x-asciicast' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url; a.download = `${rec.name}.cast`; a.click()
    URL.revokeObjectURL(url)
  } catch (e) { alert('导出失败: ' + e) }
}

async function delRecording(id) {
  if (!confirm('确认删除此录制？')) return
  try { await invoke('delete_recording', { id }); await loadRecordings() }
  catch (e) { alert('删除失败: ' + e) }
}

function ansiToHtml(text) {
  if (!text) return ''
  return text
    .replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;').replace(/'/g, '&#39;')
    .replace(/\x1b\[1;32m/g, '<span style="color:#0dbc79">')
    .replace(/\x1b\[1;31m/g, '<span style="color:#cd3131">')
    .replace(/\x1b\[1;33m/g, '<span style="color:#e5e510">')
    .replace(/\x1b\[1;36m/g, '<span style="color:#11a8cd">')
    .replace(/\x1b\[0m/g, '</span>')
    .replace(/\r\n/g, '\n').replace(/\r/g, '\n')
}

onMounted(loadRecordings)
onUnmounted(() => { clearInterval(recTimer); unlistenOutput?.(); clearInterval(playbackTimer) })
</script>
