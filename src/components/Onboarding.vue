<template>
  <Transition name="onboarding-fade">
    <div v-if="show" class="onboarding-overlay" @click.self="skip">
      <Transition name="onboarding-scale" appear>
        <div class="onboarding-card" v-if="show">
          <!-- Close button -->
          <button class="onboarding-close" @click="skip" title="跳过引导">✕</button>

          <!-- Step Content -->
          <div class="onboarding-body">
            <!-- Step 1: Welcome -->
            <div v-if="current === 0" class="onboarding-step">
              <div class="onboarding-illustration welcome-illustration">
                <div class="logo-glow">⌨️</div>
                <div class="logo-orbit">
                  <span class="orbit-dot" style="--i:0">🖥️</span>
                  <span class="orbit-dot" style="--i:1">🔗</span>
                  <span class="orbit-dot" style="--i:2">📁</span>
                  <span class="orbit-dot" style="--i:3">⚡</span>
                </div>
              </div>
              <h2 class="onboarding-title">欢迎使用 <span class="text-accent">XTerminal Pro</span></h2>
              <p class="onboarding-desc">轻量、快速、强大的 SSH 终端工具。基于 Tauri 构建，内存占用仅 30MB，让远程管理从未如此轻松。</p>
              <div class="feature-chips">
                <span class="chip">🚀 极速启动</span>
                <span class="chip">🔒 安全连接</span>
                <span class="chip">📦 轻量 30MB</span>
              </div>
            </div>

            <!-- Step 2: SSH Connection -->
            <div v-if="current === 1" class="onboarding-step">
              <div class="onboarding-illustration">
                <div class="ssh-demo">
                  <div class="ssh-sidebar">
                    <div class="ssh-add-btn pulsing">＋</div>
                    <div class="ssh-item active">🖥️ 生产服</div>
                    <div class="ssh-item">🖥️ 测试服</div>
                  </div>
                  <div class="ssh-arrow">→</div>
                  <div class="ssh-terminal-preview">
                    <div class="terminal-header">
                      <span class="dot red"></span>
                      <span class="dot yellow"></span>
                      <span class="dot green"></span>
                    </div>
                    <div class="terminal-body">
                      <span class="prompt">$</span> ssh user@server
                    </div>
                  </div>
                </div>
              </div>
              <h2 class="onboarding-title">添加 SSH 连接</h2>
              <p class="onboarding-desc">点击左侧侧边栏的 <kbd class="kbd-inline">＋</kbd> 按钮，添加你的第一个服务器连接。支持密码和密钥两种认证方式。</p>
              <div class="step-tip">
                <span class="tip-icon">💡</span>
                <span>支持 SSH 密钥、密码、跳板机等高级配置</span>
              </div>
            </div>

            <!-- Step 3: Quick Commands -->
            <div v-if="current === 2" class="onboarding-step">
              <div class="onboarding-illustration">
                <div class="quick-cmd-grid">
                  <div class="cmd-card" v-for="cmd in quickCommands" :key="cmd.name">
                    <span class="cmd-icon">{{ cmd.icon }}</span>
                    <span class="cmd-name">{{ cmd.name }}</span>
                    <code class="cmd-code">{{ cmd.code }}</code>
                  </div>
                </div>
              </div>
              <h2 class="onboarding-title">快捷命令</h2>
              <p class="onboarding-desc">底部工具栏提供常用系统命令快捷入口，一键查看系统负载、磁盘、内存等信息。</p>
              <div class="cmd-demo-bar">
                <span v-for="cmd in demoCmds" :key="cmd" class="demo-cmd-item" @click="">{{ cmd }}</span>
              </div>
            </div>

            <!-- Step 4: SFTP -->
            <div v-if="current === 3" class="onboarding-step">
              <div class="onboarding-illustration">
                <div class="sftp-preview">
                  <div class="sftp-toolbar">
                    <span class="sftp-tab active">📁 文件</span>
                    <span class="sftp-tab">⌨️ 终端</span>
                  </div>
                  <div class="sftp-panels">
                    <div class="sftp-panel">
                      <div class="sftp-item folder">📁 /home</div>
                      <div class="sftp-item folder">📁 /var</div>
                      <div class="sftp-item file">📄 config.yml</div>
                      <div class="sftp-item file">📦 backup.tar</div>
                    </div>
                    <div class="sftp-divider">⟷</div>
                    <div class="sftp-panel">
                      <div class="sftp-item folder">📁 Documents</div>
                      <div class="sftp-item folder">📁 Downloads</div>
                      <div class="sftp-item file">📄 notes.txt</div>
                    </div>
                  </div>
                  <div class="sftp-status">
                    <span>📂 双栏文件管理</span>
                    <span class="drag-hint">拖拽上传 ↓  拖拽下载 ↑</span>
                  </div>
                </div>
              </div>
              <h2 class="onboarding-title">SFTP 文件管理</h2>
              <p class="onboarding-desc">点击顶部视图切换到「文件」模式，双栏文件管理器让上传下载变得轻松直观。支持拖拽、批量操作。</p>
            </div>

            <!-- Step 5: Done -->
            <div v-if="current === 4" class="onboarding-step">
              <div class="onboarding-illustration done-illustration">
                <div class="rocket-burst">🚀</div>
                <div class="confetti">
                  <span v-for="n in 8" :key="n" class="confetti-piece" :style="{ '--n': n }">✦</span>
                </div>
              </div>
              <h2 class="onboarding-title">一切就绪！</h2>
              <p class="onboarding-desc">现在开始使用 XTerminal Pro 管理你的服务器吧。</p>
              <div class="done-shortcuts">
                <div class="shortcut-row">
                  <kbd>Ctrl+T</kbd>
                  <span>新建终端</span>
                </div>
                <div class="shortcut-row">
                  <kbd>Ctrl+B</kbd>
                  <span>切换侧栏</span>
                </div>
                <div class="shortcut-row">
                  <kbd>Ctrl+Shift+F</kbd>
                  <span>搜索</span>
                </div>
                <div class="shortcut-row">
                  <kbd>?</kbd>
                  <span>快捷键帮助</span>
                </div>
              </div>
            </div>
          </div>

          <!-- Progress & Navigation -->
          <div class="onboarding-footer">
            <!-- Progress bar -->
            <div class="progress-track">
              <div class="progress-fill" :style="{ width: ((current + 1) / steps.length * 100) + '%' }"></div>
            </div>

            <div class="onboarding-nav">
              <button v-if="current > 0" class="btn-nav btn-prev" @click="current--">
                ← 上一步
              </button>
              <div v-else class="nav-spacer"></div>

              <!-- Step indicator -->
              <div class="step-indicator">
                {{ current + 1 }} / {{ steps.length }}
              </div>

              <button v-if="current < steps.length - 1" class="btn-nav btn-next" @click="current++">
                下一步 →
              </button>
              <button v-else class="btn-nav btn-finish" @click="finish">
                开始使用 🚀
              </button>
            </div>

            <button class="btn-skip" @click="skip">跳过引导</button>
          </div>
        </div>
      </Transition>
    </div>
  </Transition>
</template>

<script setup>
import { ref, onMounted } from 'vue'

const show = ref(false)
const current = ref(0)

const steps = [
  { title: '欢迎' },
  { title: 'SSH 连接' },
  { title: '快捷命令' },
  { title: 'SFTP' },
  { title: '开始使用' },
]

const quickCommands = [
  { icon: '📊', name: '系统负载', code: 'top' },
  { icon: '💾', name: '磁盘空间', code: 'df -h' },
  { icon: '🧠', name: '内存使用', code: 'free -h' },
  { icon: '🌐', name: '网络连接', code: 'ss -tlnp' },
]

const demoCmds = ['top', 'df -h', 'free -h', 'uptime', 'ss -tlnp', 'ps aux']

onMounted(() => {
  const onboarded = localStorage.getItem('xterminal_onboarded')
  if (!onboarded) {
    show.value = true
  }
})

function finish() {
  localStorage.setItem('xterminal_onboarded', 'true')
  show.value = false
}

function skip() {
  localStorage.setItem('xterminal_onboarded', 'true')
  show.value = false
}

// expose for parent
defineExpose({ show, finish })
</script>

<style scoped>
/* ─── Overlay ─── */
.onboarding-overlay {
  position: fixed;
  inset: 0;
  z-index: 100;
  background: oklch(0 0 0 / 0.75);
  backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
}

/* ─── Card ─── */
.onboarding-card {
  width: 560px;
  max-height: 90vh;
  background: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  box-shadow: 0 24px 80px oklch(0 0 0 / 0.5);
  overflow: hidden;
  display: flex;
  flex-direction: column;
  position: relative;
}

/* ─── Close ─── */
.onboarding-close {
  position: absolute;
  top: 12px;
  right: 12px;
  z-index: 2;
  width: 28px;
  height: 28px;
  border-radius: 50%;
  border: none;
  background: var(--bg-hover);
  color: var(--fg-muted);
  font-size: 12px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all var(--transition);
}
.onboarding-close:hover {
  background: var(--danger);
  color: white;
}

/* ─── Body ─── */
.onboarding-body {
  padding: 40px 36px 24px;
  flex: 1;
  overflow-y: auto;
  min-height: 320px;
}

.onboarding-step {
  animation: stepIn 350ms cubic-bezier(0.16, 1, 0.3, 1);
}

@keyframes stepIn {
  from { opacity: 0; transform: translateY(12px) scale(0.97); }
  to { opacity: 1; transform: translateY(0) scale(1); }
}

/* ─── Title & Desc ─── */
.onboarding-title {
  font-size: 22px;
  font-weight: 600;
  color: var(--fg-primary);
  margin: 16px 0 8px;
  text-align: center;
}

.text-accent {
  color: var(--accent);
}

.onboarding-desc {
  font-size: 14px;
  line-height: 1.7;
  color: var(--fg-secondary);
  text-align: center;
  max-width: 440px;
  margin: 0 auto;
}

/* ─── Illustrations ─── */
.onboarding-illustration {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 140px;
  margin-bottom: 8px;
}

/* Step 1: Welcome logo */
.welcome-illustration {
  position: relative;
  width: 160px;
  height: 160px;
  margin: 0 auto;
}

.logo-glow {
  font-size: 56px;
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  animation: logoPulse 3s ease-in-out infinite;
  filter: drop-shadow(0 0 20px oklch(0.65 0.18 250 / 0.4));
}

@keyframes logoPulse {
  0%, 100% { transform: translate(-50%, -50%) scale(1); }
  50% { transform: translate(-50%, -50%) scale(1.08); }
}

.logo-orbit {
  position: absolute;
  inset: 0;
  animation: orbitSpin 12s linear infinite;
}

@keyframes orbitSpin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.orbit-dot {
  position: absolute;
  font-size: 20px;
  --angle: calc(var(--i) * 90deg);
  top: 50%;
  left: 50%;
  transform: rotate(var(--angle)) translateY(-70px) rotate(calc(-1 * var(--angle)));
  animation: orbitSpin 12s linear infinite reverse;
}

/* Step 2: SSH demo */
.ssh-demo {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 16px;
  background: var(--bg-elevated);
  border-radius: var(--radius);
  border: 1px solid var(--border-subtle);
}

.ssh-sidebar {
  display: flex;
  flex-direction: column;
  gap: 6px;
  min-width: 100px;
}

.ssh-add-btn {
  width: 100%;
  padding: 6px;
  text-align: center;
  background: var(--accent);
  color: white;
  border-radius: var(--radius-sm);
  font-size: 14px;
  font-weight: 600;
}

.ssh-add-btn.pulsing {
  animation: btnPulse 2s ease-in-out infinite;
}

@keyframes btnPulse {
  0%, 100% { box-shadow: 0 0 0 0 oklch(0.65 0.18 250 / 0.4); }
  50% { box-shadow: 0 0 0 8px oklch(0.65 0.18 250 / 0); }
}

.ssh-item {
  padding: 4px 8px;
  font-size: 11px;
  color: var(--fg-secondary);
  background: var(--bg-hover);
  border-radius: var(--radius-sm);
}
.ssh-item.active {
  background: var(--accent);
  color: white;
}

.ssh-arrow {
  font-size: 20px;
  color: var(--fg-muted);
}

.ssh-terminal-preview {
  flex: 1;
  background: oklch(0.12 0 0);
  border-radius: var(--radius-sm);
  overflow: hidden;
  border: 1px solid var(--border-subtle);
}

.terminal-header {
  display: flex;
  gap: 4px;
  padding: 6px 10px;
  background: oklch(0.16 0 0);
}
.dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
}
.dot.red { background: #ff5f57; }
.dot.yellow { background: #ffbd2e; }
.dot.green { background: #28c940; }

.terminal-body {
  padding: 10px;
  font-family: var(--font-mono);
  font-size: 12px;
  color: var(--fg-primary);
}
.prompt {
  color: var(--success);
  margin-right: 6px;
}

/* Step 3: Quick commands */
.quick-cmd-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 8px;
  width: 100%;
  max-width: 380px;
}

.cmd-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  padding: 12px 8px;
  background: var(--bg-elevated);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius);
  transition: all var(--transition);
}

.cmd-icon {
  font-size: 20px;
}

.cmd-name {
  font-size: 12px;
  color: var(--fg-secondary);
  font-weight: 500;
}

.cmd-code {
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--accent);
  background: var(--bg-hover);
  padding: 2px 8px;
  border-radius: 4px;
}

.cmd-demo-bar {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  justify-content: center;
  margin-top: 12px;
}

.demo-cmd-item {
  font-family: var(--font-mono);
  font-size: 11px;
  padding: 4px 10px;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: 999px;
  color: var(--fg-secondary);
  cursor: default;
  transition: all var(--transition);
}
.demo-cmd-item:hover {
  border-color: var(--accent);
  color: var(--accent);
}

/* Step 4: SFTP */
.sftp-preview {
  width: 100%;
  max-width: 420px;
  background: var(--bg-elevated);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius);
  overflow: hidden;
}

.sftp-toolbar {
  display: flex;
  gap: 4px;
  padding: 6px 10px;
  background: var(--bg-hover);
  border-bottom: 1px solid var(--border-subtle);
}

.sftp-tab {
  font-size: 11px;
  padding: 3px 10px;
  border-radius: var(--radius-sm);
  color: var(--fg-muted);
}
.sftp-tab.active {
  background: var(--accent);
  color: white;
}

.sftp-panels {
  display: flex;
  align-items: stretch;
  min-height: 100px;
}

.sftp-panel {
  flex: 1;
  padding: 8px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.sftp-divider {
  display: flex;
  align-items: center;
  color: var(--fg-muted);
  font-size: 12px;
  padding: 0 2px;
}

.sftp-item {
  font-size: 11px;
  padding: 3px 8px;
  border-radius: 4px;
  color: var(--fg-secondary);
  background: var(--bg-hover);
}

.sftp-status {
  display: flex;
  justify-content: space-between;
  padding: 6px 10px;
  font-size: 10px;
  color: var(--fg-muted);
  background: var(--bg-hover);
  border-top: 1px solid var(--border-subtle);
}

.drag-hint {
  font-family: var(--font-mono);
}

/* Step 5: Done */
.done-illustration {
  position: relative;
  height: 120px;
}

.rocket-burst {
  font-size: 64px;
  animation: rocketBounce 1.5s ease-in-out infinite;
  filter: drop-shadow(0 4px 20px oklch(0.65 0.18 250 / 0.3));
}

@keyframes rocketBounce {
  0%, 100% { transform: translateY(0); }
  50% { transform: translateY(-12px); }
}

.confetti {
  position: absolute;
  inset: 0;
}

.confetti-piece {
  position: absolute;
  top: 50%;
  left: 50%;
  font-size: 14px;
  color: var(--accent);
  animation: confettiFly 2s ease-out infinite;
  animation-delay: calc(var(--n) * 0.2s);
}

@keyframes confettiFly {
  0% {
    opacity: 1;
    transform: translate(0, 0) rotate(0deg);
  }
  100% {
    opacity: 0;
    transform: translate(
      calc(cos(var(--n) * 45deg) * 80px),
      calc(sin(var(--n) * 45deg) * 80px - 30px)
    ) rotate(360deg);
  }
}

.done-shortcuts {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 8px;
  margin-top: 16px;
  max-width: 340px;
  margin-left: auto;
  margin-right: auto;
}

.shortcut-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 10px;
  background: var(--bg-elevated);
  border-radius: var(--radius-sm);
  border: 1px solid var(--border-subtle);
}

.shortcut-row kbd {
  font-family: var(--font-mono);
  font-size: 11px;
  padding: 2px 6px;
  background: var(--bg-hover);
  border: 1px solid var(--border);
  border-radius: 4px;
  color: var(--accent);
  white-space: nowrap;
}

.shortcut-row span {
  font-size: 12px;
  color: var(--fg-secondary);
}

/* ─── Chips & Tips ─── */
.feature-chips {
  display: flex;
  gap: 8px;
  justify-content: center;
  margin-top: 16px;
}

.chip {
  font-size: 12px;
  padding: 4px 12px;
  background: var(--bg-elevated);
  border: 1px solid var(--border-subtle);
  border-radius: 999px;
  color: var(--fg-secondary);
}

.step-tip {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-top: 12px;
  padding: 8px 12px;
  background: oklch(0.65 0.18 250 / 0.08);
  border: 1px solid oklch(0.65 0.18 250 / 0.15);
  border-radius: var(--radius);
  font-size: 12px;
  color: var(--fg-secondary);
}

.tip-icon {
  font-size: 14px;
}

.kbd-inline {
  font-family: var(--font-mono);
  font-size: 12px;
  padding: 1px 6px;
  background: var(--bg-hover);
  border: 1px solid var(--border);
  border-radius: 4px;
  color: var(--accent);
}

/* ─── Footer ─── */
.onboarding-footer {
  padding: 0 36px 28px;
}

/* Progress */
.progress-track {
  height: 3px;
  background: var(--bg-hover);
  border-radius: 2px;
  overflow: hidden;
  margin-bottom: 20px;
}

.progress-fill {
  height: 100%;
  background: var(--accent);
  border-radius: 2px;
  transition: width 400ms cubic-bezier(0.16, 1, 0.3, 1);
}

/* Nav */
.onboarding-nav {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.nav-spacer {
  width: 80px;
}

.step-indicator {
  font-size: 12px;
  color: var(--fg-muted);
  font-variant-numeric: tabular-nums;
}

.btn-nav {
  padding: 8px 20px;
  border: none;
  border-radius: var(--radius);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all var(--transition);
}

.btn-prev {
  background: var(--bg-hover);
  color: var(--fg-secondary);
}
.btn-prev:hover {
  background: var(--border);
  color: var(--fg-primary);
}

.btn-next {
  background: var(--accent);
  color: white;
}
.btn-next:hover {
  background: var(--accent-hover);
}

.btn-finish {
  background: var(--success);
  color: white;
  font-weight: 600;
}
.btn-finish:hover {
  filter: brightness(1.1);
}

.btn-skip {
  display: block;
  margin: 12px auto 0;
  background: none;
  border: none;
  font-size: 12px;
  color: var(--fg-muted);
  cursor: pointer;
  padding: 4px 12px;
  border-radius: var(--radius-sm);
  transition: all var(--transition);
}
.btn-skip:hover {
  color: var(--fg-secondary);
  background: var(--bg-hover);
}

/* ─── Transitions ─── */
.onboarding-fade-enter-active,
.onboarding-fade-leave-active {
  transition: opacity 250ms ease;
}
.onboarding-fade-enter-from,
.onboarding-fade-leave-to {
  opacity: 0;
}

.onboarding-scale-enter-active {
  transition: all 350ms cubic-bezier(0.16, 1, 0.3, 1);
}
.onboarding-scale-leave-active {
  transition: all 200ms ease-in;
}
.onboarding-scale-enter-from {
  opacity: 0;
  transform: scale(0.92) translateY(20px);
}
.onboarding-scale-leave-to {
  opacity: 0;
  transform: scale(0.95);
}
</style>
