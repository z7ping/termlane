<template>
  <div class="fixed inset-0 bg-black/60 flex items-center justify-center z-50" @click.self="$emit('close')">
    <div class="bg-gray-800 rounded-lg w-[440px] border border-gray-600 shadow-2xl max-h-[90vh] overflow-y-auto">
      <div class="p-4 border-b border-gray-700 flex items-center justify-between">
        <h3 class="text-sm font-medium">{{ editing ? '编辑连接' : '新建连接' }}</h3>
        <button @click="$emit('close')" class="text-gray-400 hover:text-white">✕</button>
      </div>

      <div class="p-4 space-y-3">
        <!-- 名称 -->
        <div>
          <label class="text-xs text-gray-400 block mb-1">名称</label>
          <input v-model="form.name" class="w-full bg-gray-900 border border-gray-600 rounded px-3 py-1.5 text-sm focus:outline-none focus:border-blue-500" placeholder="我的服务器" />
        </div>

        <!-- 主机 + 端口 -->
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

        <!-- 用户名 -->
        <div>
          <label class="text-xs text-gray-400 block mb-1">用户名</label>
          <input v-model="form.username" class="w-full bg-gray-900 border border-gray-600 rounded px-3 py-1.5 text-sm focus:outline-none focus:border-blue-500" placeholder="root" />
        </div>

        <!-- 认证方式 -->
        <div>
          <label class="text-xs text-gray-400 block mb-1">认证方式</label>
          <div class="flex gap-2">
            <button
              v-for="auth in authTypes"
              :key="auth.value"
              @click="form.authType = auth.value"
              class="flex-1 px-3 py-1.5 text-sm rounded border"
              :class="form.authType === auth.value ? 'bg-blue-600/20 border-blue-500 text-blue-300' : 'border-gray-600 text-gray-400 hover:border-gray-500'"
            >{{ auth.label }}</button>
          </div>
        </div>

        <!-- 密码 -->
        <div v-if="form.authType === 'password'">
          <label class="text-xs text-gray-400 block mb-1">密码</label>
          <div class="relative">
            <input
              v-model="form.password"
              :type="showPassword ? 'text' : 'password'"
              class="w-full bg-gray-900 border border-gray-600 rounded px-3 py-1.5 text-sm focus:outline-none focus:border-blue-500 pr-8"
            />
            <button
              @click="showPassword = !showPassword"
              class="absolute right-2 top-1/2 -translate-y-1/2 text-gray-500 hover:text-gray-300 text-xs"
            >{{ showPassword ? '🙈' : '👁️' }}</button>
          </div>
        </div>

        <!-- 密钥路径 -->
        <div v-if="form.authType === 'key'">
          <label class="text-xs text-gray-400 block mb-1">密钥路径</label>
          <input v-model="form.keyPath" class="w-full bg-gray-900 border border-gray-600 rounded px-3 py-1.5 text-sm focus:outline-none focus:border-blue-500" placeholder="~/.ssh/id_rsa" />
        </div>
        <div v-if="form.authType === 'key'">
          <label class="text-xs text-gray-400 block mb-1">密钥密码（可选）</label>
          <input v-model="form.passphrase" type="password" class="w-full bg-gray-900 border border-gray-600 rounded px-3 py-1.5 text-sm focus:outline-none focus:border-blue-500" />
        </div>

        <!-- 分组 -->
        <div>
          <label class="text-xs text-gray-400 block mb-1">分组</label>
          <div class="flex gap-2">
            <input v-model="form.group" class="flex-1 bg-gray-900 border border-gray-600 rounded px-3 py-1.5 text-sm focus:outline-none focus:border-blue-500" placeholder="默认" />
            <select v-model="form.icon" class="bg-gray-900 border border-gray-600 rounded px-2 text-sm">
              <option value="🖥️">🖥️</option>
              <option value="☁️">☁️</option>
              <option value="🏠">🏠</option>
              <option value="🔒">🔒</option>
              <option value="🌐">🌐</option>
              <option value="💾">💾</option>
            </select>
          </div>
        </div>
      </div>

      <!-- 测试结果 -->
      <div v-if="testResult" class="px-4 pb-2">
        <div
          class="text-xs px-3 py-2 rounded"
          :class="testResult.success ? 'bg-green-900/50 text-green-300' : 'bg-red-900/50 text-red-300'"
        >{{ testResult.message }}</div>
      </div>

      <div class="p-4 border-t border-gray-700 flex justify-between">
        <button
          @click="testConnection"
          :disabled="testing || !form.host || !form.username"
          class="px-4 py-1.5 text-sm text-gray-400 hover:text-white disabled:opacity-50"
        >{{ testing ? '测试中...' : '🔗 测试连接' }}</button>
        <div class="flex gap-2">
          <button @click="$emit('close')" class="px-4 py-1.5 text-sm text-gray-400 hover:text-white">取消</button>
          <button
            @click="save"
            :disabled="!form.name || !form.host || !form.username"
            class="px-4 py-1.5 text-sm bg-blue-600 hover:bg-blue-500 rounded text-white disabled:opacity-50"
          >保存</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { reactive, ref } from 'vue'
import { invoke } from '../utils/tauri.js'

const props = defineProps({
  editing: Object,
})

const emit = defineEmits(['save', 'close'])

const showPassword = ref(false)
const testing = ref(false)
const testResult = ref(null)

const authTypes = [
  { value: 'password', label: '密码' },
  { value: 'key', label: 'SSH 密钥' },
]

const form = reactive({
  name: props.editing?.name || '',
  host: props.editing?.host || '',
  port: props.editing?.port || 22,
  username: props.editing?.username || '',
  authType: props.editing?.authType || 'password',
  password: props.editing?.password || '',
  keyPath: props.editing?.keyPath || '',
  passphrase: props.editing?.passphrase || '',
  group: props.editing?.group || '默认',
  icon: props.editing?.icon || '🖥️',
})

async function testConnection() {
  testing.value = true
  testResult.value = null

  try {
    let sid
    if (form.authType === 'key') {
      sid = await invoke('ssh_connect_key', {
        host: form.host, port: form.port, username: form.username,
        keyPath: form.keyPath, passphrase: form.passphrase,
      })
    } else {
      sid = await invoke('ssh_connect', {
        host: form.host, port: form.port, username: form.username,
        password: form.password,
      })
    }
    testResult.value = { success: true, message: `✓ 连接成功！会话: ${sid}` }
    // 断开测试连接
    await invoke('ssh_disconnect', { sessionId: sid }).catch(() => {})
  } catch (err) {
    testResult.value = { success: false, message: `✗ 连接失败: ${err}` }
  } finally {
    testing.value = false
  }
}

function save() {
  if (!form.name || !form.host || !form.username) return
  emit('save', { ...form })
}
</script>
