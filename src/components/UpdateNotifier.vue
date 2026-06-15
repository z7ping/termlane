<template>
  <div v-if="showUpdate" class="fixed bottom-4 right-4 rounded-lg shadow-2xl w-80 z-50" style="background: var(--bg-surface); border-color: var(--border-subtle)">
    <div class="p-3 border-b flex items-center gap-2" style="border-color: var(--border)">
      <span class="text-lg">🔄</span>
      <div>
        <div class="text-sm" style="color: var(--fg-primary)">发现新版本</div>
        <div class="text-xs" style="color: var(--fg-muted)">v{{ currentVersion }} → v{{ newVersion }}</div>
      </div>
      <button @click="showUpdate = false" class="ml-auto text-xs" style="color: var(--fg-muted)">✕</button>
    </div>

    <div class="p-3">
      <div class="text-xs mb-2" style="color: var(--fg-muted)">更新内容：</div>
      <div class="text-xs rounded p-2 max-h-24 overflow-y-auto" style="color: var(--fg-secondary); background: var(--bg-base)">
        <div v-for="note in releaseNotes" :key="note">{{ note }}</div>
      </div>

      <div v-if="downloading" class="mt-3">
        <div class="flex justify-between text-xs mb-1" style="color: var(--fg-muted)">
          <span>下载中...</span>
          <span>{{ downloadProgress }}%</span>
        </div>
        <div class="w-full h-2 rounded-full overflow-hidden" style="background: var(--bg-elevated)">
          <div class="h-full rounded-full transition-all" style="background: var(--accent)" :style="{ width: downloadProgress + '%' }" />
        </div>
      </div>

      <div class="flex gap-2 mt-3">
        <button @click="skipVersion" class="flex-1 px-3 py-1.5 text-xs rounded" style="color: var(--fg-muted); border-color: var(--border-subtle)">跳过此版本</button>
        <button @click="downloadUpdate" :disabled="downloading" class="flex-1 px-3 py-1.5 text-xs rounded disabled:opacity-50" style="background: var(--accent); color: white">
          {{ downloading ? '下载中...' : '立即更新' }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const showUpdate = ref(false)
const currentVersion = ref('0.1.0')
const newVersion = ref('')
const releaseNotes = ref([])
const downloading = ref(false)
const downloadProgress = ref(0)

// Check for updates via Tauri backend (bypasses CORS)
async function checkForUpdates() {
  try {
    // Optional: skip update check in development
    if (import.meta.env.DEV) {
      console.log('Development mode: skipping update check')
      return
    }

    const updateInfo = await invoke('check_update', { currentVersion: currentVersion.value })

    if (updateInfo) {
      // Has new update
      newVersion.value = updateInfo.version
      releaseNotes.value = updateInfo.body.split('\n').filter(l => l.trim())
      showUpdate.value = true
      console.log(`发现新版本: v${updateInfo.version}`)
    } else {
      // No update or no release - silent
      console.log('Already on latest version or no release available')
    }
  } catch (err) {
    // Silent fail - don't bother user if update check fails
    console.warn('Update check failed:', err)
  }
}

async function downloadUpdate() {
  downloading.value = true
  downloadProgress.value = 0

  // Simulate download progress
  const interval = setInterval(() => {
    downloadProgress.value += Math.floor(Math.random() * 15) + 5
    if (downloadProgress.value >= 100) {
      downloadProgress.value = 100
      clearInterval(interval)
      setTimeout(() => {
        showUpdate.value = false
        downloading.value = false
        // In real Tauri app, would call updater API here
        // window.__TAURI__.updater.installUpdate()
      }, 500)
    }
  }, 200)
}

function skipVersion() {
  showUpdate.value = false
  // Save skipped version to localStorage
  localStorage.setItem('skipped_version', newVersion.value)
}

onMounted(() => {
  // Check on startup after 3 seconds
  setTimeout(checkForUpdates, 3000)
})
</script>
