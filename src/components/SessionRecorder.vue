<template>
  <div class="h-full flex flex-col bg-gray-900">
    <div class="h-9 bg-gray-800 border-b border-gray-700 flex items-center px-3 gap-2">
      <span class="text-sm font-medium text-gray-300">会话录制</span>
      <div class="flex-1" />
      <span v-if="recording" class="flex items-center gap-1 text-xs text-red-400">
        <span class="w-2 h-2 rounded-full bg-red-500 animate-pulse" />
        录制中 {{ formatDuration(recordingDuration) }}
      </span>
      <button
        @click="toggleRecording"
        class="text-xs px-2 py-0.5 rounded"
        :class="recording ? 'bg-red-600 hover:bg-red-500 text-white' : 'bg-blue-600 hover:bg-blue-500 text-white'"
      >{{ recording ? '⏹ 停止' : '⏺ 开始录制' }}</button>
    </div>

    <div class="flex-1 overflow-y-auto p-3">
      <div v-for="rec in recordings" :key="rec.id" class="bg-gray-800 rounded mb-2 p-3 hover:bg-gray-750">
        <div class="flex items-center justify-between">
          <div>
            <div class="text-sm text-gray-200">{{ rec.name }}</div>
            <div class="text-xs text-gray-500 mt-0.5">
              {{ rec.date }} · {{ formatDuration(rec.duration) }} · {{ formatSize(rec.size) }}
            </div>
          </div>
          <div class="flex gap-1">
            <button @click="playRecording(rec)" class="text-xs px-2 py-0.5 rounded bg-green-600/30 text-green-300 hover:bg-green-600/50">▶ 回放</button>
            <button @click="exportRecording(rec)" class="text-xs px-2 py-0.5 rounded bg-gray-700 text-gray-400 hover:bg-gray-600">导出</button>
            <button @click="deleteRecording(rec.id)" class="text-xs px-2 py-0.5 rounded bg-gray-700 text-gray-400 hover:bg-gray-600 hover:text-white">删除</button>
          </div>
        </div>
        <div v-if="rec.tags && rec.tags.length" class="flex gap-1 mt-2">
          <span v-for="tag in rec.tags" :key="tag" class="text-xs px-1.5 py-0.5 bg-gray-700 rounded text-gray-400">{{ tag }}</span>
        </div>
      </div>

      <div v-if="recordings.length === 0" class="text-center text-gray-500 text-sm mt-10">
        暂无录制记录<br/>
        <span class="text-xs">点击右上角 ⏺ 开始录制终端会话</span>
      </div>
    </div>

    <!-- Playback Modal -->
    <div v-if="playingRec" class="fixed inset-0 bg-black/80 flex items-center justify-center z-50" @click.self="playingRec = null">
      <div class="bg-gray-800 rounded-lg w-[700px] border border-gray-600">
        <div class="p-3 border-b border-gray-700 flex items-center justify-between">
          <span class="text-sm text-gray-200">回放: {{ playingRec.name }}</span>
          <button @click="playingRec = null" class="text-gray-400 hover:text-white">✕</button>
        </div>
        <div ref="playbackRef" class="h-80" />
        <div class="p-3 border-t border-gray-700 flex items-center gap-3">
          <button @click="togglePlayback" class="text-sm px-3 py-1 bg-blue-600 rounded text-white">
            {{ playbackPaused ? '▶ 继续' : '⏸ 暂停' }}
          </button>
          <input type="range" v-model.number="playbackProgress" min="0" max="100" class="flex-1" />
          <span class="text-xs text-gray-500">{{ playbackProgress }}%</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onUnmounted } from 'vue'

const recording = ref(false)
const recordingDuration = ref(0)
const recordings = ref([
  {
    id: '1',
    name: '服务器部署 nginx',
    date: '2026-03-30 14:30',
    duration: 342,
    size: 24576,
    tags: ['部署', 'nginx'],
  },
  {
    id: '2',
    name: '数据库备份脚本调试',
    date: '2026-03-29 10:15',
    duration: 180,
    size: 12288,
    tags: ['数据库', '脚本'],
  },
])
const playingRec = ref(null)
const playbackRef = ref(null)
const playbackPaused = ref(false)
const playbackProgress = ref(0)

let recordingTimer = null

function formatDuration(seconds) {
  const m = Math.floor(seconds / 60)
  const s = seconds % 60
  return `${m}:${s.toString().padStart(2, '0')}`
}

function formatSize(bytes) {
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB'
  return (bytes / (1024 * 1024)).toFixed(1) + ' MB'
}

function toggleRecording() {
  if (recording.value) {
    // Stop recording
    clearInterval(recordingTimer)
    recordings.value.unshift({
      id: Date.now().toString(),
      name: `录制 ${new Date().toLocaleTimeString('zh-CN')}`,
      date: new Date().toLocaleString('zh-CN'),
      duration: recordingDuration.value,
      size: recordingDuration.value * 120,
      tags: [],
    })
    recordingDuration.value = 0
    recording.value = false
  } else {
    // Start recording
    recording.value = true
    recordingDuration.value = 0
    recordingTimer = setInterval(() => {
      recordingDuration.value++
    }, 1000)
  }
}

function playRecording(rec) {
  playingRec.value = rec
  playbackProgress.value = 0
  playbackPaused.value = false
}

function togglePlayback() {
  playbackPaused.value = !playbackPaused.value
}

function exportRecording(rec) {
  // In real app, would export to file
  console.log('Export:', rec.name)
}

function deleteRecording(id) {
  recordings.value = recordings.value.filter(r => r.id !== id)
}

onUnmounted(() => {
  if (recordingTimer) clearInterval(recordingTimer)
})
</script>
