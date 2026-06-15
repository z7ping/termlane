<template>
  <div class="h-full flex flex-col" style="background: var(--bg-base)">
    <div class="h-9 border-b flex items-center px-3 gap-2" style="background: var(--bg-surface); border-color: var(--border)">
      <span class="text-sm font-medium" style="color: var(--fg-secondary)">🌐 代理设置</span>
      <div class="flex-1" />
      <button @click="saveProxy" class="text-xs px-2 py-0.5 rounded" style="background: var(--accent); color: white">💾 保存</button>
    </div>

    <div class="flex-1 overflow-y-auto p-4 space-y-4">
      <!-- Enable toggle -->
      <div class="flex items-center justify-between rounded p-3" style="background: var(--bg-surface)">
        <div>
          <div class="text-sm" style="color: var(--fg-primary)">启用代理</div>
          <div class="text-xs" style="color: var(--fg-muted)">所有 SSH 连接通过代理</div>
        </div>
        <button @click="config.enabled = !config.enabled" class="w-10 h-5 rounded-full relative transition-colors" :style="config.enabled ? { background: 'var(--accent)' } : { background: 'var(--border-subtle)' }">
          <span class="absolute top-0.5 w-4 h-4 rounded-full bg-white transition-all" :class="config.enabled ? 'left-5' : 'left-0.5'" />
        </button>
      </div>

      <template v-if="config.enabled">
        <!-- Type -->
        <div>
          <label class="text-xs block mb-2" style="color: var(--fg-muted)">代理类型</label>
          <div class="flex gap-2">
            <button v-for="t in proxyTypes" :key="t.value" @click="config.type = t.value"
              class="flex-1 px-3 py-2 text-sm rounded border"
              :style="config.type === t.value ? { background: 'color-mix(in srgb, var(--accent) 20%, transparent)', color: 'var(--accent)' } : { color: 'var(--fg-muted)' }" :class="config.type === t.value ? 'border-[var(--accent)]' : 'border-[var(--border-subtle)]'">
              {{ t.label }}
            </button>
          </div>
        </div>

        <!-- Host & Port -->
        <div class="flex gap-2">
          <div class="flex-1">
            <label class="text-xs block mb-1" style="color: var(--fg-muted)">代理主机</label>
            <input v-model="config.host" class="w-full rounded px-3 py-1.5 text-sm focus:outline-none" style="background: var(--bg-surface); border-color: var(--border-subtle)" placeholder="127.0.0.1" />
          </div>
          <div class="w-24">
            <label class="text-xs block mb-1" style="color: var(--fg-muted)">端口</label>
            <input v-model.number="config.port" type="number" class="w-full rounded px-3 py-1.5 text-sm focus:outline-none" style="background: var(--bg-surface); border-color: var(--border-subtle)" placeholder="7890" />
          </div>
        </div>

        <!-- Auth -->
        <div class="flex items-center justify-between rounded p-3" style="background: var(--bg-surface)">
          <div class="text-sm" style="color: var(--fg-primary)">代理需要认证</div>
          <button @click="config.auth = !config.auth" class="w-10 h-5 rounded-full relative transition-colors" :style="config.auth ? { background: 'var(--accent)' } : { background: 'var(--border-subtle)' }">
            <span class="absolute top-0.5 w-4 h-4 rounded-full bg-white transition-all" :class="config.auth ? 'left-5' : 'left-0.5'" />
          </button>
        </div>

        <div v-if="config.auth" class="flex gap-2">
          <div class="flex-1">
            <label class="text-xs block mb-1" style="color: var(--fg-muted)">用户名</label>
            <input v-model="config.username" class="w-full rounded px-3 py-1.5 text-sm focus:outline-none" style="background: var(--bg-surface); border-color: var(--border-subtle)" />
          </div>
          <div class="flex-1">
            <label class="text-xs block mb-1" style="color: var(--fg-muted)">密码</label>
            <input v-model="config.password" type="password" class="w-full rounded px-3 py-1.5 text-sm focus:outline-none" style="background: var(--bg-surface); border-color: var(--border-subtle)" />
          </div>
        </div>

        <!-- Bypass -->
        <div>
          <label class="text-xs block mb-1" style="color: var(--fg-muted)">绕过代理（每行一个）</label>
          <textarea v-model="config.bypass" class="w-full rounded px-3 py-2 text-sm font-mono focus:outline-none h-20 resize-none" style="background: var(--bg-surface); border-color: var(--border-subtle)" placeholder="localhost&#10;127.0.0.1&#10;192.168.*&#10;*.local" />
        </div>
      </template>
    </div>

    <!-- Toast -->
    <div v-if="toastState.show" class="absolute top-2 right-2 px-3 py-1.5 rounded text-xs z-20 transition-opacity" :class="{
      'bg-green-600/90 text-white': toastState.type === 'success',
      'bg-red-600/90 text-white': toastState.type === 'error',
      'bg-gray-700/90 text-gray-200': toastState.type === 'info',
    }">{{ toastState.message }}</div>
  </div>
</template>

<script setup>
import { reactive, ref, onMounted } from 'vue'
import { secureStore } from '@/utils/secure-store-browser.js'

const proxyTypes = [
  { value: 'http', label: 'HTTP' },
  { value: 'socks5', label: 'SOCKS5' },
]

const config = reactive({
  enabled: false,
  type: 'http',
  host: '127.0.0.1',
  port: 7890,
  auth: false,
  username: '',
  password: '',
  bypass: 'localhost\n127.0.0.1',
})

// Toast
const toastState = ref({ show: false, message: '', type: 'info' })
function _toast(message, type = 'info', duration = 2500) {
  toastState.value = { show: true, message, type }
  setTimeout(() => { toastState.value.show = false }, duration)
}

async function loadProxy() {
  try {
    const saved = await secureStore.get('proxy_config')
    if (saved) Object.assign(config, saved)
  } catch {}
}

async function saveProxy() {
  await secureStore.set('proxy_config', { ...config })
  _toast('代理设置已保存', 'success')
}

onMounted(loadProxy)
</script>
