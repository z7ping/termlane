<template>
  <div v-if="showUpdate" class="fixed bottom-4 right-4 rounded-lg shadow-2xl w-80 z-50" style="background: var(--bg-surface); border-color: var(--border-subtle)">
    <div class="p-3 border-b flex items-center gap-2" style="border-color: var(--border)">
      <span class="text-lg">🔄</span>
      <div>
        <div class="text-sm" style="color: var(--fg-primary)">发现新版本</div>
        <div class="text-xs" style="color: var(--fg-muted)">v{{ currentVersion }} → v{{ newVersion }}</div>
      </div>
      <button @click="dismiss" class="ml-auto text-xs" style="color: var(--fg-muted)">✕</button>
    </div>

    <div class="p-3">
      <div class="text-xs mb-2" style="color: var(--fg-muted)">更新内容：</div>
      <div class="text-xs rounded p-2 max-h-24 overflow-y-auto" style="color: var(--fg-secondary); background: var(--bg-base)">
        <div v-for="note in releaseNotes" :key="note">{{ note }}</div>
      </div>

      <div class="mt-3 text-xs leading-5" style="color: var(--fg-muted)">
        当前仅提供版本检查。应用内下载安装将在签名更新链路完成后启用，避免展示无法真正执行的“假更新”。
      </div>

      <div class="flex gap-2 mt-3">
        <button @click="skipVersion" class="flex-1 px-3 py-1.5 text-xs rounded" style="color: var(--fg-muted); border: 1px solid var(--border-subtle)">跳过此版本</button>
        <button @click="dismiss" class="flex-1 px-3 py-1.5 text-xs rounded" style="background: var(--accent); color: white">
          稍后提醒
        </button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const showUpdate = ref(false)
const currentVersion = ref('')
const newVersion = ref('')
const releaseNotes = ref([])

async function checkForUpdates() {
  if (import.meta.env.DEV) return

  try {
    currentVersion.value = await invoke('get_app_version')
    const updateInfo = await invoke('check_update', { currentVersion: currentVersion.value })

    if (!updateInfo) return
    if (localStorage.getItem('skipped_version') === updateInfo.version) return

    newVersion.value = updateInfo.version
    releaseNotes.value = updateInfo.body.split('\n').filter(line => line.trim())
    showUpdate.value = true
  } catch (err) {
    // 更新检查失败不影响主流程，也不打扰用户。
    console.warn('Update check failed:', err)
  }
}

function dismiss() {
  showUpdate.value = false
}

function skipVersion() {
  localStorage.setItem('skipped_version', newVersion.value)
  dismiss()
}

onMounted(() => {
  setTimeout(checkForUpdates, 3000)
})
</script>
