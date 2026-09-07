<template>
  <Transition name="fade">
    <div class="fixed inset-0 bg-black/60 flex items-center justify-center z-50 backdrop-blur-sm" role="dialog" :aria-label="editing ? '编辑SSH连接' : '新建SSH连接'" aria-modal="true">
      <Transition name="slide-up">
        <div class="rounded-xl w-[480px] max-h-[90vh] overflow-y-auto shadow-2xl" style="background: var(--bg-elevated); border: 1px solid var(--border);">
          <div class="p-4 flex items-center justify-between" style="border-bottom: 1px solid var(--border-subtle);">
            <h3 class="text-sm font-semibold" style="color: var(--fg-primary);">{{ editing ? '编辑连接' : '新建连接' }}</h3>
            <button @click="$emit('close')" aria-label="关闭对话框" style="color: var(--fg-muted);">✕</button>
          </div>

          <div class="p-4 space-y-3">
            <div class="text-xs uppercase mb-1" style="color: var(--fg-muted);">基本信息</div>

            <div>
              <label class="text-xs block mb-1" style="color: var(--fg-secondary);" for="conn-name">名称</label>
              <input v-model="form.name" id="conn-name" class="w-full rounded px-3 py-1.5 text-sm focus:outline-none" style="background: var(--bg-elevated); border: 1px solid var(--border); color: var(--fg-primary);" placeholder="生产服务器" aria-required="true" />
            </div>

            <div class="flex gap-2">
              <div class="flex-1">
                <label class="text-xs block mb-1" style="color: var(--fg-secondary);" for="conn-host">主机</label>
                <input v-model="form.host" id="conn-host" class="w-full rounded px-3 py-1.5 text-sm focus:outline-none" style="background: var(--bg-elevated); border: 1px solid var(--border); color: var(--fg-primary);" placeholder="192.168.1.1" aria-required="true" />
              </div>
              <div class="w-20">
                <label class="text-xs block mb-1" style="color: var(--fg-secondary);" for="conn-port">端口</label>
                <input v-model.number="form.port" id="conn-port" type="number" min="1" max="65535" class="w-full rounded px-3 py-1.5 text-sm focus:outline-none" style="background: var(--bg-elevated); border: 1px solid var(--border); color: var(--fg-primary);" aria-required="true" />
              </div>
            </div>

            <div>
              <label class="text-xs block mb-1" style="color: var(--fg-secondary);" for="conn-username">用户名</label>
              <input v-model="form.username" id="conn-username" class="w-full rounded px-3 py-1.5 text-sm focus:outline-none" style="background: var(--bg-elevated); border: 1px solid var(--border); color: var(--fg-primary);" placeholder="root" aria-required="true" />
            </div>

            <div class="text-xs uppercase mb-1 mt-4" style="color: var(--fg-muted);">认证</div>

            <div class="flex gap-2" role="radiogroup" aria-label="认证方式">
              <button
                v-for="auth in authTypes"
                :key="auth.value"
                @click="form.authType = auth.value"
                class="flex-1 px-3 py-1.5 text-sm rounded border"
                :style="form.authType === auth.value ? 'background: var(--accent-hover); border-color: var(--accent); color: var(--accent);' : 'background: var(--bg-elevated); border-color: var(--border); color: var(--fg-muted);'"
                :aria-pressed="form.authType === auth.value"
                :aria-label="auth.label"
              >{{ auth.label }}</button>
            </div>

            <div v-if="form.authType === 'password'">
              <label class="text-xs block mb-1" style="color: var(--fg-secondary);" for="conn-password">密码</label>
              <div class="relative">
                <input v-model="form.password" id="conn-password" :type="showPassword ? 'text' : 'password'" class="w-full rounded px-3 py-1.5 text-sm focus:outline-none pr-8" style="background: var(--bg-elevated); border: 1px solid var(--border); color: var(--fg-primary);" aria-required="true" />
                <button @click="showPassword = !showPassword" type="button" :aria-label="showPassword ? '隐藏密码' : '显示密码'" class="absolute right-2 top-1/2 -translate-y-1/2 text-xs" style="color: var(--fg-muted);">{{ showPassword ? '🙈' : '👁️' }}</button>
              </div>
            </div>

            <div v-if="form.authType === 'key'">
              <label class="text-xs block mb-1" style="color: var(--fg-secondary);" for="conn-keypath">密钥路径</label>
              <input v-model="form.keyPath" id="conn-keypath" class="w-full rounded px-3 py-1.5 text-sm focus:outline-none" style="background: var(--bg-elevated); border: 1px solid var(--border); color: var(--fg-primary);" placeholder="~/.ssh/id_rsa" aria-required="true" />
            </div>
            <div v-if="form.authType === 'key'">
              <label class="text-xs block mb-1" style="color: var(--fg-secondary);" for="conn-passphrase">密钥密码（可选）</label>
              <input v-model="form.passphrase" id="conn-passphrase" type="password" class="w-full rounded px-3 py-1.5 text-sm focus:outline-none" style="background: var(--bg-elevated); border: 1px solid var(--border); color: var(--fg-primary);" />
            </div>

            <div class="rounded-lg px-3 py-2 text-xs" style="background: var(--bg-surface); color: var(--fg-muted); border: 1px solid var(--border-subtle);">
              首次连接服务器时会显示主机密钥算法与 SHA256 指纹，确认后才写入 known_hosts。密钥变化时会拒绝连接。
            </div>

            <div class="text-xs uppercase mb-1 mt-4" style="color: var(--fg-muted);">分组 & 标签</div>
            <div class="flex gap-2">
              <input v-model="form.group" class="flex-1 rounded px-3 py-1.5 text-sm focus:outline-none" style="background: var(--bg-elevated); border: 1px solid var(--border); color: var(--fg-primary);" placeholder="默认" />
              <select v-model="form.icon" class="rounded px-2 text-sm" style="background: var(--bg-elevated); border: 1px solid var(--border); color: var(--fg-primary);">
                <option value="🖥️">🖥️</option>
                <option value="☁️">☁️</option>
                <option value="🏠">🏠</option>
                <option value="🔒">🔒</option>
                <option value="🌐">🌐</option>
                <option value="💾">💾</option>
              </select>
            </div>
            <div>
              <label class="text-xs text-gray-400 block mb-1">标签颜色</label>
              <div class="flex gap-2">
                <button v-for="c in tagColors" :key="c.value" @click="form.color = c.value"
                  class="w-6 h-6 rounded-full border-2 transition-all" :class="form.color === c.value ? 'border-white scale-110' : 'border-transparent'"
                  :style="{ background: c.hex }" :title="c.label" />
              </div>
            </div>
            <div>
              <label class="text-xs block mb-1" style="color: var(--fg-secondary);">标签（逗号分隔）</label>
              <input v-model="tagsInput" class="w-full rounded px-3 py-1.5 text-sm focus:outline-none" style="background: var(--bg-elevated); border: 1px solid var(--border); color: var(--fg-primary);" placeholder="生产, Web, 数据库" />
            </div>
          </div>

          <div v-if="testResult" class="px-4 pb-2">
            <div class="text-xs px-3 py-2 rounded" :class="testResult.success ? 'bg-green-900/50 text-green-300' : 'bg-red-900/50 text-red-300'">
              {{ testResult.message }}
            </div>
          </div>

          <div class="p-4 flex justify-between" style="border-top: 1px solid var(--border-subtle);">
            <button @click="testConnection" :disabled="testing || !form.host || !form.username" type="button" aria-label="测试连接" class="px-4 py-1.5 text-sm disabled:opacity-50" style="color: var(--fg-muted);">
              {{ testing ? '测试中...' : '🔗 测试连接' }}
            </button>
            <div class="flex gap-2">
              <button @click="$emit('close')" type="button" aria-label="取消" class="px-4 py-1.5 text-sm" style="color: var(--fg-muted);">取消</button>
              <button @click="save" :disabled="!canSave" type="button" aria-label="保存连接" class="px-4 py-1.5 text-sm rounded text-white disabled:opacity-50" style="background: var(--accent);">保存</button>
            </div>
          </div>
        </div>
      </Transition>
    </div>
  </Transition>
</template>

<script setup>
import { computed, reactive, ref, onMounted } from 'vue'
import { invoke } from '../utils/tauri.js'
import { loadCredential } from '../utils/credentials.js'
import { hostKeyTarget, parseHostKeyError } from '../utils/ssh-host-key.js'

const props = defineProps({ editing: Object })
const emit = defineEmits(['save', 'close'])

const showPassword = ref(false)
const testing = ref(false)
const testResult = ref(null)

const tagColors = [
  { value: 'red', hex: '#ef4444', label: '生产' },
  { value: 'yellow', hex: '#eab308', label: '测试' },
  { value: 'green', hex: '#22c55e', label: '开发' },
  { value: 'blue', hex: '#3b82f6', label: '一般' },
  { value: 'purple', hex: '#a855f7', label: '特殊' },
  { value: '', hex: '#6b7280', label: '无' },
]

const tagsInput = ref((props.editing?.tags || []).join(', '))

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
  password: '',
  keyPath: props.editing?.keyPath || '',
  passphrase: '',
  group: props.editing?.group || '默认',
  icon: props.editing?.icon || '🖥️',
  color: props.editing?.color || '',
})

const canSave = computed(() => {
  if (!form.name || !form.host || !form.username || !form.port) return false
  if (form.authType === 'password') return !!form.password || !!props.editing?.id
  return !!form.keyPath
})

onMounted(async () => {
  if (!props.editing?.id) return
  const secret = await loadCredential(props.editing.id)
  if (props.editing.authType === 'password') form.password = secret
  if (props.editing.authType === 'key') form.passphrase = secret
})

function hostKeyTestMessage(detail) {
  const target = hostKeyTarget(detail)
  if (detail.code === 'HOST_KEY_UNKNOWN') {
    return `首次连接 ${target}，请保存后从终端连接并确认指纹：${detail.fingerprint}`
  }
  return `${target} 主机密钥已变化，已拒绝连接。当前指纹：${detail.fingerprint}`
}

async function testConnection() {
  testing.value = true
  testResult.value = null

  if (!form.host || !form.username) {
    testResult.value = { success: false, message: '✗ 请填写主机和用户名' }
    testing.value = false
    return
  }
  if (form.authType === 'password' && !form.password) {
    testResult.value = { success: false, message: '✗ 请输入密码' }
    testing.value = false
    return
  }
  if (form.authType === 'key' && !form.keyPath) {
    testResult.value = { success: false, message: '✗ 请选择或填写 SSH 密钥路径' }
    testing.value = false
    return
  }

  try {
    let sid
    if (form.authType === 'key') {
      sid = await invoke('ssh_connect_key', {
        host: form.host,
        port: form.port || 22,
        username: form.username,
        keyPath: form.keyPath,
        passphrase: form.passphrase,
        trustNewHostKey: false,
      })
    } else {
      sid = await invoke('ssh_connect', {
        host: form.host,
        port: form.port || 22,
        username: form.username,
        password: form.password,
        trustNewHostKey: false,
      })
    }
    testResult.value = { success: true, message: '✓ 连接成功！' }
    await invoke('ssh_disconnect', { sessionId: sid }).catch(() => {})
  } catch (err) {
    const hostKeyError = parseHostKeyError(err)
    testResult.value = {
      success: false,
      message: hostKeyError ? hostKeyTestMessage(hostKeyError) : `✗ 连接失败: ${err}`,
    }
  } finally {
    testing.value = false
  }
}

function save() {
  if (!canSave.value) return
  const tags = tagsInput.value.split(',').map(t => t.trim()).filter(Boolean)
  emit('save', { ...form, tags })
}
</script>
