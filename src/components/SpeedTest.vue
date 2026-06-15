<template>
  <div class="h-full flex flex-col" style="background: var(--bg-base)">
    <div class="h-9 border-b flex items-center px-3 gap-2" style="background: var(--bg-surface); border-color: var(--border)">
      <span class="text-sm font-medium" style="color: var(--fg-secondary)">⚡ 连接速度测试</span>
      <div class="flex-1" />
      <button @click="testAll" :disabled="testing" class="text-xs px-2 py-0.5 rounded" :style="testing ? { background: 'var(--warning)', color: 'white' } : { background: 'var(--accent)', color: 'white' }">
        {{ testing ? '测试中...' : '🚀 测试全部' }}
      </button>
    </div>

    <div class="flex-1 overflow-y-auto p-3">
      <div v-for="s in servers" :key="s.id" class="rounded-lg mb-3 p-4" style="background: var(--bg-surface)">
        <div class="flex items-center justify-between mb-3">
          <div class="flex items-center gap-2">
            <span class="w-2 h-2 rounded-full" :style="s.testing ? { background: 'var(--warning)', animation: 'pulse 2s infinite' } : s.results.length ? latencyDot(s.results[s.results.length-1]) : { background: 'var(--fg-muted)' }" />
            <span class="text-sm" style="color: var(--fg-primary)">{{ s.name }}</span>
            <span class="text-xs" style="color: var(--fg-muted)">{{ s.host }}:{{ s.port }}</span>
          </div>
          <button @click="testOne(s)" :disabled="s.testing" class="text-xs px-2 py-0.5 rounded disabled:opacity-50" style="background: var(--bg-elevated); color: var(--fg-secondary)">
            {{ s.testing ? '...' : '测试' }}
          </button>
        </div>

        <!-- Latency Bar Chart -->
        <div v-if="s.results.length" class="flex items-end gap-0.5 h-14 mb-3">
          <div v-for="(r, i) in s.results.slice(-15)" :key="i"
            class="flex-1 rounded-t transition-all duration-300"
            :style="latencyBar(r)"
            :style="{ height: Math.max(4, Math.min(100, (r / 300) * 100)) + '%' }"
            :title="r + 'ms'" />
        </div>

        <!-- Stats -->
        <div v-if="s.results.length" class="grid grid-cols-4 gap-2 text-center text-xs">
          <div class="rounded p-2" style="background: var(--bg-base)">
            <div class="" style="color: var(--fg-muted)">最新</div>
            <div class="font-mono" :style="latencyText(s.results[s.results.length-1])">{{ s.results[s.results.length-1] }}ms</div>
          </div>
          <div class="rounded p-2" style="background: var(--bg-base)">
            <div class="" style="color: var(--fg-muted)">平均</div>
            <div class="font-mono" style="color: var(--fg-secondary)">{{ avg(s.results) }}ms</div>
          </div>
          <div class="rounded p-2" style="background: var(--bg-base)">
            <div class="" style="color: var(--fg-muted)">最低</div>
            <div class="font-mono" style="color: var(--success)">{{ min(s.results) }}ms</div>
          </div>
          <div class="rounded p-2" style="background: var(--bg-base)">
            <div class="" style="color: var(--fg-muted)">最高</div>
            <div class="font-mono" style="color: var(--danger)">{{ max(s.results) }}ms</div>
          </div>
        </div>

        <!-- Rating -->
        <div v-if="s.results.length" class="mt-2 text-xs">
          评级: <span :style="ratingClass(s.results[s.results.length-1])">{{ rating(s.results[s.results.length-1]) }}</span>
        </div>

        <div v-if="!s.results.length && !s.testing" class="text-xs text-center py-2" style="color: var(--fg-muted)">点击「测试」开始</div>
        <div v-if="s.error" class="text-xs mt-1" style="color: var(--danger)">{{ s.error }}</div>
      </div>

      <div v-if="servers.length === 0" class="text-center text-sm mt-10" style="color: var(--fg-muted)">
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
  if (ms < 30) return { background: 'var(--success)' }
  if (ms < 80) return { background: 'var(--success)' }
  if (ms < 150) return { background: 'var(--warning)' }
  if (ms < 300) return { background: 'var(--warning)' }
  return { background: 'var(--danger)' }
}

function latencyDot(ms) {
  if (ms < 80) return { background: 'var(--success)' }
  if (ms < 200) return { background: 'var(--warning)' }
  return { background: 'var(--danger)' }
}

function latencyText(ms) {
  if (ms < 30) return { color: 'var(--success)' }
  if (ms < 80) return { color: 'var(--success)' }
  if (ms < 150) return { color: 'var(--warning)' }
  if (ms < 300) return { color: 'var(--warning)' }
  return { color: 'var(--danger)' }
}

function rating(ms) {
  if (ms < 20) return '⭐⭐⭐ 极佳'
  if (ms < 50) return '⭐⭐ 优秀'
  if (ms < 100) return '⭐ 良好'
  if (ms < 200) return '一般'
  return '较慢'
}

function ratingClass(ms) {
  if (ms < 50) return { color: 'var(--success)' }
  if (ms < 100) return { color: 'var(--warning)' }
  return { color: 'var(--danger)' }
}

onMounted(loadServers)
</script>
