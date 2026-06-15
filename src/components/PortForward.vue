<template>
  <div class="h-full flex flex-col" style="background: var(--bg-base)">
    <div class="h-9 border-b flex items-center px-3 gap-2" style="background: var(--bg-surface); border-color: var(--border)">
      <span class="text-sm font-medium" style="color: var(--fg-secondary)">端口转发</span>
      <span class="text-xs px-1.5 py-0.5 rounded border" style="background: color-mix(in srgb, var(--warning) 50%, transparent); color: var(--warning); border-color: color-mix(in srgb, var(--warning) 50%, transparent)">演示模式</span>
      <div class="flex-1" />
      <button @click="showAdd = true" class="text-xs px-2 py-0.5 rounded" style="background: var(--accent); color: white">+ 新增</button>
    </div>

    <div class="flex-1 overflow-y-auto p-3">
      <div v-for="fwd in forwards" :key="fwd.id" class="rounded mb-2 p-3" style="background: var(--bg-surface)">
        <div class="flex items-center justify-between mb-2">
          <div class="flex items-center gap-2">
            <span class="w-2 h-2 rounded-full" :style="fwd.active ? { background: 'var(--success)' } : { background: 'var(--fg-muted)' }" />
            <span class="text-sm" style="color: var(--fg-primary)">{{ fwd.name }}</span>
            <span class="text-xs px-1.5 py-0.5 rounded" :style="typeClass(fwd.type)">{{ fwd.type }}</span>
          </div>
          <div class="flex gap-1">
            <button @click="toggleForward(fwd)" class="text-xs px-2 py-0.5 rounded" :style="fwd.active ? { background: 'color-mix(in srgb, var(--danger) 30%, transparent)', color: 'var(--danger)' } : { background: 'color-mix(in srgb, var(--success) 30%, transparent)', color: 'var(--success)' }">
              {{ fwd.active ? '停止' : '启动' }}
            </button>
            <button @click="deleteForward(fwd.id)" class="text-xs px-2 py-0.5 rounded" style="background: var(--bg-elevated); color: var(--fg-muted)">删除</button>
          </div>
        </div>
        <div class="text-xs font-mono" style="color: var(--fg-muted)">
          <span v-if="fwd.type === 'local'">{{ fwd.localAddr }}:{{ fwd.localPort }} → {{ fwd.remoteAddr }}:{{ fwd.remotePort }}</span>
          <span v-else-if="fwd.type === 'remote'">{{ fwd.remoteAddr }}:{{ fwd.remotePort }} → {{ fwd.localAddr }}:{{ fwd.localPort }}</span>
          <span v-else>动态转发 {{ fwd.localAddr }}:{{ fwd.localPort }} (SOCKS5)</span>
        </div>
      </div>

      <div v-if="forwards.length === 0" class="text-center text-sm mt-10" style="color: var(--fg-muted)">
        暂无端口转发规则<br/>
        <span class="text-xs">点击右上角 + 新增</span>
      </div>
    </div>

    <!-- Add Dialog -->
    <BaseModal :show="showAdd" width="384px" @close="showAdd = false">
      <h3 class="text-sm font-medium">新增端口转发</h3>

      <div>
        <label class="text-xs block mb-1" style="color: var(--fg-muted)">名称</label>
        <input v-model="newFwd.name" class="w-full rounded px-3 py-1.5 text-sm focus:outline-none" style="background: var(--bg-base); border-color: var(--border-subtle)" placeholder="数据库连接" />
      </div>

      <div>
        <label class="text-xs block mb-1" style="color: var(--fg-muted)">类型</label>
        <div class="flex gap-2">
          <button
            v-for="t in types"
            :key="t.value"
            @click="newFwd.type = t.value"
            class="flex-1 px-2 py-1 text-xs rounded border"
            :style="newFwd.type === t.value ? { background: 'color-mix(in srgb, var(--accent) 20%, transparent)', color: 'var(--accent)' } : { color: 'var(--fg-muted)' }" :class="newFwd.type === t.value ? 'border-[var(--accent)]' : 'border-[var(--border-subtle)]'"
          >{{ t.label }}</button>
        </div>
      </div>

      <div class="flex gap-2">
        <div class="flex-1">
          <label class="text-xs block mb-1" style="color: var(--fg-muted)">本地端口</label>
          <input v-model.number="newFwd.localPort" type="number" class="w-full rounded px-3 py-1.5 text-sm focus:outline-none" style="background: var(--bg-base); border-color: var(--border-subtle)" placeholder="3306" />
        </div>
        <div v-if="newFwd.type !== 'dynamic'" class="flex-1">
          <label class="text-xs block mb-1" style="color: var(--fg-muted)">远程地址</label>
          <input v-model="newFwd.remoteAddr" class="w-full rounded px-3 py-1.5 text-sm focus:outline-none" style="background: var(--bg-base); border-color: var(--border-subtle)" placeholder="127.0.0.1" />
        </div>
        <div v-if="newFwd.type !== 'dynamic'" class="w-20">
          <label class="text-xs block mb-1" style="color: var(--fg-muted)">远程端口</label>
          <input v-model.number="newFwd.remotePort" type="number" class="w-full rounded px-3 py-1.5 text-sm focus:outline-none" style="background: var(--bg-base); border-color: var(--border-subtle)" placeholder="3306" />
        </div>
      </div>

      <div class="flex justify-end gap-2">
        <button @click="showAdd = false" class="px-3 py-1 text-sm" style="color: var(--fg-muted)">取消</button>
        <button @click="addForward" :disabled="!newFwd.name || !newFwd.localPort" class="px-3 py-1 text-sm rounded disabled:opacity-50" style="background: var(--accent); color: white">添加</button>
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
    local: { background: 'color-mix(in srgb, var(--accent) 30%, transparent)', color: 'var(--accent)' },
    remote: { background: 'color-mix(in srgb, var(--accent) 30%, transparent)', color: 'var(--accent)' },
    dynamic: { background: 'color-mix(in srgb, var(--warning) 30%, transparent)', color: 'var(--warning)' },
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
