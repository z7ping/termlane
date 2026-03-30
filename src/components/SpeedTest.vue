<template>
  <div class="h-full flex flex-col bg-gray-900">
    <div class="h-9 bg-gray-800 border-b border-gray-700 flex items-center px-3 gap-2">
      <span class="text-sm font-medium text-gray-300">⚡ 连接速度测试</span>
      <div class="flex-1" />
      <button @click="testAll" :disabled="testing" class="text-xs px-2 py-0.5 rounded" :class="testing ? 'bg-yellow-600 text-white' : 'bg-blue-600 hover:bg-blue-500 text-white'">
        {{ testing ? '测试中...' : '🚀 测试全部' }}
      </button>
    </div>

    <div class="flex-1 overflow-y-auto p-3">
      <div v-for="s in servers" :key="s.id" class="bg-gray-800 rounded-lg mb-3 p-4">
        <div class="flex items-center justify-between mb-3">
          <div class="flex items-center gap-2">
            <span class="w-2 h-2 rounded-full" :class="s.testing ? 'bg-yellow-400 animate-pulse' : s.results.length ? latencyDot(s.results[s.results.length-1]) : 'bg-gray-500'" />
            <span class="text-sm text-gray-200">{{ s.name }}</span>
            <span class="text-xs text-gray-500">{{ s.host }}:{{ s.port }}</span>
          </div>
          <button @click="testOne(s)" :disabled="s.testing" class="text-xs px-2 py-0.5 rounded bg-gray-700 hover:bg-gray-600 text-gray-300 disabled:opacity-50">
            {{ s.testing ? '...' : '测试' }}
          </button>
        </div>

        <!-- Latency Bar Chart -->
        <div v-if="s.results.length" class="flex items-end gap-0.5 h-14 mb-3">
          <div v-for="(r, i) in s.results.slice(-15)" :key="i"
            class="flex-1 rounded-t transition-all duration-300"
            :class="latencyBar(r)"
            :style="{ height: Math.max(4, Math.min(100, (r / 300) * 100)) + '%' }"
            :title="r + 'ms'" />
        </div>

        <!-- Stats -->
        <div v-if="s.results.length" class="grid grid-cols-4 gap-2 text-center text-xs">
          <div class="bg-gray-900 rounded p-2">
            <div class="text-gray-500">最新</div>
            <div class="font-mono" :class="latencyText(s.results[s.results.length-1])">{{ s.results[s.results.length-1] }}ms</div>
          </div>
          <div class="bg-gray-900 rounded p-2">
            <div class="text-gray-500">平均</div>
            <div class="font-mono text-gray-300">{{ avg(s.results) }}ms</div>
          </div>
          <div class="bg-gray-900 rounded p-2">
            <div class="text-gray-500">最低</div>
            <div class="font-mono text-green-400">{{ min(s.results) }}ms</div>
          </div>
          <div class="bg-gray-900 rounded p-2">
            <div class="text-gray-500">最高</div>
            <div class="font-mono text-red-400">{{ max(s.results) }}ms</div>
          </div>
        </div>

        <!-- Rating -->
        <div v-if="s.results.length" class="mt-2 text-xs">
          评级: <span :class="ratingClass(s.results[s.results.length-1])">{{ rating(s.results[s.results.length-1]) }}</span>
        </div>

        <div v-if="!s.results.length && !s.testing" class="text-xs text-gray-600 text-center py-2">点击「测试」开始</div>
        <div v-if="s.error" class="text-xs text-red-400 mt-1">{{ s.error }}</div>
      </div>

      <div v-if="servers.length === 0" class="text-center text-gray-500 text-sm mt-10">
        暂无连接<br/><span class="text-xs">先在左侧添加 SSH 连接</span>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { invoke } from '../utils/tauri.js'

const servers = ref([])
const testing = ref(false)

async function loadServers() {
  try {
    const conns = await invoke('load_connections')
    servers.value = (conns || [])
      .filter(c => c.host && c.host !== 'localhost')
      .map(c => ({
        id: c.id,
        name: c.name,
        host: c.host,
        port: c.port || 22,
        results: [],
        testing: false,
        error: null,
      }))
  } catch { servers.value = [] }
}

async function testOne(s) {
  s.testing = true
  s.error = null
  try {
    const ms = await invoke('tcp_ping', { host: s.host, port: s.port })
    s.results.push(ms)
    if (s.results.length > 20) s.results.shift()
  } catch (e) {
    s.error = typeof e === 'string' ? e : '连接超时'
    s.results.push(9999)
    if (s.results.length > 20) s.results.shift()
  } finally {
    s.testing = false
  }
}

async function testAll() {
  testing.value = true
  for (let i = 0; i < 5; i++) {
    await Promise.all(servers.value.map(s => testOne(s)))
    if (i < 4) await new Promise(r => setTimeout(r, 500))
  }
  testing.value = false
}

function avg(arr) { return arr.length ? Math.round(arr.reduce((a, b) => a + b, 0) / arr.length) : '-' }
function min(arr) { return arr.length ? Math.min(...arr) : '-' }
function max(arr) { return arr.length ? Math.max(...arr) : '-' }

function latencyBar(ms) {
  if (ms < 30) return 'bg-green-500'
  if (ms < 80) return 'bg-green-400'
  if (ms < 150) return 'bg-yellow-500'
  if (ms < 300) return 'bg-orange-500'
  return 'bg-red-500'
}

function latencyDot(ms) {
  if (ms < 80) return 'bg-green-400'
  if (ms < 200) return 'bg-yellow-400'
  return 'bg-red-400'
}

function latencyText(ms) {
  if (ms < 30) return 'text-green-400'
  if (ms < 80) return 'text-green-300'
  if (ms < 150) return 'text-yellow-400'
  if (ms < 300) return 'text-orange-400'
  return 'text-red-400'
}

function rating(ms) {
  if (ms < 20) return '⭐⭐⭐ 极佳'
  if (ms < 50) return '⭐⭐ 优秀'
  if (ms < 100) return '⭐ 良好'
  if (ms < 200) return '一般'
  return '较慢'
}

function ratingClass(ms) {
  if (ms < 50) return 'text-green-400'
  if (ms < 100) return 'text-yellow-400'
  return 'text-red-400'
}

onMounted(loadServers)
</script>
