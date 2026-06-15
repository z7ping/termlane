<template>
  <div class="h-full flex flex-col" style="background: var(--bg-base)">
    <div class="h-9 border-b flex items-center px-3 gap-2" style="background: var(--bg-surface); border-color: var(--border)">
      <span class="text-sm font-medium" style="color: var(--fg-secondary)">📊 服务器监控</span>
      <div class="flex-1" />
      <label class="flex items-center gap-1 text-xs" style="color: var(--fg-muted)">
        <input type="checkbox" v-model="autoRefresh" class="w-3 h-3" /> 自动刷新 (5s)
      </label>
      <button @click="refreshAll" class="text-xs px-2 py-0.5 rounded" style="background: var(--bg-elevated); color: var(--fg-secondary)">⟳ 刷新</button>
    </div>

    <div class="flex-1 overflow-y-auto p-3">
      <div v-for="s in servers" :key="s.id" class="rounded-lg mb-3 p-4" style="background: var(--bg-surface)">
        <!-- Header -->
        <div class="flex items-center justify-between mb-3">
          <div class="flex items-center gap-2">
            <span class="w-3 h-3 rounded-full" :style="s.loading ? { background: 'var(--warning)', animation: 'pulse 2s infinite' } : s.error ? { background: 'var(--danger)' } : { background: 'var(--success)' }" />
            <span class="text-sm font-medium" style="color: var(--fg-primary)">{{ s.name }}</span>
            <span class="text-xs" style="color: var(--fg-muted)">{{ s.host }}:{{ s.port }}</span>
          </div>
          <span v-if="s.error" class="text-xs" style="color: var(--danger)">{{ s.error }}</span>
          <span v-else-if="s.data" class="text-xs" style="color: var(--fg-muted)">
            负载 {{ s.data.load1.toFixed(2) }} / {{ s.data.load5.toFixed(2) }} / {{ s.data.load15.toFixed(2) }}
          </span>
        </div>

        <!-- Metrics Grid -->
        <div v-if="s.data" class="grid grid-cols-2 lg:grid-cols-4 gap-3">
          <!-- CPU -->
          <div class="rounded p-3" style="background: var(--bg-base)">
            <div class="text-xs mb-1" style="color: var(--fg-muted)">🧠 CPU</div>
            <div class="text-lg font-mono" style="color: var(--fg-primary)">{{ s.data.cpuUsage.toFixed(1) }}%</div>
            <div class="w-full h-1.5 rounded-full mt-2" style="background: var(--bg-elevated)">
              <div class="h-full rounded-full transition-all duration-500"
                :style="{ ...barColor(s.data.cpuUsage), width: Math.min(100, s.data.cpuUsage) + '%' }" />
            </div>
          </div>

          <!-- Memory -->
          <div class="rounded p-3" style="background: var(--bg-base)">
            <div class="text-xs mb-1" style="color: var(--fg-muted)">💾 内存</div>
            <div class="text-lg font-mono" style="color: var(--fg-primary)">{{ s.data.memoryPercent.toFixed(1) }}%</div>
            <div class="text-xs mt-0.5" style="color: var(--fg-muted)">{{ fmtBytes(s.data.memoryUsed) }} / {{ fmtBytes(s.data.memoryTotal) }}</div>
            <div class="w-full h-1.5 rounded-full mt-2" style="background: var(--bg-elevated)">
              <div class="h-full rounded-full transition-all duration-500"
                :style="{ ...barColor(s.data.memoryPercent), width: Math.min(100, s.data.memoryPercent) + '%' }" />
            </div>
          </div>

          <!-- Disk -->
          <div class="rounded p-3" style="background: var(--bg-base)">
            <div class="text-xs mb-1" style="color: var(--fg-muted)">💿 磁盘 (/)</div>
            <div class="text-lg font-mono" style="color: var(--fg-primary)">{{ s.data.diskPercent.toFixed(1) }}%</div>
            <div class="text-xs mt-0.5" style="color: var(--fg-muted)">{{ fmtBytes(s.data.diskUsed) }} / {{ fmtBytes(s.data.diskTotal) }}</div>
            <div class="w-full h-1.5 rounded-full mt-2" style="background: var(--bg-elevated)">
              <div class="h-full rounded-full transition-all duration-500"
                :style="{ ...barColor(s.data.diskPercent), width: Math.min(100, s.data.diskPercent) + '%' }" />
            </div>
          </div>

          <!-- Uptime -->
          <div class="rounded p-3" style="background: var(--bg-base)">
            <div class="text-xs mb-1" style="color: var(--fg-muted)">⏱️ 运行时间</div>
            <div class="text-sm font-mono" style="color: var(--fg-primary)">{{ fmtUptime(s.data.uptimeSeconds) }}</div>
            <div class="text-xs mt-2" style="color: var(--fg-muted)">负载 1/5/15min</div>
            <div class="text-xs font-mono" style="color: var(--fg-secondary)">{{ s.data.load1.toFixed(2) }} / {{ s.data.load5.toFixed(2) }} / {{ s.data.load15.toFixed(2) }}</div>
          </div>
        </div>

        <!-- Charts Row -->
        <div v-if="s.data" class="grid grid-cols-2 lg:grid-cols-4 gap-3 mt-3">
          <div class="rounded p-2" style="background: var(--bg-base)">
            <div class="text-xs mb-1" style="color: var(--fg-muted)">CPU 趋势</div>
            <canvas :ref="el => setChartRef(s.id, 'cpu', el)" height="120"></canvas>
          </div>
          <div class="rounded p-2" style="background: var(--bg-base)">
            <div class="text-xs mb-1" style="color: var(--fg-muted)">内存趋势</div>
            <canvas :ref="el => setChartRef(s.id, 'memory', el)" height="120"></canvas>
          </div>
          <div class="rounded p-2" style="background: var(--bg-base)">
            <div class="text-xs mb-1" style="color: var(--fg-muted)">磁盘趋势</div>
            <canvas :ref="el => setChartRef(s.id, 'disk', el)" height="120"></canvas>
          </div>
          <div class="rounded p-2" style="background: var(--bg-base)">
            <div class="text-xs mb-1" style="color: var(--fg-muted)">负载趋势</div>
            <canvas :ref="el => setChartRef(s.id, 'load', el)" height="120"></canvas>
          </div>
        </div>

        <!-- No data yet -->
        <div v-else-if="!s.error" class="text-center text-sm py-4" style="color: var(--fg-muted)">
          {{ s.loading ? '加载中...' : '点击刷新获取数据' }}
        </div>
      </div>

      <div v-if="servers.length === 0" class="text-center text-sm mt-10" style="color: var(--fg-muted)">
        暂无连接<br/><span class="text-xs">先在左侧添加 SSH 连接</span>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted, watch } from 'vue'
import { invoke } from '../utils/tauri.js'
import { formatBytes } from '../utils/format.js'
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

  h.cpu.push(data.cpuUsage)
  h.memory.push(data.memoryPercent)
  h.disk.push(data.diskPercent)
  h.load.push(data.load1)

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
  if (pct > 85) return { background: 'var(--danger)' }
  if (pct > 60) return { background: 'var(--warning)' }
  return { background: 'var(--success)' }
}

function fmtBytes(b) {
  return formatBytes(b)
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
