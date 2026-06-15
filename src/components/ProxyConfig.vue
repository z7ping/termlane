<template>
  <div class="h-full flex flex-col bg-gray-900">
    <div class="h-9 bg-gray-800 border-b border-gray-700 flex items-center px-3 gap-2">
      <span class="text-sm font-medium text-gray-300">🌐 代理设置</span>
      <div class="flex-1" />
      <button @click="saveProxy" class="text-xs px-2 py-0.5 bg-blue-600 hover:bg-blue-500 rounded text-white">💾 保存</button>
    </div>

    <div class="flex-1 overflow-y-auto p-4 space-y-4">
      <!-- Enable toggle -->
      <div class="flex items-center justify-between bg-gray-800 rounded p-3">
        <div>
          <div class="text-sm text-gray-200">启用代理</div>
          <div class="text-xs text-gray-500">所有 SSH 连接通过代理</div>
        </div>
        <button @click="config.enabled = !config.enabled" class="w-10 h-5 rounded-full relative transition-colors" :class="config.enabled ? 'bg-blue-600' : 'bg-gray-600'">
          <span class="absolute top-0.5 w-4 h-4 rounded-full bg-white transition-all" :class="config.enabled ? 'left-5' : 'left-0.5'" />
        </button>
      </div>

      <template v-if="config.enabled">
        <!-- Type -->
        <div>
          <label class="text-xs text-gray-400 block mb-2">代理类型</label>
          <div class="flex gap-2">
            <button v-for="t in proxyTypes" :key="t.value" @click="config.type = t.value"
              class="flex-1 px-3 py-2 text-sm rounded border"
              :class="config.type === t.value ? 'bg-blue-600/20 border-blue-500 text-blue-300' : 'border-gray-600 text-gray-400 hover:border-gray-500'">
              {{ t.label }}
            </button>
          </div>
        </div>

        <!-- Host & Port -->
        <div class="flex gap-2">
          <div class="flex-1">
            <label class="text-xs text-gray-400 block mb-1">代理主机</label>
            <input v-model="config.host" class="w-full bg-gray-800 border border-gray-600 rounded px-3 py-1.5 text-sm focus:outline-none focus:border-blue-500" placeholder="127.0.0.1" />
          </div>
          <div class="w-24">
            <label class="text-xs text-gray-400 block mb-1">端口</label>
            <input v-model.number="config.port" type="number" class="w-full bg-gray-800 border border-gray-600 rounded px-3 py-1.5 text-sm focus:outline-none focus:border-blue-500" placeholder="7890" />
          </div>
        </div>

        <!-- Auth -->
        <div class="flex items-center justify-between bg-gray-800 rounded p-3">
          <div class="text-sm text-gray-200">代理需要认证</div>
          <button @click="config.auth = !config.auth" class="w-10 h-5 rounded-full relative transition-colors" :class="config.auth ? 'bg-blue-600' : 'bg-gray-600'">
            <span class="absolute top-0.5 w-4 h-4 rounded-full bg-white transition-all" :class="config.auth ? 'left-5' : 'left-0.5'" />
          </button>
        </div>

        <div v-if="config.auth" class="flex gap-2">
          <div class="flex-1">
            <label class="text-xs text-gray-400 block mb-1">用户名</label>
            <input v-model="config.username" class="w-full bg-gray-800 border border-gray-600 rounded px-3 py-1.5 text-sm focus:outline-none focus:border-blue-500" />
          </div>
          <div class="flex-1">
            <label class="text-xs text-gray-400 block mb-1">密码</label>
            <input v-model="config.password" type="password" class="w-full bg-gray-800 border border-gray-600 rounded px-3 py-1.5 text-sm focus:outline-none focus:border-blue-500" />
          </div>
        </div>

        <!-- Bypass -->
        <div>
          <label class="text-xs text-gray-400 block mb-1">绕过代理（每行一个）</label>
          <textarea v-model="config.bypass" class="w-full bg-gray-800 border border-gray-600 rounded px-3 py-2 text-sm font-mono focus:outline-none focus:border-blue-500 h-20 resize-none" placeholder="localhost&#10;127.0.0.1&#10;192.168.*&#10;*.local" />
        </div>
      </template>
    </div>
  </div>
</template>

<script setup>
import { reactive, onMounted } from 'vue'
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

async function loadProxy() {
  try {
    const saved = await secureStore.get('proxy_config')
    if (saved) Object.assign(config, saved)
  } catch {}
}

async function saveProxy() {
  await secureStore.set('proxy_config', { ...config })
  alert('代理设置已保存')
}

onMounted(loadProxy)
</script>
