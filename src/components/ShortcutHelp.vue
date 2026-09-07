<template>
  <div v-if="visible" class="help-backdrop" @click.self="$emit('close')">
    <div class="help-dialog" role="dialog" aria-modal="true" aria-label="快捷键帮助" @keydown.esc="$emit('close')">
      <div class="help-header">
        <Keyboard :size="16" :stroke-width="1.8" />
        <span>快捷键</span>
        <div class="flex-1" />
        <button type="button" class="icon-button" aria-label="关闭快捷键帮助" @click="$emit('close')">
          <X :size="15" :stroke-width="1.8" />
        </button>
      </div>

      <div class="help-content">
        <section v-for="group in shortcutGroups" :key="group.name">
          <h4>{{ group.name }}</h4>
          <div class="shortcut-list">
            <div v-for="shortcut in group.items" :key="shortcut.name" class="shortcut-row">
              <div>
                <div class="shortcut-name">{{ shortcut.name }}</div>
                <div v-if="shortcut.note" class="shortcut-note">{{ shortcut.note }}</div>
              </div>
              <div class="key-group">
                <kbd v-for="(key, index) in keys(shortcut.binding)" :key="`${shortcut.name}-${index}`">{{ key }}</kbd>
              </div>
            </div>
          </div>
        </section>
      </div>

      <div class="help-footer">应用快捷键可在设置中调整；终端控制键保持 Shell 的标准行为。</div>
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue'
import { Keyboard, X } from 'lucide-vue-next'
import { getShortcut } from '@/utils/shortcuts'

defineProps({ visible: Boolean })
defineEmits(['close'])

const shortcutGroups = computed(() => [
  {
    name: '终端',
    items: [
      { name: '搜索终端输出', binding: getShortcut('搜索') },
      { name: '清屏', binding: 'Ctrl+L', note: '终端固定控制键，不在应用设置中重映射。' },
      { name: '中断当前命令', binding: 'Ctrl+C', note: '有选中文本时优先复制选中内容。' },
    ],
  },
  {
    name: '工作区',
    items: [
      { name: '新建本地终端', binding: getShortcut('新建标签') },
      { name: '关闭当前标签', binding: getShortcut('关闭标签') },
      { name: '切换侧边栏', binding: getShortcut('切换侧边栏') },
      { name: '打开设置', binding: getShortcut('设置') },
      { name: '全屏', binding: getShortcut('全屏') },
      { name: '快捷键帮助', binding: getShortcut('帮助') },
    ],
  },
])

function keys(binding) {
  if (!binding) return []
  return binding === '?' ? ['?'] : binding.split('+')
}
</script>

<style scoped>
.help-backdrop { position: fixed; inset: 0; z-index: 70; display: flex; align-items: center; justify-content: center; padding: 20px; background: rgba(0, 0, 0, 0.62); }
.help-dialog { width: min(520px, 100%); max-height: min(620px, 86vh); display: flex; flex-direction: column; overflow: hidden; border: 1px solid var(--border); border-radius: 10px; background: var(--bg-elevated); box-shadow: var(--shadow-lg); }
.help-header { height: 40px; display: flex; align-items: center; gap: 8px; padding: 0 11px; flex-shrink: 0; border-bottom: 1px solid var(--border-subtle); color: var(--fg-primary); font-size: 13px; font-weight: 600; }
.icon-button { width: 27px; height: 27px; display: inline-flex; align-items: center; justify-content: center; border: 0; border-radius: 5px; background: transparent; color: var(--fg-muted); }
.icon-button:hover { background: var(--bg-hover); color: var(--fg-primary); }
.help-content { overflow-y: auto; padding: 14px; }
.help-content section + section { margin-top: 16px; }
.help-content h4 { margin: 0 0 6px; color: var(--fg-muted); font-size: 10px; font-weight: 600; letter-spacing: 0.08em; text-transform: uppercase; }
.shortcut-list { overflow: hidden; border: 1px solid var(--border-subtle); border-radius: 8px; background: var(--bg-surface); }
.shortcut-row { min-height: 42px; display: flex; align-items: center; justify-content: space-between; gap: 14px; padding: 7px 10px; border-bottom: 1px solid var(--border-subtle); }
.shortcut-row:last-child { border-bottom: 0; }
.shortcut-name { color: var(--fg-secondary); font-size: 12px; }
.shortcut-note { max-width: 290px; margin-top: 2px; color: var(--fg-muted); font-size: 9px; line-height: 1.4; }
.key-group { display: flex; gap: 4px; flex-shrink: 0; }
kbd { min-width: 24px; padding: 2px 6px; border: 1px solid var(--border); border-radius: 5px; background: var(--bg-base); color: var(--fg-muted); font-family: monospace; font-size: 10px; text-align: center; }
.help-footer { padding: 9px 12px; flex-shrink: 0; border-top: 1px solid var(--border-subtle); color: var(--fg-muted); font-size: 10px; text-align: center; }
@media (max-width: 560px) { .shortcut-row { align-items: flex-start; flex-direction: column; } .key-group { align-self: flex-end; } }
</style>
