<template>
  <div class="latency-view h-full flex flex-col">
    <div class="view-header">
      <Gauge :size="15" :stroke-width="1.8" />
      <span>连接延迟</span>
      <div class="flex-1" />
      <button type="button" class="primary-action" :disabled="testing || servers.length === 0" @click="testAll">
        <LoaderCircle v-if="testing" :size="13" :stroke-width="1.8" class="spin" />
        <Activity v-else :size="13" :stroke-width="1.8" />
        <span>{{ testing ? '测试中' : '测试全部' }}</span>
      </button>
    </div>

    <div class="info-bar">
      <Info :size="13" :stroke-width="1.8" />
      <span>结果是到 SSH 主机端口的 TCP 建连耗时，不代表网络带宽或 SSH 登录耗时。</span>
    </div>

    <div class="flex-1 overflow-y-auto p-3">
      <div v-if="servers.length === 0" class="empty-state">
        <Server :size="28" :stroke-width="1.4" />
        <div>暂无远程连接</div>
        <span>先在左侧添加 SSH 连接。</span>
      </div>

      <div v-for="server in servers" v-else :key="server.id" class="server-card">
        <div class="server-header">
          <span class="status-dot" :class="statusClass(server)" />
          <div class="min-w-0 flex-1">
            <div class="server-name">{{ server.name }}</div>
            <div class="server-target">{{ server.host }}:{{ server.port }}</div>
          </div>
          <button type="button" class="secondary-action" :disabled="server.testing || testing" @click="testOne(server)">
            <LoaderCircle v-if="server.testing" :size="13" :stroke-width="1.8" class="spin" />
            <Activity v-else :size="13" :stroke-width="1.8" />
            <span>测试</span>
          </button>
        </div>

        <div v-if="validResults(server).length" class="latency-bars" aria-label="最近连接耗时">
          <span
            v-for="(result, index) in server.results.slice(-15)"
            :key="index"
            class="latency-bar"
            :class="result == null ? 'failed' : latencyLevel(result)"
            :style="{ height: result == null ? '4px' : `${barHeight(result)}%` }"
            :title="result == null ? '连接失败' : `${result}ms`"
          />
        </div>

        <div v-if="validResults(server).length" class="stats-grid">
          <div><span>最新</span><strong :class="latencyLevel(latest(server))">{{ latest(server) }}ms</strong></div>
          <div><span>平均</span><strong>{{ average(server) }}ms</strong></div>
          <div><span>最低</span><strong class="good">{{ minimum(server) }}ms</strong></div>
          <div><span>最高</span><strong :class="latencyLevel(maximum(server))">{{ maximum(server) }}ms</strong></div>
        </div>

        <div v-if="validResults(server).length" class="rating-row">
          <span>连接质量</span>
          <strong :class="latencyLevel(latest(server))">{{ rating(latest(server)) }}</strong>
          <span v-if="failureCount(server)" class="failure-count">{{ failureCount(server) }} 次失败</span>
        </div>

        <div v-else-if="server.error" class="server-error">
          <CircleAlert :size="14" :stroke-width="1.8" />
          <span>{{ server.error }}</span>
        </div>
        <div v-else-if="!server.testing" class="server-hint">点击“测试”测量 TCP 建连耗时</div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { onMounted, ref } from 'vue'
import { Activity, CircleAlert, Gauge, Info, LoaderCircle, Server } from 'lucide-vue-next'
import { invoke } from '../utils/tauri.js'

const servers = ref([])
const testing = ref(false)

async function loadServers() {
  try {
    const connections = await invoke('load_connections')
    servers.value = (connections || [])
      .filter(connection => connection.host && !['localhost', '127.0.0.1'].includes(connection.host))
      .map(connection => ({
        id: connection.id,
        name: connection.name,
        host: connection.host,
        port: connection.port || 22,
        results: [],
        testing: false,
        error: '',
      }))
  } catch {
    servers.value = []
  }
}

async function testOne(server) {
  if (server.testing) return
  server.testing = true
  server.error = ''
  try {
    const milliseconds = await invoke('tcp_ping', { host: server.host, port: server.port })
    server.results.push(Number(milliseconds))
  } catch (error) {
    server.results.push(null)
    server.error = typeof error === 'string' ? error : '连接失败'
  } finally {
    if (server.results.length > 20) server.results.splice(0, server.results.length - 20)
    server.testing = false
  }
}

async function testAll() {
  if (testing.value || servers.value.length === 0) return
  testing.value = true
  try {
    for (let round = 0; round < 5; round++) {
      await Promise.all(servers.value.map(testOne))
      if (round < 4) await delay(500)
    }
  } finally {
    testing.value = false
  }
}

function validResults(server) {
  return server.results.filter(value => Number.isFinite(value))
}

function latest(server) {
  return validResults(server).at(-1) ?? 0
}

function average(server) {
  const values = validResults(server)
  return values.length ? Math.round(values.reduce((sum, value) => sum + value, 0) / values.length) : 0
}

function minimum(server) {
  const values = validResults(server)
  return values.length ? Math.min(...values) : 0
}

function maximum(server) {
  const values = validResults(server)
  return values.length ? Math.max(...values) : 0
}

function failureCount(server) {
  return server.results.filter(value => value == null).length
}

function latencyLevel(milliseconds) {
  if (milliseconds < 80) return 'good'
  if (milliseconds < 200) return 'medium'
  return 'poor'
}

function statusClass(server) {
  if (server.testing) return 'testing'
  if (server.error && validResults(server).length === 0) return 'poor'
  if (validResults(server).length) return latencyLevel(latest(server))
  return 'idle'
}

function rating(milliseconds) {
  if (milliseconds < 20) return '极佳'
  if (milliseconds < 50) return '优秀'
  if (milliseconds < 100) return '良好'
  if (milliseconds < 200) return '一般'
  return '较慢'
}

function barHeight(milliseconds) {
  return Math.max(8, Math.min(100, (milliseconds / 300) * 100))
}

function delay(milliseconds) {
  return new Promise(resolve => setTimeout(resolve, milliseconds))
}

onMounted(loadServers)
</script>

<style scoped>
.latency-view { background: var(--bg-base); color: var(--fg-secondary); }
.view-header { height: 36px; display: flex; align-items: center; gap: 7px; padding: 0 10px; flex-shrink: 0; border-bottom: 1px solid var(--border-subtle); background: var(--bg-surface); color: var(--fg-secondary); font-size: 12px; font-weight: 500; }
.primary-action, .secondary-action { height: 28px; display: inline-flex; align-items: center; justify-content: center; gap: 5px; padding: 0 9px; border: 0; border-radius: 6px; font-size: 11px; }
.primary-action { background: var(--accent); color: white; }
.secondary-action { background: var(--bg-hover); color: var(--fg-secondary); }
.primary-action:disabled, .secondary-action:disabled { cursor: not-allowed; opacity: 0.4; }
.info-bar { min-height: 30px; display: flex; align-items: center; gap: 7px; padding: 5px 10px; flex-shrink: 0; border-bottom: 1px solid var(--border-subtle); background: var(--bg-surface); color: var(--fg-muted); font-size: 10px; }
.empty-state { min-height: 220px; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 7px; color: var(--fg-muted); font-size: 12px; text-align: center; }
.empty-state span { font-size: 11px; }
.server-card { margin-bottom: 8px; padding: 10px; border: 1px solid var(--border-subtle); border-radius: 8px; background: var(--bg-surface); }
.server-header { display: flex; align-items: center; gap: 8px; }
.status-dot { width: 7px; height: 7px; flex-shrink: 0; border-radius: 999px; background: var(--fg-muted); }
.status-dot.testing { background: var(--warning); animation: pulse 1s ease-in-out infinite; }
.status-dot.good, strong.good { color: var(--success); }
.status-dot.good { background: var(--success); }
.status-dot.medium, strong.medium { color: var(--warning); }
.status-dot.medium { background: var(--warning); }
.status-dot.poor, strong.poor { color: var(--danger); }
.status-dot.poor { background: var(--danger); }
.server-name { overflow: hidden; color: var(--fg-primary); font-size: 12px; font-weight: 500; text-overflow: ellipsis; white-space: nowrap; }
.server-target { margin-top: 2px; color: var(--fg-muted); font-family: monospace; font-size: 9px; }
.latency-bars { height: 52px; display: flex; align-items: flex-end; gap: 2px; margin-top: 10px; }
.latency-bar { flex: 1; min-width: 2px; max-width: 18px; border-radius: 2px 2px 0 0; background: var(--fg-muted); }
.latency-bar.good { background: var(--success); }
.latency-bar.medium { background: var(--warning); }
.latency-bar.poor { background: var(--danger); }
.latency-bar.failed { background: var(--border); opacity: 0.5; }
.stats-grid { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 5px; margin-top: 9px; }
.stats-grid > div { min-width: 0; padding: 6px 7px; border-radius: 6px; background: var(--bg-base); text-align: center; }
.stats-grid span { display: block; color: var(--fg-muted); font-size: 9px; }
.stats-grid strong { display: block; margin-top: 2px; color: var(--fg-secondary); font-family: monospace; font-size: 11px; font-weight: 500; }
.rating-row { display: flex; align-items: center; gap: 7px; margin-top: 8px; color: var(--fg-muted); font-size: 9px; }
.rating-row strong { font-size: 10px; }
.failure-count { margin-left: auto; color: var(--danger); }
.server-error { display: flex; align-items: center; gap: 6px; margin-top: 9px; color: var(--danger); font-size: 10px; }
.server-hint { margin-top: 9px; color: var(--fg-muted); font-size: 10px; }
.spin { animation: spin 0.8s linear infinite; }
@keyframes spin { to { transform: rotate(360deg); } }
@keyframes pulse { 0%, 100% { opacity: 1; } 50% { opacity: 0.35; } }
@media (max-width: 700px) { .stats-grid { grid-template-columns: repeat(2, minmax(0, 1fr)); } .info-bar { align-items: flex-start; } }
</style>
