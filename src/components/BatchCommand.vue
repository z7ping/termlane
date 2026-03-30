<template>
  <div class="h-full flex flex-col bg-gray-900">
    <div class="h-9 bg-gray-800 border-b border-gray-700 flex items-center px-3 gap-2">
      <span class="text-sm font-medium text-gray-300">批量命令</span>
      <div class="flex-1" />
      <span class="text-xs text-gray-500">{{ selectedServers.length }} 台已选</span>
    </div>

    <div class="flex-1 flex overflow-hidden">
      <!-- 服务器选择 -->
      <div class="w-48 border-r border-gray-700 flex flex-col">
        <div class="p-2 border-b border-gray-700 flex items-center justify-between">
          <span class="text-xs text-gray-500">选择服务器</span>
          <button @click="selectAll" class="text-xs text-blue-400 hover:text-blue-300">全选</button>
        </div>
        <div class="flex-1 overflow-y-auto p-1">
          <div
            v-for="server in servers"
            :key="server.id"
            @click="toggleServer(server.id)"
            class="flex items-center gap-2 px-2 py-1.5 rounded cursor-pointer text-sm"
            :class="selectedServers.includes(server.id) ? 'bg-blue-600/20 text-blue-300' : 'text-gray-300 hover:bg-gray-700'"
          >
            <span class="w-2 h-2 rounded-full" :class="selectedServers.includes(server.id) ? 'bg-blue-400' : 'bg-gray-600'" />
            <span class="truncate">{{ server.name }}</span>
          </div>
        </div>
      </div>

      <!-- 命令输入 + 输出 -->
      <div class="flex-1 flex flex-col">
        <!-- 命令输入 -->
        <div class="p-2 border-b border-gray-700 flex gap-2">
          <input
            v-model="command"
            @keydown.enter="executeBatch"
            class="flex-1 bg-gray-900 text-sm text-gray-200 px-3 py-1.5 rounded border border-gray-600 focus:outline-none focus:border-blue-500 font-mono"
            placeholder="输入命令，按 Enter 执行到所有选中服务器..."
            :disabled="running"
          />
          <button
            @click="executeBatch"
            :disabled="running || !command || selectedServers.length === 0"
            class="px-3 py-1.5 text-sm rounded"
            :class="running ? 'bg-yellow-600 text-white' : 'bg-blue-600 hover:bg-blue-500 text-white disabled:opacity-50'"
          >{{ running ? '执行中...' : '执行' }}</button>
        </div>

        <!-- 输出区域 -->
        <div class="flex-1 overflow-y-auto p-2 space-y-2">
          <div v-for="result in results" :key="result.serverId" class="bg-gray-800 rounded overflow-hidden">
            <div class="flex items-center justify-between px-3 py-1.5 border-b border-gray-700">
              <div class="flex items-center gap-2">
                <span class="w-2 h-2 rounded-full" :class="result.status === 'success' ? 'bg-green-400' : result.status === 'error' ? 'bg-red-400' : 'bg-yellow-400 animate-pulse'" />
                <span class="text-sm text-gray-300">{{ result.serverName }}</span>
              </div>
              <span class="text-xs text-gray-500">{{ result.duration }}ms</span>
            </div>
            <pre class="p-3 text-xs text-gray-300 font-mono overflow-x-auto whitespace-pre-wrap max-h-40">{{ result.output || '(无输出)' }}</pre>
          </div>

          <div v-if="results.length === 0" class="text-center text-gray-500 text-sm mt-10">
            选择服务器 → 输入命令 → 执行
          </div>
        </div>
      </div>
    </div>

    <!-- 历史记录 -->
    <div v-if="history.length > 0" class="h-16 bg-gray-850 border-t border-gray-700 overflow-x-auto flex items-center gap-2 px-2" style="background: #1a1a1a;">
      <span class="text-xs text-gray-600 whitespace-nowrap">历史:</span>
      <button
        v-for="(cmd, i) in history.slice(-10)"
        :key="i"
        @click="command = cmd"
        class="text-xs px-2 py-0.5 bg-gray-700 hover:bg-gray-600 rounded text-gray-400 whitespace-nowrap"
      >{{ cmd }}</button>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue'
import { invoke } from '../utils/tauri.js'

const servers = ref([
  { id: '1', name: '生产服务器', host: '192.168.1.100' },
  { id: '2', name: '测试服务器', host: '192.168.1.101' },
  { id: '3', name: '开发服务器', host: '192.168.1.102' },
  { id: '4', name: '备份服务器', host: '192.168.1.103' },
])

const selectedServers = ref([])
const command = ref('')
const running = ref(false)
const results = ref([])
const history = ref([])

function toggleServer(id) {
  const idx = selectedServers.value.indexOf(id)
  if (idx >= 0) {
    selectedServers.value.splice(idx, 1)
  } else {
    selectedServers.value.push(id)
  }
}

function selectAll() {
  if (selectedServers.value.length === servers.value.length) {
    selectedServers.value = []
  } else {
    selectedServers.value = servers.value.map(s => s.id)
  }
}

async function executeBatch() {
  if (!command.value || selectedServers.value.length === 0) return

  running.value = true
  history.value.push(command.value)
  results.value = []

  const cmd = command.value

  // Execute on all selected servers in parallel
  const promises = selectedServers.value.map(async (serverId) => {
    const server = servers.value.find(s => s.id === serverId)
    const result = {
      serverId,
      serverName: server.name,
      status: 'running',
      output: '',
      duration: 0,
    }
    results.value.push(result)

    const start = Date.now()
    try {
      // In real app, would use the server's actual session
      // For now, use mock execution
      const output = await invoke('ssh_execute', { command: cmd })
      result.output = output
      result.status = 'success'
    } catch (err) {
      result.output = `错误: ${err}`
      result.status = 'error'
    }
    result.duration = Date.now() - start
  })

  await Promise.all(promises)
  running.value = false
}
</script>
