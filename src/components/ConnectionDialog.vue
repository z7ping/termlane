<template>
  <div class="fixed inset-0 bg-black/60 flex items-center justify-center z-50" @click.self="$emit('close')">
    <div class="bg-gray-800 rounded-lg w-96 border border-gray-600 shadow-2xl">
      <div class="p-4 border-b border-gray-700">
        <h3 class="text-sm font-medium">新建连接</h3>
      </div>
      <div class="p-4 space-y-3">
        <div>
          <label class="text-xs text-gray-400 block mb-1">名称</label>
          <input v-model="form.name" class="w-full bg-gray-900 border border-gray-600 rounded px-3 py-1.5 text-sm focus:outline-none focus:border-blue-500" placeholder="我的服务器" />
        </div>
        <div class="flex gap-2">
          <div class="flex-1">
            <label class="text-xs text-gray-400 block mb-1">主机</label>
            <input v-model="form.host" class="w-full bg-gray-900 border border-gray-600 rounded px-3 py-1.5 text-sm focus:outline-none focus:border-blue-500" placeholder="192.168.1.1" />
          </div>
          <div class="w-20">
            <label class="text-xs text-gray-400 block mb-1">端口</label>
            <input v-model.number="form.port" type="number" class="w-full bg-gray-900 border border-gray-600 rounded px-3 py-1.5 text-sm focus:outline-none focus:border-blue-500" />
          </div>
        </div>
        <div>
          <label class="text-xs text-gray-400 block mb-1">用户名</label>
          <input v-model="form.username" class="w-full bg-gray-900 border border-gray-600 rounded px-3 py-1.5 text-sm focus:outline-none focus:border-blue-500" placeholder="root" />
        </div>
        <div>
          <label class="text-xs text-gray-400 block mb-1">认证方式</label>
          <select v-model="form.authType" class="w-full bg-gray-900 border border-gray-600 rounded px-3 py-1.5 text-sm focus:outline-none focus:border-blue-500">
            <option value="password">密码</option>
            <option value="key">SSH 密钥</option>
          </select>
        </div>
        <div v-if="form.authType === 'password'">
          <label class="text-xs text-gray-400 block mb-1">密码</label>
          <input v-model="form.password" type="password" class="w-full bg-gray-900 border border-gray-600 rounded px-3 py-1.5 text-sm focus:outline-none focus:border-blue-500" />
        </div>
        <div v-if="form.authType === 'key'">
          <label class="text-xs text-gray-400 block mb-1">密钥路径</label>
          <input v-model="form.keyPath" class="w-full bg-gray-900 border border-gray-600 rounded px-3 py-1.5 text-sm focus:outline-none focus:border-blue-500" placeholder="~/.ssh/id_rsa" />
        </div>
        <div>
          <label class="text-xs text-gray-400 block mb-1">分组</label>
          <input v-model="form.group" class="w-full bg-gray-900 border border-gray-600 rounded px-3 py-1.5 text-sm focus:outline-none focus:border-blue-500" placeholder="生产环境" />
        </div>
      </div>
      <div class="p-4 border-t border-gray-700 flex justify-end gap-2">
        <button @click="$emit('close')" class="px-4 py-1.5 text-sm text-gray-400 hover:text-white">取消</button>
        <button @click="save" class="px-4 py-1.5 text-sm bg-blue-600 hover:bg-blue-500 rounded text-white">保存</button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { reactive } from 'vue'

const emit = defineEmits(['save', 'close'])

const form = reactive({
  name: '',
  host: '',
  port: 22,
  username: '',
  authType: 'password',
  password: '',
  keyPath: '',
  group: '默认',
  icon: '🖥️',
})

function save() {
  if (!form.name || !form.host || !form.username) return
  emit('save', { ...form })
}
</script>
