<template>
  <div class="h-full flex flex-col bg-gray-900">
    <div class="h-9 bg-gray-800 border-b border-gray-700 flex items-center px-3 gap-2">
      <span class="text-sm font-medium text-gray-300">命令片段库</span>
      <div class="flex-1" />
      <input v-model="search" class="w-40 bg-gray-900 text-xs text-gray-300 px-2 py-1 rounded border border-gray-600 focus:outline-none focus:border-blue-500" placeholder="🔍 搜索..." />
      <button @click="showAdd = true" class="text-xs px-2 py-0.5 bg-blue-600 hover:bg-blue-500 rounded text-white">+ 新增</button>
    </div>

    <div class="flex-1 overflow-y-auto p-3">
      <div v-for="group in filteredGroups" :key="group.name" class="mb-4">
        <div class="text-xs text-gray-500 mb-2 flex items-center gap-2" @click="toggleGroup(group.name)">
          <span class="transition-transform" :class="collapsed[group.name] ? '-rotate-90' : ''">▼</span>
          {{ group.name }}
          <span class="text-gray-600">({{ group.items.length }})</span>
        </div>
        <div v-show="!collapsed[group.name]" class="space-y-1">
          <div v-for="s in group.items" :key="s.id" class="bg-gray-800 rounded p-2 group hover:bg-gray-750">
            <div class="flex items-center justify-between">
              <div class="text-sm text-gray-200">{{ s.name }}</div>
              <div class="flex gap-1 opacity-0 group-hover:opacity-100">
                <button @click="copySnippet(s)" class="text-xs px-1.5 py-0.5 bg-gray-700 hover:bg-gray-600 rounded text-gray-300">📋 复制</button>
                <button @click="$emit('run', s.command)" class="text-xs px-1.5 py-0.5 bg-blue-600/30 hover:bg-blue-600/50 rounded text-blue-300">▶ 执行</button>
                <button @click="deleteSnippet(s.id)" class="text-xs px-1.5 py-0.5 bg-gray-700 hover:bg-gray-600 rounded text-gray-400">✕</button>
              </div>
            </div>
            <div class="text-xs text-gray-500 font-mono mt-1 break-all">{{ s.command }}</div>
            <div v-if="s.desc" class="text-xs text-gray-600 mt-1">{{ s.desc }}</div>
          </div>
        </div>
      </div>
      <div v-if="filteredGroups.length === 0" class="text-center text-gray-500 text-sm mt-10">
        <div class="text-3xl mb-2">📝</div>
        暂无命令片段<br/><span class="text-xs">点击 + 新增常用命令</span>
      </div>
    </div>

    <!-- Add Dialog -->
    <div v-if="showAdd" class="fixed inset-0 bg-black/60 flex items-center justify-center z-50" @click.self="showAdd = false">
      <div class="bg-gray-800 rounded-lg w-96 border border-gray-600 p-4 space-y-3">
        <h3 class="text-sm font-medium">新增命令片段</h3>
        <input v-model="newS.name" class="w-full bg-gray-900 border border-gray-600 rounded px-3 py-1.5 text-sm focus:outline-none focus:border-blue-500" placeholder="名称（如：查看日志）" />
        <textarea v-model="newS.command" class="w-full bg-gray-900 border border-gray-600 rounded px-3 py-1.5 text-sm font-mono focus:outline-none focus:border-blue-500 h-20 resize-none" placeholder="命令" />
        <input v-model="newS.desc" class="w-full bg-gray-900 border border-gray-600 rounded px-3 py-1.5 text-sm focus:outline-none focus:border-blue-500" placeholder="说明（可选）" />
        <input v-model="newS.group" class="w-full bg-gray-900 border border-gray-600 rounded px-3 py-1.5 text-sm focus:outline-none focus:border-blue-500" placeholder="分组（默认：自定义）" />
        <div class="flex justify-end gap-2">
          <button @click="showAdd = false" class="px-3 py-1 text-sm text-gray-400">取消</button>
          <button @click="addSnippet" :disabled="!newS.name || !newS.command" class="px-3 py-1 text-sm bg-blue-600 rounded text-white disabled:opacity-50">添加</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, reactive } from 'vue'

defineEmits(['run'])

const showAdd = ref(false)
const search = ref('')
const collapsed = reactive({})
const newS = reactive({ name: '', command: '', desc: '', group: '自定义' })

const snippets = ref([
  { id: '1', name: '查看系统日志', command: 'journalctl -xe --no-pager | tail -50', group: '系统', desc: '查看最近50条系统日志' },
  { id: '2', name: '查看磁盘IO', command: 'iostat -x 1 5', group: '系统', desc: '磁盘IO统计' },
  { id: '3', name: '查看网络连接', command: 'ss -tlnp', group: '网络', desc: '查看监听端口' },
  { id: '4', name: '查看进程树', command: 'pstree -p', group: '系统', desc: '进程树状图' },
  { id: '5', name: 'Docker 日志', command: 'docker logs --tail 100 -f', group: 'Docker', desc: '实时查看容器日志' },
  { id: '6', name: 'Docker 清理', command: 'docker system prune -af', group: 'Docker', desc: '清理未使用资源' },
  { id: '7', name: 'Git 状态', command: 'git status && git log --oneline -5', group: 'Git', desc: '查看状态和最近提交' },
  { id: '8', name: '查找大文件', command: 'find / -type f -size +100M -exec ls -lh {} \\; 2>/dev/null', group: '工具', desc: '查找大于100M的文件' },
  { id: '9', name: '内存占用TOP10', command: 'ps aux --sort=-%mem | head -11', group: '系统', desc: '内存占用最多的10个进程' },
  { id: '10', name: 'CPU占用TOP10', command: 'ps aux --sort=-%cpu | head -11', group: '系统', desc: 'CPU占用最多的10个进程' },
])

const filteredGroups = computed(() => {
  const groups = {}
  for (const s of snippets.value) {
    if (search.value && !s.name.includes(search.value) && !s.command.includes(search.value)) continue
    const g = s.group || '其他'
    if (!groups[g]) groups[g] = { name: g, items: [] }
    groups[g].items.push(s)
  }
  return Object.values(groups)
})

function toggleGroup(name) { collapsed[name] = !collapsed[name] }

function copySnippet(s) {
  navigator.clipboard.writeText(s.command).catch(() => {})
}

function addSnippet() {
  if (!newS.name || !newS.command) return
  snippets.value.push({ id: Date.now().toString(), ...{ ...newS } })
  newS.name = ''; newS.command = ''; newS.desc = ''; newS.group = '自定义'
  showAdd.value = false
}

function deleteSnippet(id) { snippets.value = snippets.value.filter(s => s.id !== id) }
</script>
