<template>
  <div class="h-full flex flex-col bg-gray-900">
    <div class="h-9 bg-gray-800 border-b border-gray-700 flex items-center px-3 gap-2">
      <span class="text-sm font-medium text-gray-300">定时任务</span>
      <div class="flex-1" />
      <button @click="showAdd = true" class="text-xs px-2 py-0.5 bg-blue-600 hover:bg-blue-500 rounded text-white">+ 新增</button>
    </div>

    <div class="flex-1 overflow-y-auto p-3">
      <div v-for="task in tasks" :key="task.id" class="bg-gray-800 rounded mb-2 p-3">
        <div class="flex items-center justify-between mb-1">
          <div class="flex items-center gap-2">
            <span class="w-2 h-2 rounded-full" :class="task.enabled ? 'bg-green-400' : 'bg-gray-500'" />
            <span class="text-sm text-gray-200">{{ task.name }}</span>
          </div>
          <div class="flex gap-1">
            <button @click="toggleTask(task.id)" class="text-xs px-2 py-0.5 rounded" :class="task.enabled ? 'bg-yellow-600/30 text-yellow-300' : 'bg-green-600/30 text-green-300'">
              {{ task.enabled ? '暂停' : '启用' }}
            </button>
            <button @click="deleteTask(task.id)" class="text-xs px-2 py-0.5 rounded bg-gray-700 text-gray-400 hover:bg-gray-600 hover:text-white">删除</button>
          </div>
        </div>
        <div class="text-xs text-gray-500 font-mono">{{ task.command }}</div>
        <div class="text-xs text-gray-600 mt-1">
          计划: {{ task.schedule }} · 下次执行: {{ formatTime(task.nextRun) }}
        </div>
      </div>

      <div v-if="tasks.length === 0" class="text-center text-gray-500 text-sm mt-10">
        <div class="text-3xl mb-2">⏰</div>
        暂无定时任务<br/><span class="text-xs">设定时执行的命令</span>
      </div>
    </div>

    <!-- Add Dialog -->
    <div v-if="showAdd" class="fixed inset-0 bg-black/60 flex items-center justify-center z-50" @click.self="showAdd = false">
      <div class="bg-gray-800 rounded-lg w-96 border border-gray-600 p-4 space-y-3">
        <h3 class="text-sm font-medium">新增定时任务</h3>
        <input v-model="newTask.name" class="w-full bg-gray-900 border border-gray-600 rounded px-3 py-1.5 text-sm focus:outline-none focus:border-blue-500" placeholder="任务名称" />
        <input v-model="newTask.command" class="w-full bg-gray-900 border border-gray-600 rounded px-3 py-1.5 text-sm font-mono focus:outline-none focus:border-blue-500" placeholder="命令" />
        <div>
          <label class="text-xs text-gray-400 block mb-1">执行计划</label>
          <div class="flex gap-2 flex-wrap">
            <button v-for="preset in presets" :key="preset.value" @click="newTask.schedule = preset.value" class="text-xs px-2 py-1 rounded border" :class="newTask.schedule === preset.value ? 'bg-blue-600/20 border-blue-500 text-blue-300' : 'border-gray-600 text-gray-400'">
              {{ preset.label }}
            </button>
          </div>
          <input v-model="newTask.schedule" class="w-full bg-gray-900 border border-gray-600 rounded px-3 py-1.5 text-sm font-mono focus:outline-none focus:border-blue-500 mt-2" placeholder="自定义: 0 9 * * *" />
        </div>
        <div class="flex justify-end gap-2">
          <button @click="showAdd = false" class="px-3 py-1 text-sm text-gray-400">取消</button>
          <button @click="addTask" :disabled="!newTask.name || !newTask.command" class="px-3 py-1 text-sm bg-blue-600 rounded text-white disabled:opacity-50">添加</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, reactive } from 'vue'

const showAdd = ref(false)
const newTask = reactive({ name: '', command: '', schedule: '0 9 * * *' })

const presets = [
  { label: '每天9点', value: '0 9 * * *' },
  { label: '每小时', value: '0 * * * *' },
  { label: '每周一', value: '0 9 * * 1' },
  { label: '每30分钟', value: '*/30 * * * *' },
]

const tasks = ref([
  { id: '1', name: '每日备份检查', command: 'df -h && du -sh /var/backup/', schedule: '0 9 * * *', enabled: true, nextRun: new Date(Date.now() + 3600000).toISOString() },
  { id: '2', name: 'Docker 清理', command: 'docker system prune -f', schedule: '0 3 * * 0', enabled: true, nextRun: new Date(Date.now() + 86400000).toISOString() },
])

function formatTime(iso) {
  if (!iso) return '-'
  return new Date(iso).toLocaleString('zh-CN', { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' })
}

function addTask() {
  if (!newTask.name || !newTask.command) return
  tasks.value.push({
    id: Date.now().toString(),
    name: newTask.name,
    command: newTask.command,
    schedule: newTask.schedule,
    enabled: true,
    nextRun: new Date(Date.now() + 3600000).toISOString(),
  })
  newTask.name = ''; newTask.command = ''; newTask.schedule = '0 9 * * *'
  showAdd.value = false
}

function toggleTask(id) {
  const task = tasks.value.find(t => t.id === id)
  if (task) task.enabled = !task.enabled
}

function deleteTask(id) {
  tasks.value = tasks.value.filter(t => t.id !== id)
}
</script>
