<template>
  <Transition name="fade">
    <div class="dialog-backdrop" role="dialog" :aria-label="editing ? '编辑 SSH 连接' : '新建 SSH 连接'" aria-modal="true">
      <Transition name="slide-up">
        <div class="dialog-card">
          <header class="dialog-header">
            <h3>{{ editing ? '编辑连接' : '新建连接' }}</h3>
            <button type="button" class="icon-button" aria-label="关闭对话框" @click="$emit('close')">
              <X :size="15" :stroke-width="1.8" />
            </button>
          </header>

          <div class="dialog-body">
            <section class="form-section">
              <div class="section-title">基本信息</div>

              <label class="field">
                <span>名称</span>
                <input v-model="form.name" class="input" placeholder="生产服务器" autocomplete="off" />
              </label>

              <div class="grid grid-cols-[1fr_88px] gap-2">
                <label class="field">
                  <span>主机</span>
                  <input v-model="form.host" class="input" placeholder="192.168.1.1" autocomplete="off" />
                </label>
                <label class="field">
                  <span>端口</span>
                  <input v-model.number="form.port" class="input" type="number" min="1" max="65535" />
                </label>
              </div>

              <label class="field">
                <span>用户名</span>
                <input v-model="form.username" class="input" placeholder="root" autocomplete="username" />
              </label>
            </section>

            <section class="form-section">
              <div class="section-title">认证</div>
              <div class="auth-switch" role="radiogroup" aria-label="认证方式">
                <button
                  v-for="auth in authTypes"
                  :key="auth.value"
                  type="button"
                  :class="{ active: form.authType === auth.value }"
                  :aria-pressed="form.authType === auth.value"
                  @click="form.authType = auth.value"
                >
                  {{ auth.label }}
                </button>
              </div>

              <label v-if="form.authType === 'password'" class="field">
                <span>密码</span>
                <div class="password-wrap">
                  <input
                    v-model="form.password"
                    class="input pr-9"
                    :type="showPassword ? 'text' : 'password'"
                    autocomplete="current-password"
                  />
                  <button
                    type="button"
                    class="password-toggle"
                    :aria-label="showPassword ? '隐藏密码' : '显示密码'"
                    @click="showPassword = !showPassword"
                  >
                    <EyeOff v-if="showPassword" :size="14" :stroke-width="1.8" />
                    <Eye v-else :size="14" :stroke-width="1.8" />
                  </button>
                </div>
              </label>

              <template v-else>
                <label class="field">
                  <span>密钥路径</span>
                  <input v-model="form.keyPath" class="input" placeholder="~/.ssh/id_ed25519" autocomplete="off" />
                </label>
                <label class="field">
                  <span>密钥密码 <small>可选</small></span>
                  <input v-model="form.passphrase" class="input" type="password" autocomplete="off" />
                </label>
              </template>

              <div class="security-note">
                <ShieldCheck :size="16" :stroke-width="1.8" />
                <span>首次连接会显示主机密钥算法和 SHA256 指纹，确认后才写入 known_hosts；密钥变化会拒绝连接。</span>
              </div>
            </section>

            <section class="form-section">
              <div class="section-title">整理</div>
              <label class="field">
                <span>分组</span>
                <input v-model="form.group" class="input" placeholder="默认 / 生产 / Web" autocomplete="off" />
              </label>

              <label class="field">
                <span>标签 <small>逗号分隔</small></span>
                <input v-model="tagsInput" class="input" placeholder="生产, Web, 数据库" autocomplete="off" />
              </label>

              <div class="field">
                <span>标记颜色</span>
                <div class="color-options" role="radiogroup" aria-label="连接标记颜色">
                  <button
                    v-for="color in tagColors"
                    :key="color.value || 'none'"
                    type="button"
                    class="color-button"
                    :class="{ active: form.color === color.value }"
                    :style="{ '--tag-color': color.hex }"
                    :title="color.label"
                    :aria-label="color.label"
                    :aria-pressed="form.color === color.value"
                    @click="form.color = color.value"
                  />
                </div>
              </div>
            </section>

            <div v-if="testResult" class="test-result" :class="testResult.success ? 'success' : 'error'">
              {{ testResult.message }}
            </div>
          </div>

          <footer class="dialog-footer">
            <button
              type="button"
              class="test-button"
              :disabled="testing || !form.host || !form.username"
              @click="testConnection"
            >
              <Cable :size="14" :stroke-width="1.8" />
              <span>{{ testing ? '测试中...' : '测试连接' }}</span>
            </button>

            <div class="flex gap-2">
              <button type="button" class="secondary-button" @click="$emit('close')">取消</button>
              <button type="button" class="primary-button" :disabled="!canSave" @click="save">保存</button>
            </div>
          </footer>
        </div>
      </Transition>
    </div>
  </Transition>
</template>

<script setup>
import { computed, onMounted, reactive, ref } from 'vue'
import { Cable, Eye, EyeOff, ShieldCheck, X } from 'lucide-vue-next'
import { invoke } from '../utils/tauri.js'
import { loadCredential } from '../utils/credentials.js'
import { hostKeyTarget, parseHostKeyError } from '../utils/ssh-host-key.js'

const props = defineProps({ editing: Object })
const emit = defineEmits(['save', 'close'])

const showPassword = ref(false)
const testing = ref(false)
const testResult = ref(null)

const authTypes = [
  { value: 'password', label: '密码' },
  { value: 'key', label: 'SSH 密钥' },
]

const tagColors = [
  { value: 'red', hex: '#ef4444', label: '红色' },
  { value: 'yellow', hex: '#eab308', label: '黄色' },
  { value: 'green', hex: '#22c55e', label: '绿色' },
  { value: 'blue', hex: '#3b82f6', label: '蓝色' },
  { value: 'purple', hex: '#a855f7', label: '紫色' },
  { value: '', hex: '#6b7280', label: '无标记' },
]

const tagsInput = ref((props.editing?.tags || []).join(', '))

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
  color: props.editing?.color || '',
})

const canSave = computed(() => {
  if (!form.name.trim() || !form.host.trim() || !form.username.trim() || !form.port) return false
  if (form.authType === 'password') return !!form.password
  return !!form.keyPath.trim()
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
    return `首次连接 ${target}，请保存后从终端确认指纹：${detail.fingerprint}`
  }
  return `${target} 主机密钥已变化，已拒绝连接。当前指纹：${detail.fingerprint}`
}

async function testConnection() {
  testing.value = true
  testResult.value = null

  if (!form.host.trim() || !form.username.trim()) {
    testResult.value = { success: false, message: '请填写主机和用户名' }
    testing.value = false
    return
  }
  if (form.authType === 'password' && !form.password) {
    testResult.value = { success: false, message: '请输入密码' }
    testing.value = false
    return
  }
  if (form.authType === 'key' && !form.keyPath.trim()) {
    testResult.value = { success: false, message: '请输入 SSH 密钥路径' }
    testing.value = false
    return
  }

  try {
    let sessionId
    if (form.authType === 'key') {
      sessionId = await invoke('ssh_connect_key', {
        host: form.host,
        port: form.port || 22,
        username: form.username,
        keyPath: form.keyPath,
        passphrase: form.passphrase,
        trustNewHostKey: false,
      })
    } else {
      sessionId = await invoke('ssh_connect', {
        host: form.host,
        port: form.port || 22,
        username: form.username,
        password: form.password,
        trustNewHostKey: false,
      })
    }

    testResult.value = { success: true, message: '连接成功' }
    await invoke('ssh_disconnect', { sessionId }).catch(() => {})
  } catch (error) {
    const hostKeyError = parseHostKeyError(error)
    testResult.value = {
      success: false,
      message: hostKeyError ? hostKeyTestMessage(hostKeyError) : `连接失败：${error}`,
    }
  } finally {
    testing.value = false
  }
}

function save() {
  if (!canSave.value) return
  const tags = tagsInput.value.split(',').map(tag => tag.trim()).filter(Boolean)
  emit('save', { ...form, tags })
}
</script>

<style scoped>
.dialog-backdrop {
  position: fixed;
  inset: 0;
  z-index: 50;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
  background: rgba(0, 0, 0, 0.62);
  backdrop-filter: blur(4px);
}

.dialog-card {
  width: min(480px, 100%);
  max-height: min(760px, 92vh);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  border: 1px solid var(--border);
  border-radius: 12px;
  background: var(--bg-elevated);
  box-shadow: var(--shadow-lg);
}

.dialog-header,
.dialog-footer {
  display: flex;
  align-items: center;
  flex-shrink: 0;
  padding: 10px 12px;
}

.dialog-header {
  border-bottom: 1px solid var(--border-subtle);
}

.dialog-header h3 {
  margin: 0;
  color: var(--fg-primary);
  font-size: 13px;
  font-weight: 600;
}

.dialog-header .icon-button {
  margin-left: auto;
}

.dialog-body {
  display: flex;
  flex-direction: column;
  gap: 18px;
  padding: 14px;
  overflow-y: auto;
}

.form-section {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.section-title {
  color: var(--fg-muted);
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 5px;
  color: var(--fg-secondary);
  font-size: 11px;
}

.field small {
  color: var(--fg-muted);
  font-size: 10px;
}

.input {
  width: 100%;
  height: 31px;
  padding: 0 9px;
  border: 1px solid var(--border);
  border-radius: 6px;
  outline: none;
  background: var(--bg-base);
  color: var(--fg-primary);
  font-size: 12px;
  transition: border-color var(--transition-fast), background-color var(--transition-fast);
}

.input:focus {
  border-color: var(--accent);
}

.auth-switch {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 4px;
  padding: 3px;
  border-radius: 7px;
  background: var(--bg-base);
}

.auth-switch button {
  height: 29px;
  border: 0;
  border-radius: 5px;
  background: transparent;
  color: var(--fg-muted);
  font-size: 12px;
}

.auth-switch button:hover {
  color: var(--fg-primary);
}

.auth-switch button.active {
  background: var(--bg-elevated);
  color: var(--accent);
  box-shadow: var(--shadow-sm);
}

.password-wrap {
  position: relative;
}

.password-toggle {
  position: absolute;
  right: 3px;
  top: 50%;
  width: 28px;
  height: 26px;
  display: flex;
  align-items: center;
  justify-content: center;
  transform: translateY(-50%);
  border: 0;
  border-radius: 5px;
  background: transparent;
  color: var(--fg-muted);
}

.password-toggle:hover,
.icon-button:hover {
  background: var(--bg-hover);
  color: var(--fg-primary);
}

.security-note {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  padding: 9px 10px;
  border: 1px solid var(--border-subtle);
  border-radius: 7px;
  background: var(--bg-surface);
  color: var(--fg-muted);
  font-size: 11px;
  line-height: 1.5;
}

.security-note svg {
  flex-shrink: 0;
  margin-top: 1px;
  color: var(--success);
}

.color-options {
  display: flex;
  gap: 7px;
}

.color-button {
  width: 20px;
  height: 20px;
  border: 2px solid transparent;
  border-radius: 999px;
  background: var(--tag-color);
  box-shadow: 0 0 0 1px var(--border);
}

.color-button.active {
  border-color: var(--bg-elevated);
  box-shadow: 0 0 0 2px var(--accent);
}

.test-result {
  padding: 8px 10px;
  border-radius: 6px;
  font-size: 11px;
}

.test-result.success {
  background: color-mix(in srgb, var(--success) 12%, transparent);
  color: var(--success);
}

.test-result.error {
  background: color-mix(in srgb, var(--danger) 12%, transparent);
  color: var(--danger);
}

.dialog-footer {
  justify-content: space-between;
  gap: 12px;
  border-top: 1px solid var(--border-subtle);
}

.icon-button {
  width: 28px;
  height: 28px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 0;
  border-radius: 5px;
  background: transparent;
  color: var(--fg-muted);
}

.test-button,
.secondary-button,
.primary-button {
  height: 30px;
  border: 0;
  border-radius: 6px;
  font-size: 12px;
}

.test-button {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 0 9px;
  background: transparent;
  color: var(--fg-muted);
}

.test-button:hover:not(:disabled) {
  background: var(--bg-hover);
  color: var(--fg-primary);
}

.secondary-button,
.primary-button {
  padding: 0 12px;
}

.secondary-button {
  background: var(--bg-hover);
  color: var(--fg-secondary);
}

.primary-button {
  background: var(--accent);
  color: white;
}

button:disabled {
  cursor: not-allowed;
  opacity: 0.45;
}
</style>
