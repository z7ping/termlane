<template>
  <div class="h-full flex flex-col" style="background: var(--bg-base)">
    <div class="h-9 border-b flex items-center px-3 gap-2" style="background: var(--bg-surface); border-color: var(--border)">
      <span class="text-sm font-medium" style="color: var(--fg-secondary)">定时任务</span>
      <span class="text-xs px-1.5 py-0.5 rounded border" style="background: color-mix(in srgb, var(--warning) 50%, transparent); color: var(--warning); border-color: color-mix(in srgb, var(--warning) 50%, transparent)">演示模式</span>
      <div class="flex-1" />
      <button @click="showAdd = true" class="text-xs px-2 py-0.5 rounded" style="background: var(--accent); color: white">+ 新增</button>
    </div>

    <div class="flex-1 overflow-y-auto p-3">
      <div v-for="task in tasks" :key="task.id" class="rounded mb-2 p-3" style="background: var(--bg-surface)">
        <div class="flex items-center justify-between mb-1">
          <div class="flex items-center gap-2">
            <span class="w-2 h-2 rounded-full" :style="task.enabled ? { background: 'var(--success)' } : { background: 'var(--fg-muted)' }" />
            <span class="text-sm" style="color: var(--fg-primary)">{{ task.name }}</span>
          </div>
          <div class="flex gap-1">
            <button @click="toggleTask(task.id)" class="text-xs px-2 py-0.5 rounded" :style="task.enabled ? { background: 'color-mix(in srgb, var(--warning) 30%, transparent)', color: 'var(--warning)' } : { background: 'color-mix(in srgb, var(--success) 30%, transparent)', color: 'var(--success)' }">
              {{ task.enabled ? '暂停' : '启用' }}
            </button>
            <button @click="deleteTask(task.id)" class="text-xs px-2 py-0.5 rounded" style="background: var(--bg-elevated); color: var(--fg-muted)">删除</button>
          </div>
        </div>
        <div class="text-xs font-mono" style="color: var(--fg-muted)">{{ task.command }}</div>
        <div class="text-xs mt-1" style="color: var(--fg-muted)">
          计划: {{ task.schedule }} · 下次执行: {{ formatTime(task.nextRun) }}
        </div>
      </div>

      <div v-if="tasks.length === 0" class="text-center text-sm mt-10" style="color: var(--fg-muted)">
        <div class="text-3xl mb-2">⏰</div>
        暂无定时任务<br/><span class="text-xs">设定时执行的命令</span>
      </div>
    </div>

    <!-- Add Dialog -->
    <BaseModal :show="showAdd" width="384px" @close="showAdd = false">
      <h3 class="text-sm font-medium">新增定时任务</h3>
      <input v-model="newTask.name" class="w-full rounded px-3 py-1.5 text-sm focus:outline-none" style="background: var(--bg-base); border-color: var(--border-subtle)" placeholder="任务名称" />
      <input v-model="newTask.command" class="w-full rounded px-3 py-1.5 text-sm font-mono focus:outline-none" style="background: var(--bg-base); border-color: var(--border-subtle)" placeholder="命令" />
      <div>
        <label class="text-xs block mb-1" style="color: var(--fg-muted)">执行计划</label>
        <div class="flex gap-2 flex-wrap">
          <button v-for="preset in presets" :key="preset.value" @click="newTask.schedule = preset.value" class="text-xs px-2 py-1 rounded border" :style="newTask.schedule === preset.value ? { background: 'color-mix(in srgb, var(--accent) 20%, transparent)', color: 'var(--accent)' } : { color: 'var(--fg-muted)' }" :class="newTask.schedule === preset.value ? 'border-[var(--accent)]' : 'border-[var(--border-subtle)]'">
            {{ preset.label }}
          </button>
        </div>
        <input v-model="newTask.schedule" class="w-full rounded px-3 py-1.5 text-sm font-mono focus:outline-none mt-2" style="background: var(--bg-base); border-color: var(--border-subtle)" placeholder="自定义: 0 9 * * *" />
      </div>
      <div class="flex justify-end gap-2">
        <button @click="showAdd = false" class="px-3 py-1 text-sm" style="color: var(--fg-muted)">取消</button>
        <button @click="addTask" :disabled="!newTask.name || !newTask.command" class="px-3 py-1 text-sm rounded disabled:opacity-50" style="background: var(--accent); color: white">添加</button>
      </div>
    </BaseModal>
  </div>
</template>

<script setup>
import { ref, reactive } from 'vue'
import BaseModal from './BaseModal.vue'

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
