<template>
  <div class="h-full flex flex-col bg-gray-900">
    <div class="h-9 bg-gray-800 border-b border-gray-700 flex items-center px-3 gap-2">
      <span class="text-sm font-medium text-gray-300">📊 服务器监控</span>
      <div class="flex-1" />
      <label class="flex items-center gap-1 text-xs text-gray-400">
        <input type="checkbox" v-model="autoRefresh" class="w-3 h-3" /> 自动刷新 (5s)
      </label>
      <button @click="refreshAll" class="text-xs px-2 py-0.5 bg-gray-700 hover:bg-gray-600 rounded text-gray-300">⟳ 刷新</button>
    </div>

    <div class="flex-1 overflow-y-auto p-3">
      <div v-for="s in servers" :key="s.id" class="bg-gray-800 rounded-lg mb-3 p-4">
        <!-- Header -->
        <div class="flex items-center justify-between mb-3">
          <div class="flex items-center gap-2">
            <span class="w-3 h-3 rounded-full" :class="s.loading ? 'bg-yellow-400 animate-pulse' : s.error ? 'bg-red-400' : 'bg-green-400'" />
            <span class="text-sm font-medium text-gray-200">{{ s.name }}</span>
            <span class="text-xs text-gray-500">{{ s.host }}:{{ s.port }}</span>
          </div>
          <span v-if="s.error" class="text-xs text-red-400">{{ s.error }}</span>
          <span v-else-if="s.data" class="text-xs text-gray-500">
            负载 {{ s.data.load_1.toFixed(2) }} / {{ s.data.load_5.toFixed(2) }} / {{ s.data.load_15.toFixed(2) }}
          </span>
        </div>

        <!-- Metrics Grid -->
        <div v-if="s.data" class="grid grid-cols-2 lg:grid-cols-4 gap-3">
          <!-- CPU -->
          <div class="bg-gray-900 rounded p-3">
            <div class="text-xs text-gray-500 mb-1">🧠 CPU</div>
            <div class="text-lg font-mono text-gray-200">{{ s.data.cpu_usage.toFixed(1) }}%</div>
            <div class="w-full h-1.5 bg-gray-700 rounded-full mt-2">
              <div class="h-full rounded-full transition-all duration-500"
                :class="barColor(s.data.cpu_usage)"
                :style="{ width: Math.min(100, s.data.cpu_usage) + '%' }" />
            </div>
          </div>

          <!-- Memory -->
          <div class="bg-gray-900 rounded p-3">
            <div class="text-xs text-gray-500 mb-1">💾 内存</div>
            <div class="text-lg font-mono text-gray-200">{{ s.data.memory_percent.toFixed(1) }}%</div>
            <div class="text-xs text-gray-500 mt-0.5">{{ fmtBytes(s.data.memory_used) }} / {{ fmtBytes(s.data.memory_total) }}</div>
            <div class="w-full h-1.5 bg-gray-700 rounded-full mt-2">
              <div class="h-full rounded-full transition-all duration-500"
                :class="barColor(s.data.memory_percent)"
                :style="{ width: Math.min(100, s.data.memory_percent) + '%' }" />
            </div>
          </div>

          <!-- Disk -->
          <div class="bg-gray-900 rounded p-3">
            <div class="text-xs text-gray-500 mb-1">💿 磁盘 (/)</div>
            <div class="text-lg font-mono text-gray-200">{{ s.data.disk_percent.toFixed(1) }}%</div>
            <div class="text-xs text-gray-500 mt-0.5">{{ fmtBytes(s.data.disk_used) }} / {{ fmtBytes(s.data.disk_total) }}</div>
            <div class="w-full h-1.5 bg-gray-700 rounded-full mt-2">
              <div class="h-full rounded-full transition-all duration-500"
                :class="barColor(s.data.disk_percent)"
                :style="{ width: Math.min(100, s.data.disk_percent) + '%' }" />
            </div>
          </div>

          <!-- Uptime -->
          <div class="bg-gray-900 rounded p-3">
            <div class="text-xs text-gray-500 mb-1">⏱️ 运行时间</div>
            <div class="text-sm font-mono text-gray-200">{{ fmtUptime(s.data.uptime_seconds) }}</div>
            <div class="text-xs text-gray-500 mt-2">负载 1/5/15min</div>
            <div class="text-xs font-mono text-gray-300">{{ s.data.load_1.toFixed(2) }} / {{ s.data.load_5.toFixed(2) }} / {{ s.data.load_15.toFixed(2) }}</div>
          </div>
        </div>

        <!-- Charts Row -->
        <div v-if="s.data" class="grid grid-cols-2 lg:grid-cols-4 gap-3 mt-3">
          <div class="bg-gray-900 rounded p-2">
            <div class="text-xs text-gray-500 mb-1">CPU 趋势</div>
            <canvas :ref="el => setChartRef(s.id, 'cpu', el)" height="120"></canvas>
          </div>
          <div class="bg-gray-900 rounded p-2">
            <div class="text-xs text-gray-500 mb-1">内存趋势</div>
            <canvas :ref="el => setChartRef(s.id, 'memory', el)" height="120"></canvas>
          </div>
          <div class="bg-gray-900 rounded p-2">
            <div class="text-xs text-gray-500 mb-1">磁盘趋势</div>
            <canvas :ref="el => setChartRef(s.id, 'disk', el)" height="120"></canvas>
          </div>
          <div class="bg-gray-900 rounded p-2">
            <div class="text-xs text-gray-500 mb-1">负载趋势</div>
            <canvas :ref="el => setChartRef(s.id, 'load', el)" height="120"></canvas>
          </div>
        </div>

        <!-- No data yet -->
        <div v-else-if="!s.error" class="text-center text-gray-500 text-sm py-4">
          {{ s.loading ? '加载中...' : '点击刷新获取数据' }}
        </div>
      </div>

      <div v-if="servers.length === 0" class="text-center text-gray-500 text-sm mt-10">
        暂无连接<br/><span class="text-xs">先在左侧添加 SSH 连接</span>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted, watch } from 'vue'
import { invoke } from '../utils/tauri.js'
import { Chart, LineController, LineElement, PointElement, LinearScale, CategoryScale, Filler } from 'chart.js'

Chart.register(LineController, LineElement, PointElement, LinearScale, CategoryScale, Filler)

const MAX_HISTORY = 20

const props = defineProps({ connections: Array, activeSessionId: String })

const servers = ref([])
const autoRefresh = ref(false)
let refreshTimer = null

// History data: { [serverId]: { cpu: [], memory: [], disk: [], load: [] } }
const history = {}
// Chart instances: { [serverId]: { cpu: Chart, memory: Chart, disk: Chart, load: Chart } }
const chartInstances = {}
// Canvas refs: { [serverId_key]: HTMLElement }
const chartRefs = {}

const chartConfigs = {
  cpu:    { label: 'CPU %',  color: '#22c55e', max: 100 },
  memory: { label: '内存 %', color: '#3b82f6', max: 100 },
  disk:   { label: '磁盘 %', color: '#eab308', max: 100 },
  load:   { label: '负载',   color: '#a855f7', max: null },
}

function setChartRef(serverId, key, el) {
  if (!el) return
  const refKey = `${serverId}_${key}`
  chartRefs[refKey] = el
  // If we already have data, try to create chart
  initChartIfReady(serverId, key)
}

function initChartIfReady(serverId, key) {
  const refKey = `${serverId}_${key}`
  const canvas = chartRefs[refKey]
  if (!canvas) return
  if (chartInstances[serverId]?.[key]) return // already created

  const cfg = chartConfigs[key]
  const data = history[serverId]?.[key] || []

  const labels = data.map((_, i) => i + 1)

  const chart = new Chart(canvas, {
    type: 'line',
    data: {
      labels,
      datasets: [{
        data: [...data],
        borderColor: cfg.color,
        backgroundColor: cfg.color + '20',
        fill: true,
        tension: 0.4,
        pointRadius: 0,
        borderWidth: 1.5,
      }]
    },
    options: {
      responsive: true,
      maintainAspectRatio: false,
      animation: { duration: 300 },
      plugins: {
        legend: { display: false },
        tooltip: { enabled: true },
      },
      scales: {
        x: {
          display: false,
        },
        y: {
          display: true,
          min: 0,
          max: cfg.max || undefined,
          ticks: { color: '#9ca3af', font: { size: 9 }, maxTicksLimit: 5 },
          grid: { color: '#374151' },
          border: { display: false },
        }
      }
    }
  })

  if (!chartInstances[serverId]) chartInstances[serverId] = {}
  chartInstances[serverId][key] = chart
}

function updateChartData(serverId, key, newData) {
  const chart = chartInstances[serverId]?.[key]
  if (!chart) return
  const data = history[serverId][key]
  chart.data.labels = data.map((_, i) => i + 1)
  chart.data.datasets[0].data = [...data]
  chart.update('none')
}

function pushHistory(serverId, data) {
  if (!history[serverId]) {
    history[serverId] = { cpu: [], memory: [], disk: [], load: [] }
  }
  const h = history[serverId]

  h.cpu.push(data.cpu_usage)
  h.memory.push(data.memory_percent)
  h.disk.push(data.disk_percent)
  h.load.push(data.load_1)

  // Keep only last MAX_HISTORY
  for (const key of ['cpu', 'memory', 'disk', 'load']) {
    if (h[key].length > MAX_HISTORY) h[key].shift()
  }

  // Update charts
  for (const key of ['cpu', 'memory', 'disk', 'load']) {
    initChartIfReady(serverId, key)
    updateChartData(serverId, key)
  }
}

function destroyCharts(serverId) {
  if (!chartInstances[serverId]) return
  for (const key of Object.keys(chartInstances[serverId])) {
    chartInstances[serverId][key]?.destroy()
  }
  delete chartInstances[serverId]
}

function barColor(pct) {
  if (pct > 85) return 'bg-red-500'
  if (pct > 60) return 'bg-yellow-500'
  return 'bg-green-500'
}

function fmtBytes(b) {
  if (!b) return '0 B'
  const k = 1024, s = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(b) / Math.log(k))
  return (b / Math.pow(k, i)).toFixed(1) + ' ' + s[i]
}

function fmtUptime(secs) {
  if (!secs) return '-'
  const d = Math.floor(secs / 86400)
  const h = Math.floor((secs % 86400) / 3600)
  const m = Math.floor((secs % 3600) / 60)
  const parts = []
  if (d > 0) parts.push(d + '天')
  if (h > 0) parts.push(h + '小时')
  parts.push(m + '分钟')
  return parts.join(' ')
}

async function loadServers() {
  try {
    const conns = await invoke('load_connections')
    // Destroy old charts before re-creating
    for (const s of servers.value) destroyCharts(s.id)
    servers.value = (conns || []).map(c => ({
      id: c.id,
      name: c.name,
      host: c.host,
      port: c.port,
      data: null,
      loading: false,
      error: null,
    }))
  } catch {
    servers.value = []
  }
}

async function refreshAll() {
  for (const s of servers.value) {
    s.loading = true
    s.error = null
    try {
      if (props.activeSessionId) {
        s.data = await invoke('ssh_monitor', { sessionId: props.activeSessionId })
        // Push to history
        pushHistory(s.id, s.data)
      } else {
        s.error = '未连接'
      }
    } catch (e) {
      s.error = typeof e === 'string' ? e : '获取失败'
    } finally {
      s.loading = false
    }
  }
}

watch(autoRefresh, (on) => {
  if (on) {
    refreshTimer = setInterval(refreshAll, 5000)
    refreshAll()
  } else {
    clearInterval(refreshTimer)
  }
})

onMounted(() => loadServers())
onUnmounted(() => {
  clearInterval(refreshTimer)
  for (const s of servers.value) destroyCharts(s.id)
})
</script>
