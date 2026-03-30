<template>
  <div class="h-full flex flex-col bg-gray-900">
    <div class="h-9 bg-gray-800 border-b border-gray-700 flex items-center px-3 gap-2">
      <span class="text-sm font-medium text-gray-300">快捷命令</span>
      <div class="flex-1" />
      <button @click="showAdd = true" class="text-xs px-2 py-0.5 bg-blue-600 hover:bg-blue-500 rounded text-white">+ 新增</button>
    </div>

    <div class="flex-1 overflow-y-auto p-3">
      <div v-for="group in groupedCommands" :key="group.name" class="mb-4">
        <div class="text-xs text-gray-500 mb-2 flex items-center gap-2">
          <span>📁</span> {{ group.name }}
          <span class="text-gray-600">({{ group.items.length }})</span>
        </div>
        <div class="grid grid-cols-2 gap-2">
          <div
            v-for="cmd in group.items"
            :key="cmd.id"
            @click="$emit('run', cmd.command)"
            class="bg-gray-800 hover:bg-gray-700 rounded px-3 py-2 cursor-pointer group relative"
          >
            <div class="text-sm text-gray-200 truncate">{{ cmd.name }}</div>
            <div class="text-xs text-gray-500 truncate mt-0.5 font-mono">{{ cmd.command }}</div>
            <button
              @click.stop="deleteCommand(cmd.id)"
              class="absolute top-1 right-1 text-gray-600 hover:text-red-400 text-xs opacity-0 group-hover:opacity-100"
            >✕</button>
          </div>
        </div>
      </div>

      <div v-if="commands.length === 0" class="text-center text-gray-500 text-sm mt-10">
        暂无快捷命令<br/>
        <span class="text-xs">点击右上角 + 新增</span>
      </div>
    </div>

    <!-- Add Dialog -->
    <div v-if="showAdd" class="fixed inset-0 bg-black/60 flex items-center justify-center z-50" @click.self="showAdd = false">
      <div class="bg-gray-800 rounded-lg w-80 border border-gray-600 p-4 space-y-3">
        <h3 class="text-sm font-medium">新增快捷命令</h3>
        <input v-model="newCmd.name" class="w-full bg-gray-900 border border-gray-600 rounded px-3 py-1.5 text-sm focus:outline-none focus:border-blue-500" placeholder="名称" />
        <input v-model="newCmd.command" class="w-full bg-gray-900 border border-gray-600 rounded px-3 py-1.5 text-sm font-mono focus:outline-none focus:border-blue-500" placeholder="命令" />
        <input v-model="newCmd.group" class="w-full bg-gray-900 border border-gray-600 rounded px-3 py-1.5 text-sm focus:outline-none focus:border-blue-500" placeholder="分组（默认：自定义）" />
        <div class="flex justify-end gap-2">
          <button @click="showAdd = false" class="px-3 py-1 text-sm text-gray-400">取消</button>
          <button @click="addCommand" :disabled="!newCmd.name || !newCmd.command" class="px-3 py-1 text-sm bg-blue-600 rounded text-white disabled:opacity-50">添加</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, reactive } from 'vue'

defineEmits(['run'])

const showAdd = ref(false)
const newCmd = reactive({ name: '', command: '', group: '自定义' })

const commands = ref([
  { id: '1', name: '系统信息', command: 'uname -a', group: '系统' },
  { id: '2', name: '磁盘使用', command: 'df -h', group: '系统' },
  { id: '3', name: '内存使用', command: 'free -h', group: '系统' },
  { id: '4', name: '进程列表', command: 'ps aux --sort=-%mem | head -20', group: '系统' },
  { id: '5', name: '网络连接', command: 'ss -tlnp', group: '网络' },
  { id: '6', name: 'Docker 容器', command: 'docker ps -a', group: 'Docker' },
  { id: '7', name: 'Docker 日志', command: 'docker logs --tail 50', group: 'Docker' },
  { id: '8', name: 'Git 状态', command: 'git status', group: 'Git' },
  { id: '9', name: 'Git 日志', command: 'git log --oneline -10', group: 'Git' },
  { id: '10', name: '查找大文件', command: 'find / -type f -size +100M 2>/dev/null | head -10', group: '工具' },
])

const groupedCommands = computed(() => {
  const groups = {}
  for (const cmd of commands.value) {
    const g = cmd.group || '其他'
    if (!groups[g]) groups[g] = { name: g, items: [] }
    groups[g].items.push(cmd)
  }
  return Object.values(groups)
})

function addCommand() {
  if (!newCmd.name || !newCmd.command) return
  commands.value.push({
    id: Date.now().toString(),
    name: newCmd.name,
    command: newCmd.command,
    group: newCmd.group || '自定义',
  })
  newCmd.name = ''
  newCmd.command = ''
  newCmd.group = '自定义'
  showAdd.value = false
}

function deleteCommand(id) {
  commands.value = commands.value.filter(c => c.id !== id)
}
</script>
