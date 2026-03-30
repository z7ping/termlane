<template>
  <div class="h-full flex flex-col bg-gray-900">
    <div class="h-9 bg-gray-800 border-b border-gray-700 flex items-center px-3 gap-2">
      <span class="text-sm font-medium text-gray-300">连接监控</span>
      <div class="flex-1" />
      <button @click="refreshAll" class="text-xs px-2 py-0.5 bg-gray-700 hover:bg-gray-600 rounded text-gray-300">⟳ 刷新</button>
    </div>

    <div class="flex-1 overflow-y-auto p-3">
      <div v-for="server in servers" :key="server.id" class="bg-gray-800 rounded mb-2 p-3">
        <div class="flex items-center justify-between mb-2">
          <div class="flex items-center gap-2">
            <span class="w-2.5 h-2.5 rounded-full" :class="statusColor(server.status)" />
            <span class="text-sm text-gray-200">{{ server.name }}</span>
            <span class="text-xs text-gray-500">{{ server.host }}</span>
          </div>
          <span class="text-xs px-1.5 py-0.5 rounded" :class="statusBadge(server.status)">
            {{ statusLabel(server.status) }}
          </span>
        </div>

        <div v-if="server.status === 'online'" class="grid grid-cols-3 gap-2 text-xs">
          <div class="bg-gray-900 rounded p-2">
            <div class="text-gray-500">延迟</div>
            <div class="text-gray-200 font-mono">{{ server.latency }}ms</div>
          </div>
          <div class="bg-gray-900 rounded p-2">
            <div class="text-gray-500">CPU</div>
            <div class="text-gray-200 font-mono">{{ server.cpu }}%</div>
            <div class="w-full h-1 bg-gray-700 rounded-full mt-1">
              <div class="h-full rounded-full" :class="server.cpu > 80 ? 'bg-red-500' : server.cpu > 50 ? 'bg-yellow-500' : 'bg-green-500'" :style="{ width: server.cpu + '%' }" />
            </div>
          </div>
          <div class="bg-gray-900 rounded p-2">
            <div class="text-gray-500">内存</div>
            <div class="text-gray-200 font-mono">{{ server.mem }}%</div>
            <div class="w-full h-1 bg-gray-700 rounded-full mt-1">
              <div class="h-full rounded-full" :class="server.mem > 80 ? 'bg-red-500' : server.mem > 50 ? 'bg-yellow-500' : 'bg-green-500'" :style="{ width: server.mem + '%' }" />
            </div>
          </div>
        </div>

        <div v-if="server.status === 'online'" class="grid grid-cols-2 gap-2 text-xs mt-2">
          <div class="bg-gray-900 rounded p-2">
            <div class="text-gray-500">磁盘</div>
            <div class="text-gray-200 font-mono">{{ server.diskUsed }} / {{ server.diskTotal }}</div>
          </div>
          <div class="bg-gray-900 rounded p-2">
            <div class="text-gray-500">运行时间</div>
            <div class="text-gray-200 font-mono">{{ server.uptime }}</div>
          </div>
        </div>

        <div v-if="server.status === 'offline'" class="text-xs text-gray-500 mt-1">
          最后在线: {{ server.lastSeen || '未知' }}
        </div>
      </div>

      <div v-if="servers.length === 0" class="text-center text-gray-500 text-sm mt-10">
        暂无监控服务器<br/>
        <span class="text-xs">添加 SSH 连接后自动显示</span>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue'

const servers = ref([
  {
    id: '1',
    name: '生产服务器',
    host: '192.168.1.100',
    status: 'online',
    latency: 12,
    cpu: 35,
    mem: 62,
    diskUsed: '42G',
    diskTotal: '100G',
    uptime: '45天 12小时',
  },
  {
    id: '2',
    name: '测试服务器',
    host: '192.168.1.101',
    status: 'online',
    latency: 8,
    cpu: 15,
    mem: 28,
    diskUsed: '18G',
    diskTotal: '50G',
    uptime: '12天 6小时',
  },
  {
    id: '3',
    name: '备份服务器',
    host: '192.168.1.102',
    status: 'offline',
    lastSeen: '2026-03-29 22:00',
  },
])

function statusColor(status) {
  return {
    online: 'bg-green-400',
    offline: 'bg-red-400',
    unknown: 'bg-gray-400',
  }[status]
}

function statusBadge(status) {
  return {
    online: 'bg-green-600/30 text-green-300',
    offline: 'bg-red-600/30 text-red-300',
    unknown: 'bg-gray-600/30 text-gray-300',
  }[status]
}

function statusLabel(status) {
  return { online: '在线', offline: '离线', unknown: '未知' }[status]
}

function refreshAll() {
  // Simulate refresh
  servers.value.forEach(s => {
    if (s.status === 'online') {
      s.latency = Math.floor(Math.random() * 20) + 5
      s.cpu = Math.floor(Math.random() * 40) + 10
      s.mem = Math.floor(Math.random() * 50) + 20
    }
  })
}
</script>
