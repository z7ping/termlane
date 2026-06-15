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
        <!-- 基本信息 -->
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
            <input v-model.number="form.port" id="conn-port" type="number" class="w-full rounded px-3 py-1.5 text-sm focus:outline-none" style="background: var(--bg-elevated); border: 1px solid var(--border); color: var(--fg-primary);" aria-required="true" />
          </div>
        </div>

        <div>
          <label class="text-xs block mb-1" style="color: var(--fg-secondary);" for="conn-username">用户名</label>
          <input v-model="form.username" id="conn-username" class="w-full rounded px-3 py-1.5 text-sm focus:outline-none" style="background: var(--bg-elevated); border: 1px solid var(--border); color: var(--fg-primary);" placeholder="root" aria-required="true" />
        </div>

        <!-- 认证 -->
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

        <!-- 代理跳板 -->
        <div class="text-xs uppercase mb-1 mt-4" style="color: var(--fg-muted);">代理跳板（可选）</div>

        <div class="flex items-center justify-between">
          <div>
            <div class="text-sm" style="color: var(--fg-primary);">启用跳板机</div>
            <div class="text-xs" style="color: var(--fg-muted);">通过中间服务器连接</div>
          </div>
          <button
            @click="form.useJumpHost = !form.useJumpHost"
            type="button"
            role="switch"
            :aria-checked="form.useJumpHost"
            aria-label="启用跳板机"
            class="w-10 h-5 rounded-full relative transition-colors"
            :style="form.useJumpHost ? 'background: var(--accent);' : 'background: var(--border);'"
          >
            <span class="absolute top-0.5 w-4 h-4 rounded-full bg-white transition-all" :class="form.useJumpHost ? 'left-5' : 'left-0.5'" />
          </button>
        </div>

        <template v-if="form.useJumpHost">
          <div class="flex gap-2">
            <div class="flex-1">
              <label class="text-xs block mb-1" style="color: var(--fg-secondary);">跳板机主机</label>
              <input v-model="form.jumpHost" class="w-full rounded px-3 py-1.5 text-sm focus:outline-none" style="background: var(--bg-elevated); border: 1px solid var(--border); color: var(--fg-primary);" placeholder="jump.example.com" />
            </div>
            <div class="w-20">
              <label class="text-xs block mb-1" style="color: var(--fg-secondary);">端口</label>
              <input v-model.number="form.jumpPort" type="number" class="w-full rounded px-3 py-1.5 text-sm focus:outline-none" style="background: var(--bg-elevated); border: 1px solid var(--border); color: var(--fg-primary);" />
            </div>
          </div>
          <div>
            <label class="text-xs block mb-1" style="color: var(--fg-secondary);">跳板机用户名</label>
            <input v-model="form.jumpUsername" class="w-full rounded px-3 py-1.5 text-sm focus:outline-none" style="background: var(--bg-elevated); border: 1px solid var(--border); color: var(--fg-primary);" placeholder="root" />
          </div>
          <div>
            <label class="text-xs block mb-1" style="color: var(--fg-secondary);">跳板机密码</label>
            <input v-model="form.jumpPassword" type="password" class="w-full rounded px-3 py-1.5 text-sm focus:outline-none" style="background: var(--bg-elevated); border: 1px solid var(--border); color: var(--fg-primary);" />
          </div>
        </template>

        <!-- 分组 + 标签颜色 -->
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

      <!-- 测试结果 -->
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
          <button @click="save" :disabled="!form.name || !form.host || !form.username" type="button" aria-label="保存连接" class="px-4 py-1.5 text-sm rounded text-white disabled:opacity-50" style="background: var(--accent);">保存</button>
        </div>
      </div>
    </div>
    </Transition>
  </div>
  </Transition>
</template>

<script setup>
import { reactive, ref, onMounted } from 'vue'
import { invoke } from '../utils/tauri.js'

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

// Load password from keyring on mount when editing
onMounted(async () => {
  if (props.editing?.authType === 'password' && props.editing?.id) {
    try {
      const pwd = await invoke('keyring_load_password', { connId: props.editing.id })
      if (pwd) form.password = pwd
    } catch (e) { /* keyring not available */ }
  }
})

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
  color: props.editing?.color || '',
  // 代理跳板
  useJumpHost: props.editing?.useJumpHost || false,
  jumpHost: props.editing?.jumpHost || '',
  jumpPort: props.editing?.jumpPort || 22,
  jumpUsername: props.editing?.jumpUsername || '',
  jumpPassword: props.editing?.jumpPassword || '',
})

async function testConnection() {
    testing.value = true
    testResult.value = null
    // 校验必填字段
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
    try {
      let sid
      // 如果启用了跳板机，使用 ssh_connect_jump
      if (form.useJumpHost) {
        sid = await invoke('ssh_connect_jump', {
          jumpHost: form.jumpHost,
          jumpPort: form.jumpPort || 22,
          jumpUser: form.jumpUsername,
          jumpPass: form.jumpPassword,
          targetHost: form.host,
          targetPort: form.port || 22,
          targetUser: form.username,
          targetPass: form.password || '',
        })
      } else if (form.authType === 'key') {
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
      testResult.value = { success: true, message: `✓ 连接成功！` }
      await invoke('ssh_disconnect', { sessionId: sid }).catch(() => {})
    } catch (err) {
      testResult.value = { success: false, message: `✗ 连接失败: ${err}` }
    } finally {
      testing.value = false
    }
}

function save() {
  if (!form.name || !form.host || !form.username) return
  const tags = tagsInput.value.split(',').map(t => t.trim()).filter(Boolean)
  emit('save', { ...form, tags })
}
</script>
