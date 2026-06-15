<template>
  <div class="h-full flex flex-col" style="background: var(--bg-base)">
    <div class="h-9 border-b flex items-center px-3 gap-2" style="background: var(--bg-surface); border-color: var(--border)">
      <span class="text-sm font-medium" style="color: var(--fg-secondary)">批量命令</span>
      <span class="text-xs px-1.5 py-0.5 rounded border" style="background: color-mix(in srgb, var(--warning) 50%, transparent); color: var(--warning); border-color: color-mix(in srgb, var(--warning) 50%, transparent)">演示模式</span>
      <div class="flex-1" />
      <span class="text-xs" style="color: var(--fg-muted)">{{ selectedServers.length }} 台已选</span>
    </div>

    <div class="flex-1 flex overflow-hidden">
      <!-- 服务器选择 -->
      <div class="w-48 border-r flex flex-col" style="border-color: var(--border)">
        <div class="p-2 border-b flex items-center justify-between" style="border-color: var(--border)">
          <span class="text-xs" style="color: var(--fg-muted)">选择服务器</span>
          <button @click="selectAll" class="text-xs" style="color: var(--accent)">全选</button>
        </div>
        <div class="flex-1 overflow-y-auto p-1">
          <div
            v-for="server in servers"
            :key="server.id"
            @click="toggleServer(server.id)"
            class="flex items-center gap-2 px-2 py-1.5 rounded cursor-pointer text-sm"
            :style="selectedServers.includes(server.id) ? { background: 'color-mix(in srgb, var(--accent) 20%, transparent)', color: 'var(--accent)' } : { color: 'var(--fg-secondary)' }"
          >
            <span class="w-2 h-2 rounded-full" :style="selectedServers.includes(server.id) ? { background: 'var(--accent)' } : { background: 'var(--border-subtle)' }" />
            <span class="truncate">{{ server.name }}</span>
          </div>
        </div>
      </div>

      <!-- 命令输入 + 输出 -->
      <div class="flex-1 flex flex-col">
        <!-- 命令输入 -->
        <div class="p-2 border-b flex gap-2" style="border-color: var(--border)">
          <input
            v-model="command"
            @keydown.enter="executeBatch"
            class="flex-1 text-sm px-3 py-1.5 rounded focus:outline-none font-mono" style="background: var(--bg-base); color: var(--fg-primary); border-color: var(--border-subtle)"
            placeholder="输入命令，按 Enter 执行到所有选中服务器..."
            :disabled="running"
          />
          <button
            @click="executeBatch"
            :disabled="running || !command || selectedServers.length === 0"
            class="px-3 py-1.5 text-sm rounded"
            :style="running ? { background: 'var(--warning)', color: 'white' } : { background: 'var(--accent)', color: 'white' }"
          >{{ running ? '执行中...' : '执行' }}</button>
        </div>

        <!-- 输出区域 -->
        <div class="flex-1 overflow-y-auto p-2 space-y-2">
          <div v-for="result in results" :key="result.serverId" class="rounded overflow-hidden" style="background: var(--bg-surface)">
            <div class="flex items-center justify-between px-3 py-1.5 border-b" style="border-color: var(--border)">
              <div class="flex items-center gap-2">
                <span class="w-2 h-2 rounded-full" :style="result.status === 'success' ? { background: 'var(--success)' } : result.status === 'error' ? { background: 'var(--danger)' } : { background: 'var(--warning)', animation: 'pulse 2s infinite' }" />
                <span class="text-sm" style="color: var(--fg-secondary)">{{ result.serverName }}</span>
              </div>
              <span class="text-xs" style="color: var(--fg-muted)">{{ result.duration }}ms</span>
            </div>
            <pre class="p-3 text-xs font-mono overflow-x-auto whitespace-pre-wrap max-h-40" style="color: var(--fg-secondary)">{{ result.output || '(无输出)' }}</pre>
          </div>

          <div v-if="results.length === 0" class="text-center text-sm mt-10" style="color: var(--fg-muted)">
            选择服务器 → 输入命令 → 执行
          </div>
        </div>
      </div>
    </div>

    <!-- 历史记录 -->
    <div v-if="history.length > 0" class="h-16 border-t overflow-x-auto flex items-center gap-2 px-2" style="background: var(--bg-surface); border-color: var(--border)">
      <span class="text-xs whitespace-nowrap" style="color: var(--fg-muted)">历史:</span>
      <button
        v-for="(cmd, i) in history.slice(-10)"
        :key="i"
        @click="command = cmd"
        class="text-xs px-2 py-0.5 rounded whitespace-nowrap" style="background: var(--bg-elevated); color: var(--fg-muted)"
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
