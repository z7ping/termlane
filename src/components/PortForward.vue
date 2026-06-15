<template>
  <div class="h-full flex flex-col bg-gray-900">
    <div class="h-9 bg-gray-800 border-b border-gray-700 flex items-center px-3 gap-2">
      <span class="text-sm font-medium text-gray-300">端口转发</span>
      <span class="text-xs px-1.5 py-0.5 rounded bg-yellow-900/50 text-yellow-400 border border-yellow-700/50">演示模式</span>
      <div class="flex-1" />
      <button @click="showAdd = true" class="text-xs px-2 py-0.5 bg-blue-600 hover:bg-blue-500 rounded text-white">+ 新增</button>
    </div>

    <div class="flex-1 overflow-y-auto p-3">
      <div v-for="fwd in forwards" :key="fwd.id" class="bg-gray-800 rounded mb-2 p-3">
        <div class="flex items-center justify-between mb-2">
          <div class="flex items-center gap-2">
            <span class="w-2 h-2 rounded-full" :class="fwd.active ? 'bg-green-400' : 'bg-gray-500'" />
            <span class="text-sm text-gray-200">{{ fwd.name }}</span>
            <span class="text-xs px-1.5 py-0.5 rounded" :class="typeClass(fwd.type)">{{ fwd.type }}</span>
          </div>
          <div class="flex gap-1">
            <button @click="toggleForward(fwd)" class="text-xs px-2 py-0.5 rounded" :class="fwd.active ? 'bg-red-600/30 text-red-300 hover:bg-red-600/50' : 'bg-green-600/30 text-green-300 hover:bg-green-600/50'">
              {{ fwd.active ? '停止' : '启动' }}
            </button>
            <button @click="deleteForward(fwd.id)" class="text-xs px-2 py-0.5 rounded bg-gray-700 text-gray-400 hover:bg-gray-600 hover:text-white">删除</button>
          </div>
        </div>
        <div class="text-xs text-gray-400 font-mono">
          <span v-if="fwd.type === 'local'">{{ fwd.localAddr }}:{{ fwd.localPort }} → {{ fwd.remoteAddr }}:{{ fwd.remotePort }}</span>
          <span v-else-if="fwd.type === 'remote'">{{ fwd.remoteAddr }}:{{ fwd.remotePort }} → {{ fwd.localAddr }}:{{ fwd.localPort }}</span>
          <span v-else>动态转发 {{ fwd.localAddr }}:{{ fwd.localPort }} (SOCKS5)</span>
        </div>
      </div>

      <div v-if="forwards.length === 0" class="text-center text-gray-500 text-sm mt-10">
        暂无端口转发规则<br/>
        <span class="text-xs">点击右上角 + 新增</span>
      </div>
    </div>

    <!-- Add Dialog -->
    <BaseModal :show="showAdd" width="384px" @close="showAdd = false">
      <h3 class="text-sm font-medium">新增端口转发</h3>

      <div>
        <label class="text-xs text-gray-400 block mb-1">名称</label>
        <input v-model="newFwd.name" class="w-full bg-gray-900 border border-gray-600 rounded px-3 py-1.5 text-sm focus:outline-none focus:border-blue-500" placeholder="数据库连接" />
      </div>

      <div>
        <label class="text-xs text-gray-400 block mb-1">类型</label>
        <div class="flex gap-2">
          <button
            v-for="t in types"
            :key="t.value"
            @click="newFwd.type = t.value"
            class="flex-1 px-2 py-1 text-xs rounded border"
            :class="newFwd.type === t.value ? 'bg-blue-600/20 border-blue-500 text-blue-300' : 'border-gray-600 text-gray-400'"
          >{{ t.label }}</button>
        </div>
      </div>

      <div class="flex gap-2">
        <div class="flex-1">
          <label class="text-xs text-gray-400 block mb-1">本地端口</label>
          <input v-model.number="newFwd.localPort" type="number" class="w-full bg-gray-900 border border-gray-600 rounded px-3 py-1.5 text-sm focus:outline-none focus:border-blue-500" placeholder="3306" />
        </div>
        <div v-if="newFwd.type !== 'dynamic'" class="flex-1">
          <label class="text-xs text-gray-400 block mb-1">远程地址</label>
          <input v-model="newFwd.remoteAddr" class="w-full bg-gray-900 border border-gray-600 rounded px-3 py-1.5 text-sm focus:outline-none focus:border-blue-500" placeholder="127.0.0.1" />
        </div>
        <div v-if="newFwd.type !== 'dynamic'" class="w-20">
          <label class="text-xs text-gray-400 block mb-1">远程端口</label>
          <input v-model.number="newFwd.remotePort" type="number" class="w-full bg-gray-900 border border-gray-600 rounded px-3 py-1.5 text-sm focus:outline-none focus:border-blue-500" placeholder="3306" />
        </div>
      </div>

      <div class="flex justify-end gap-2">
        <button @click="showAdd = false" class="px-3 py-1 text-sm text-gray-400">取消</button>
        <button @click="addForward" :disabled="!newFwd.name || !newFwd.localPort" class="px-3 py-1 text-sm bg-blue-600 rounded text-white disabled:opacity-50">添加</button>
      </div>
    </BaseModal>
  </div>
</template>

<script setup>
import { ref, reactive } from 'vue'
import BaseModal from './BaseModal.vue'

const showAdd = ref(false)

const types = [
  { value: 'local', label: '本地转发' },
  { value: 'remote', label: '远程转发' },
  { value: 'dynamic', label: '动态(SOCKS5)' },
]

const newFwd = reactive({
  name: '',
  type: 'local',
  localAddr: '127.0.0.1',
  localPort: null,
  remoteAddr: '127.0.0.1',
  remotePort: null,
})

const forwards = ref([
  { id: '1', name: 'MySQL', type: 'local', localAddr: '127.0.0.1', localPort: 13306, remoteAddr: '127.0.0.1', remotePort: 3306, active: false },
  { id: '2', name: 'Redis', type: 'local', localAddr: '127.0.0.1', localPort: 16379, remoteAddr: '127.0.0.1', remotePort: 6379, active: false },
  { id: '3', name: 'PostgreSQL', type: 'local', localAddr: '127.0.0.1', localPort: 15432, remoteAddr: '127.0.0.1', remotePort: 5432, active: false },
])

function typeClass(type) {
  return {
    local: 'bg-blue-600/30 text-blue-300',
    remote: 'bg-purple-600/30 text-purple-300',
    dynamic: 'bg-orange-600/30 text-orange-300',
  }[type]
}

function toggleForward(fwd) {
  fwd.active = !fwd.active
}

function addForward() {
  if (!newFwd.name || !newFwd.localPort) return
  forwards.value.push({
    id: Date.now().toString(),
    ...{ ...newFwd },
    active: false,
  })
  newFwd.name = ''
  newFwd.localPort = null
  newFwd.remotePort = null
  newFwd.remoteAddr = '127.0.0.1'
  showAdd.value = false
}

function deleteForward(id) {
  forwards.value = forwards.value.filter(f => f.id !== id)
}
</script>
