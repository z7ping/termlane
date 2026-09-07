<template>
  <Transition name="onboarding-fade">
    <div v-if="show" class="onboarding-backdrop" @click.self="skip">
      <div class="onboarding-dialog" role="dialog" aria-modal="true" aria-label="XTerminal Pro 首次使用引导" @keydown.esc="skip">
        <button type="button" class="close-button" aria-label="跳过引导" title="跳过引导" @click="skip">
          <X :size="15" :stroke-width="1.8" />
        </button>

        <div class="onboarding-content">
          <section v-if="current === 0" class="step-content">
            <div class="hero-icon"><Terminal :size="42" :stroke-width="1.35" /></div>
            <h2>欢迎使用 XTerminal Pro</h2>
            <p>一个专注日常 SSH 终端与远程文件管理的轻量桌面工具。</p>
            <div class="feature-grid">
              <div><TerminalSquare :size="17" /><span>真实 PTY 终端</span></div>
              <div><FolderOpen :size="17" /><span>远程文件管理</span></div>
              <div><ShieldCheck :size="17" /><span>主机指纹确认</span></div>
            </div>
          </section>

          <section v-else-if="current === 1" class="step-content">
            <div class="hero-icon"><Server :size="42" :stroke-width="1.35" /></div>
            <h2>添加 SSH 连接</h2>
            <p>点击左侧连接列表的“+”，填写主机、端口和用户名，可选择密码或 SSH 密钥认证。</p>
            <div class="notice-card">
              <ShieldCheck :size="18" :stroke-width="1.7" />
              <span>首次连接未知主机时，应用会显示 SHA256 指纹；确认无误后才写入 known_hosts。</span>
            </div>
          </section>

          <section v-else class="step-content">
            <div class="hero-icon success"><CheckCircle2 :size="42" :stroke-width="1.35" /></div>
            <h2>可以开始了</h2>
            <p>终端和文件是一级工作区；测速、录制、笔记、书签、快捷命令和命令序列在“工具”菜单中。</p>
            <div class="shortcut-grid">
              <div><kbd>Ctrl</kbd><kbd>T</kbd><span>新建本地终端</span></div>
              <div><kbd>Ctrl</kbd><kbd>B</kbd><span>切换侧边栏</span></div>
              <div><kbd>Ctrl</kbd><kbd>,</kbd><span>打开设置</span></div>
              <div><kbd>?</kbd><span>快捷键帮助</span></div>
            </div>
          </section>
        </div>

        <div class="onboarding-footer">
          <div class="progress" aria-label="引导进度">
            <span v-for="(_, index) in steps" :key="index" :class="{ active: index <= current }" />
          </div>
          <div class="nav-row">
            <button v-if="current > 0" type="button" class="secondary-button" @click="current--">
              <ChevronLeft :size="14" />
              <span>上一步</span>
            </button>
            <div v-else />
            <span class="step-count">{{ current + 1 }} / {{ steps.length }}</span>
            <button v-if="current < steps.length - 1" type="button" class="primary-button" @click="current++">
              <span>下一步</span>
              <ChevronRight :size="14" />
            </button>
            <button v-else type="button" class="primary-button" @click="finish">
              <span>开始使用</span>
              <ArrowRight :size="14" />
            </button>
          </div>
          <button type="button" class="skip-button" @click="skip">跳过引导</button>
        </div>
      </div>
    </div>
  </Transition>
</template>

<script setup>
import { onMounted, ref } from 'vue'
import {
  ArrowRight,
  CheckCircle2,
  ChevronLeft,
  ChevronRight,
  FolderOpen,
  Server,
  ShieldCheck,
  Terminal,
  TerminalSquare,
  X,
} from 'lucide-vue-next'
import { STORAGE_KEYS } from '@/utils/storage-keys'

const show = ref(false)
const current = ref(0)
const steps = ['欢迎', '连接', '开始']

onMounted(() => {
  if (!localStorage.getItem(STORAGE_KEYS.ONBOARDED)) show.value = true
})

function complete() {
  localStorage.setItem(STORAGE_KEYS.ONBOARDED, 'true')
  show.value = false
}

function finish() {
  complete()
}

function skip() {
  complete()
}

defineExpose({ show, finish })
</script>

<style scoped>
.onboarding-backdrop { position: fixed; inset: 0; z-index: 100; display: flex; align-items: center; justify-content: center; padding: 20px; background: rgba(0, 0, 0, 0.68); backdrop-filter: blur(6px); }
.onboarding-dialog { position: relative; width: min(540px, 100%); max-height: min(620px, 90vh); display: flex; flex-direction: column; overflow: hidden; border: 1px solid var(--border); border-radius: 12px; background: var(--bg-elevated); box-shadow: var(--shadow-lg); }
.close-button { position: absolute; top: 10px; right: 10px; z-index: 2; width: 28px; height: 28px; display: inline-flex; align-items: center; justify-content: center; border: 0; border-radius: 6px; background: transparent; color: var(--fg-muted); }
.close-button:hover { background: var(--bg-hover); color: var(--fg-primary); }
.onboarding-content { min-height: 350px; padding: 44px 34px 26px; overflow-y: auto; }
.step-content { display: flex; flex-direction: column; align-items: center; animation: step-in 180ms ease-out; }
.hero-icon { width: 78px; height: 78px; display: flex; align-items: center; justify-content: center; border: 1px solid color-mix(in srgb, var(--accent) 30%, var(--border)); border-radius: 20px; background: var(--accent-hover); color: var(--accent); }
.hero-icon.success { color: var(--success); background: color-mix(in srgb, var(--success) 10%, var(--bg-surface)); border-color: color-mix(in srgb, var(--success) 30%, var(--border)); }
h2 { margin: 20px 0 8px; color: var(--fg-primary); font-size: 20px; font-weight: 600; text-align: center; }
p { max-width: 430px; margin: 0; color: var(--fg-secondary); font-size: 12px; line-height: 1.7; text-align: center; }
.feature-grid { width: min(430px, 100%); display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 7px; margin-top: 22px; }
.feature-grid > div { min-height: 58px; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 6px; border: 1px solid var(--border-subtle); border-radius: 8px; background: var(--bg-surface); color: var(--fg-muted); font-size: 10px; }
.notice-card { width: min(430px, 100%); display: flex; align-items: flex-start; gap: 9px; margin-top: 22px; padding: 10px 12px; border: 1px solid color-mix(in srgb, var(--warning) 25%, var(--border)); border-radius: 8px; background: color-mix(in srgb, var(--warning) 7%, var(--bg-surface)); color: var(--fg-secondary); font-size: 10px; line-height: 1.55; }
.notice-card svg { flex-shrink: 0; color: var(--warning); }
.shortcut-grid { width: min(390px, 100%); display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 7px; margin-top: 22px; }
.shortcut-grid > div { min-height: 40px; display: flex; align-items: center; gap: 4px; padding: 6px 8px; border: 1px solid var(--border-subtle); border-radius: 7px; background: var(--bg-surface); }
.shortcut-grid span { margin-left: auto; color: var(--fg-muted); font-size: 9px; }
kbd { min-width: 23px; padding: 2px 5px; border: 1px solid var(--border); border-radius: 4px; background: var(--bg-base); color: var(--fg-secondary); font-family: monospace; font-size: 9px; text-align: center; }
.onboarding-footer { padding: 10px 14px 12px; flex-shrink: 0; border-top: 1px solid var(--border-subtle); background: var(--bg-surface); }
.progress { height: 3px; display: flex; gap: 3px; margin-bottom: 10px; }
.progress span { flex: 1; border-radius: 999px; background: var(--border); }
.progress span.active { background: var(--accent); }
.nav-row { display: grid; grid-template-columns: 1fr auto 1fr; align-items: center; gap: 8px; }
.nav-row > button:last-child { justify-self: end; }
.secondary-button, .primary-button { height: 30px; display: inline-flex; align-items: center; justify-content: center; gap: 4px; padding: 0 10px; border: 0; border-radius: 6px; font-size: 11px; }
.secondary-button { background: var(--bg-hover); color: var(--fg-secondary); }
.primary-button { background: var(--accent); color: white; }
.step-count { color: var(--fg-muted); font-family: monospace; font-size: 9px; }
.skip-button { display: block; margin: 8px auto 0; border: 0; background: transparent; color: var(--fg-muted); font-size: 9px; }
.skip-button:hover { color: var(--fg-secondary); }
.onboarding-fade-enter-active, .onboarding-fade-leave-active { transition: opacity 160ms ease; }
.onboarding-fade-enter-from, .onboarding-fade-leave-to { opacity: 0; }
@keyframes step-in { from { opacity: 0; transform: translateY(5px); } to { opacity: 1; transform: translateY(0); } }
@media (max-width: 560px) { .onboarding-content { padding: 42px 18px 22px; } .feature-grid { grid-template-columns: 1fr; } .shortcut-grid { grid-template-columns: 1fr; } }
</style>
