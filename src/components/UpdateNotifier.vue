<template>
  <div v-if="showUpdate" class="fixed bottom-4 right-4 bg-gray-800 border border-gray-600 rounded-lg shadow-2xl w-80 z-50">
    <div class="p-3 border-b border-gray-700 flex items-center gap-2">
      <span class="text-lg">🔄</span>
      <div>
        <div class="text-sm text-gray-200">发现新版本</div>
        <div class="text-xs text-gray-500">v{{ currentVersion }} → v{{ newVersion }}</div>
      </div>
      <button @click="showUpdate = false" class="ml-auto text-gray-400 hover:text-white text-xs">✕</button>
    </div>

    <div class="p-3">
      <div class="text-xs text-gray-400 mb-2">更新内容：</div>
      <div class="text-xs text-gray-300 bg-gray-900 rounded p-2 max-h-24 overflow-y-auto">
        <div v-for="note in releaseNotes" :key="note">{{ note }}</div>
      </div>

      <div v-if="downloading" class="mt-3">
        <div class="flex justify-between text-xs text-gray-400 mb-1">
          <span>下载中...</span>
          <span>{{ downloadProgress }}%</span>
        </div>
        <div class="w-full h-2 bg-gray-700 rounded-full overflow-hidden">
          <div class="h-full bg-blue-500 rounded-full transition-all" :style="{ width: downloadProgress + '%' }" />
        </div>
      </div>

      <div class="flex gap-2 mt-3">
        <button @click="skipVersion" class="flex-1 px-3 py-1.5 text-xs text-gray-400 hover:text-white border border-gray-600 rounded">跳过此版本</button>
        <button @click="downloadUpdate" :disabled="downloading" class="flex-1 px-3 py-1.5 text-xs bg-blue-600 hover:bg-blue-500 rounded text-white disabled:opacity-50">
          {{ downloading ? '下载中...' : '立即更新' }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'

const showUpdate = ref(false)
const currentVersion = ref('0.1.0')
const newVersion = ref('')
const releaseNotes = ref([])
const downloading = ref(false)
const downloadProgress = ref(0)

// Check for updates
async function checkForUpdates() {
  try {
    // In production, this would fetch from your update server
    // For now, simulate checking Gitea releases
    const response = await fetch('https://gitea.7ping.site/api/v1/repos/ai-area/xterminal-pro/releases/latest', {
      headers: { 'Accept': 'application/json' },
    }).catch(() => null)

    if (response && response.ok) {
      const release = await response.json()
      const latestVersion = release.tag_name?.replace('v', '') || ''

      if (latestVersion && latestVersion !== currentVersion.value) {
        newVersion.value = latestVersion
        releaseNotes.value = (release.body || '无更新说明').split('\n').filter(l => l.trim())
        showUpdate.value = true
      }
    }
  } catch (err) {
    // Silent fail - don't bother user if update check fails
    console.log('Update check failed:', err)
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
