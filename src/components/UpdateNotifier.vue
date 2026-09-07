<template>
  <div v-if="showUpdate" class="update-card" role="status" aria-live="polite">
    <div class="update-header">
      <RefreshCw :size="16" :stroke-width="1.8" />
      <div class="min-w-0 flex-1">
        <div class="update-title">发现新版本</div>
        <div class="update-version">v{{ currentVersion }} → v{{ newVersion }}</div>
      </div>
      <button type="button" class="icon-button" aria-label="关闭更新提示" @click="dismiss">
        <X :size="14" :stroke-width="1.8" />
      </button>
    </div>

    <div class="update-body">
      <div v-if="releaseNotes.length" class="release-notes">
        <div v-for="(note, index) in releaseNotes" :key="`${index}-${note}`">{{ note }}</div>
      </div>
      <div class="update-note">
        <Info :size="14" :stroke-width="1.8" />
        <span>当前仅检查新版本。应用内下载安装将在签名更新链路完成后启用。</span>
      </div>
      <div class="update-actions">
        <button type="button" class="secondary-button" @click="skipVersion">跳过此版本</button>
        <button type="button" class="primary-button" @click="dismiss">稍后提醒</button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { onMounted, ref } from 'vue'
import { Info, RefreshCw, X } from 'lucide-vue-next'
import { invoke } from '@tauri-apps/api/core'
import { STORAGE_KEYS } from '@/utils/storage-keys'

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
    if (localStorage.getItem(STORAGE_KEYS.SKIPPED_VERSION) === updateInfo.version) return

    newVersion.value = updateInfo.version
    releaseNotes.value = String(updateInfo.body || '')
      .split('\n')
      .map(line => line.trim())
      .filter(Boolean)
      .slice(0, 20)
    showUpdate.value = true
  } catch (error) {
    console.warn('[Termlane] Update check failed:', error)
  }
}

function dismiss() {
  showUpdate.value = false
}

function skipVersion() {
  localStorage.setItem(STORAGE_KEYS.SKIPPED_VERSION, newVersion.value)
  dismiss()
}

onMounted(() => {
  window.setTimeout(checkForUpdates, 3000)
})
</script>

<style scoped>
.update-card { position: fixed; right: 12px; bottom: 38px; z-index: 55; width: min(330px, calc(100vw - 24px)); overflow: hidden; border: 1px solid var(--border); border-radius: 9px; background: var(--bg-elevated); color: var(--fg-secondary); box-shadow: var(--shadow-lg); }
.update-header { min-height: 42px; display: flex; align-items: center; gap: 8px; padding: 8px 9px 8px 11px; border-bottom: 1px solid var(--border-subtle); color: var(--accent); }
.update-title { color: var(--fg-primary); font-size: 12px; font-weight: 600; }
.update-version { margin-top: 2px; color: var(--fg-muted); font-family: monospace; font-size: 9px; }
.icon-button { width: 26px; height: 26px; display: inline-flex; align-items: center; justify-content: center; border: 0; border-radius: 5px; background: transparent; color: var(--fg-muted); }
.icon-button:hover { background: var(--bg-hover); color: var(--fg-primary); }
.update-body { padding: 10px; }
.release-notes { max-height: 110px; overflow-y: auto; padding: 7px 8px; border-radius: 6px; background: var(--bg-base); color: var(--fg-secondary); font-size: 10px; line-height: 1.55; }
.release-notes > div + div { margin-top: 2px; }
.update-note { display: flex; align-items: flex-start; gap: 7px; margin-top: 8px; color: var(--fg-muted); font-size: 9px; line-height: 1.5; }
.update-note svg { flex-shrink: 0; margin-top: 1px; }
.update-actions { display: grid; grid-template-columns: 1fr 1fr; gap: 6px; margin-top: 10px; }
.secondary-button, .primary-button { height: 29px; border: 0; border-radius: 6px; font-size: 10px; }
.secondary-button { background: var(--bg-hover); color: var(--fg-secondary); }
.primary-button { background: var(--accent); color: white; }
</style>
