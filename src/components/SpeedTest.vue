<template>
  <div class="h-full flex flex-col bg-gray-900">
    <div class="h-9 bg-gray-800 border-b border-gray-700 flex items-center px-3 gap-2">
      <span class="text-sm font-medium text-gray-300">连接速度测试</span>
      <div class="flex-1" />
      <button @click="testAll" :disabled="testing" class="text-xs px-2 py-0.5 rounded" :class="testing ? 'bg-yellow-600 text-white' : 'bg-blue-600 hover:bg-blue-500 text-white'">
        {{ testing ? '测试中...' : '🚀 测试全部' }}
      </button>
    </div>

    <div class="flex-1 overflow-y-auto p-3">
      <div v-for="server in servers" :key="server.id" class="bg-gray-800 rounded mb-2 p-3">
        <div class="flex items-center justify-between mb-3">
          <div class="flex items-center gap-2">
            <span>{{ server.icon }}</span>
            <div>
              <div class="text-sm text-gray-200">{{ server.name }}</div>
              <div class="text-xs text-gray-500">{{ server.host }}:{{ server.port }}</div>
            </div>
          </div>
          <button @click="testOne(server)" :disabled="server.testing" class="text-xs px-2 py-0.5 rounded bg-gray-700 hover:bg-gray-600 text-gray-300">
            {{ server.testing ? '...' : '测试' }}
          </button>
        </div>

        <div v-if="server.results.length > 0">
          <!-- Latency Bar Chart -->
          <div class="flex items-end gap-1 h-16 mb-2">
            <div
              v-for="(r, i) in server.results.slice(-10)"
              :key="i"
              class="flex-1 rounded-t transition-all"
              :class="latencyColor(r.latency)"
              :style="{ height: Math.min(100, (r.latency / 500) * 100) + '%' }"
              :title="r.latency + 'ms'"
            />
          </div>

          <!-- Stats -->
          <div class="grid grid-cols-4 gap-2 text-center">
            <div>
              <div class="text-xs text-gray-500">最新</div>
              <div class="text-sm font-mono" :class="latencyTextColor(server.results[server.results.length - 1]?.latency)">{{ server.results[server.results.length - 1]?.latency || '-' }}ms</div>
            </div>
            <div>
              <div class="text-xs text-gray-500">平均</div>
              <div class="text-sm font-mono text-gray-300">{{ avgLatency(server.results) }}ms</div>
            </div>
            <div>
              <div class="text-xs text-gray-500">最低</div>
              <div class="text-sm font-mono text-green-400">{{ minLatency(server.results) }}ms</div>
            </div>
            <div>
              <div class="text-xs text-gray-500">最高</div>
              <div class="text-sm font-mono text-red-400">{{ maxLatency(server.results) }}ms</div>
            </div>
          </div>

          <!-- Rating -->
          <div class="mt-2 text-xs text-gray-500">
            评级: <span :class="ratingClass(server.results[server.results.length - 1]?.latency)">{{ rating(server.results[server.results.length - 1]?.latency) }}</span>
          </div>
        </div>

        <div v-else class="text-xs text-gray-600 text-center py-2">点击「测试」开始</div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue'

const testing = ref(false)

const servers = ref([
  { id: '1', name: '生产服务器', host: '192.168.1.100', port: 22, icon: '🖥️', testing: false, results: [] },
  { id: '2', name: '测试服务器', host: '192.168.1.101', port: 22, icon: '🧪', testing: false, results: [] },
  { id: '3', name: '开发服务器', host: '192.168.1.102', port: 22, icon: '💻', testing: false, results: [] },
  { id: '4', name: '备份服务器', host: '192.168.1.103', port: 22, icon: '💾', testing: false, results: [] },
])

async function testOne(server) {
  server.testing = true
  // Simulate ping test
  const start = Date.now()
  await new Promise(r => setTimeout(r, Math.random() * 200 + 20))
  const latency = Math.floor(Math.random() * 150) + 5
  server.results.push({ latency, timestamp: Date.now() })
  if (server.results.length > 20) server.results.shift()
  server.testing = false
}

async function testAll() {
  testing.value = true
  for (let i = 0; i < 5; i++) {
    await Promise.all(servers.value.map(s => testOne(s)))
    await new Promise(r => setTimeout(r, 300))
  }
  testing.value = false
}

function avgLatency(results) {
  if (!results.length) return '-'
  return Math.round(results.reduce((s, r) => s + r.latency, 0) / results.length)
}

function minLatency(results) {
  if (!results.length) return '-'
  return Math.min(...results.map(r => r.latency))
}

function maxLatency(results) {
  if (!results.length) return '-'
  return Math.max(...results.map(r => r.latency))
}

function latencyColor(ms) {
  if (!ms) return 'bg-gray-600'
  if (ms < 50) return 'bg-green-500'
  if (ms < 100) return 'bg-green-400'
  if (ms < 200) return 'bg-yellow-500'
  return 'bg-red-500'
}

function latencyTextColor(ms) {
  if (!ms) return 'text-gray-400'
  if (ms < 50) return 'text-green-400'
  if (ms < 100) return 'text-green-300'
  if (ms < 200) return 'text-yellow-400'
  return 'text-red-400'
}

function rating(ms) {
  if (!ms) return '未测试'
  if (ms < 30) return '⭐⭐⭐ 极佳'
  if (ms < 80) return '⭐⭐ 优秀'
  if (ms < 150) return '⭐ 良好'
  return '一般'
}

function ratingClass(ms) {
  if (!ms) return 'text-gray-500'
  if (ms < 80) return 'text-green-400'
  if (ms < 150) return 'text-yellow-400'
  return 'text-red-400'
}
</script>
