<template>
  <div class="h-full flex flex-col" style="background: var(--bg-base)">
    <div class="h-9 border-b flex items-center px-3 gap-2" style="background: var(--bg-surface); border-color: var(--border)">
      <span class="text-sm font-medium" style="color: var(--fg-secondary)">快捷命令</span>
      <div class="flex-1" />
      <button @click="showAdd = true" class="text-xs px-2 py-0.5 rounded" style="background: var(--accent); color: white">+ 新增</button>
    </div>

    <div class="flex-1 overflow-y-auto p-3">
      <div v-for="group in groupedCommands" :key="group.name" class="mb-4">
        <div class="text-xs mb-2 flex items-center gap-2" style="color: var(--fg-muted)">
          <span>📁</span> {{ group.name }}
          <span class="" style="color: var(--fg-muted)">({{ group.items.length }})</span>
        </div>
        <div class="grid grid-cols-2 gap-2">
          <div
            v-for="cmd in group.items"
            :key="cmd.id"
            @click="$emit('run', cmd.command)"
            class="rounded px-3 py-2 cursor-pointer group relative" style="background: var(--bg-surface)"
          >
            <div class="text-sm truncate" style="color: var(--fg-primary)">{{ cmd.name }}</div>
            <div class="text-xs truncate mt-0.5 font-mono" style="color: var(--fg-muted)">{{ cmd.command }}</div>
            <button
              @click.stop="deleteCommand(cmd.id)"
              class="absolute top-1 right-1 text-xs opacity-0 group-hover:opacity-100" style="color: var(--fg-muted)"
            >✕</button>
          </div>
        </div>
      </div>

      <div v-if="commands.length === 0" class="text-center text-sm mt-10" style="color: var(--fg-muted)">
        暂无快捷命令<br/>
        <span class="text-xs">点击右上角 + 新增</span>
      </div>
    </div>

    <!-- Add Dialog -->
    <div v-if="showAdd" class="fixed inset-0 flex items-center justify-center z-50" style="background: color-mix(in srgb, #000 60%, transparent)" @click.self="showAdd = false">
      <div class="rounded-lg w-80 p-4 space-y-3" style="background: var(--bg-surface); border-color: var(--border-subtle)">
        <h3 class="text-sm font-medium">新增快捷命令</h3>
        <input v-model="newCmd.name" class="w-full rounded px-3 py-1.5 text-sm focus:outline-none" style="background: var(--bg-base); border-color: var(--border-subtle)" placeholder="名称" />
        <input v-model="newCmd.command" class="w-full rounded px-3 py-1.5 text-sm font-mono focus:outline-none" style="background: var(--bg-base); border-color: var(--border-subtle)" placeholder="命令" />
        <input v-model="newCmd.group" class="w-full rounded px-3 py-1.5 text-sm focus:outline-none" style="background: var(--bg-base); border-color: var(--border-subtle)" placeholder="分组（默认：自定义）" />
        <div class="flex justify-end gap-2">
          <button @click="showAdd = false" class="px-3 py-1 text-sm" style="color: var(--fg-muted)">取消</button>
          <button @click="addCommand" :disabled="!newCmd.name || !newCmd.command" class="px-3 py-1 text-sm rounded disabled:opacity-50" style="background: var(--accent); color: white">添加</button>
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
