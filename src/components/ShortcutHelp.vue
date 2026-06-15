<template>
  <div v-if="visible" class="fixed inset-0 flex items-center justify-center z-50" style="background: color-mix(in srgb, #000 60%, transparent)" @click.self="$emit('close')">
    <div class="rounded-lg w-[500px] shadow-2xl" style="background: var(--bg-surface); border-color: var(--border-subtle)">
      <div class="p-4 border-b flex items-center justify-between" style="border-color: var(--border)">
        <h3 class="text-sm font-medium" style="color: var(--fg-primary)">⌨️ 快捷键</h3>
        <button @click="$emit('close')" class="" style="color: var(--fg-muted)">✕</button>
      </div>

      <div class="p-4 space-y-4 max-h-[60vh] overflow-y-auto">
        <div v-for="group in shortcutGroups" :key="group.name">
          <h4 class="text-xs uppercase mb-2" style="color: var(--fg-muted)">{{ group.name }}</h4>
          <div class="space-y-1">
            <div v-for="s in group.items" :key="s.name" class="flex items-center justify-between py-1">
              <span class="text-sm" style="color: var(--fg-secondary)">{{ s.name }}</span>
              <div class="flex gap-1">
                <kbd
                  v-for="(key, i) in s.keys"
                  :key="i"
                  class="px-1.5 py-0.5 border rounded text-xs font-mono" style="background: var(--bg-elevated); border-color: var(--border-subtle); color: var(--fg-muted)"
                >{{ key }}</kbd>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div class="p-3 border-t text-center" style="border-color: var(--border)">
        <span class="text-xs" style="color: var(--fg-muted)">按 <kbd class="px-1 py-0.5 rounded" style="background: var(--bg-elevated); color: var(--fg-muted)">?</kbd> 或 <kbd class="px-1 py-0.5 rounded" style="background: var(--bg-elevated); color: var(--fg-muted)">F1</kbd> 打开此面板</span>
      </div>
    </div>
  </div>
</template>

<script setup>
defineProps({ visible: Boolean })
defineEmits(['close'])

const shortcutGroups = [
  {
    name: '终端',
    items: [
      { name: '清屏', keys: ['Ctrl', 'L'] },
      { name: '中断', keys: ['Ctrl', 'C'] },
      { name: '搜索', keys: ['Ctrl', 'Shift', 'F'] },
      { name: '复制', keys: ['Ctrl', 'Shift', 'C'] },
      { name: '粘贴', keys: ['Ctrl', 'Shift', 'V'] },
    ],
  },
  {
    name: '标签页',
    items: [
      { name: '新建标签', keys: ['Ctrl', 'T'] },
      { name: '关闭标签', keys: ['Ctrl', 'W'] },
      { name: '下一个标签', keys: ['Ctrl', 'Tab'] },
      { name: '上一个标签', keys: ['Ctrl', 'Shift', 'Tab'] },
      { name: '切换到标签 1-9', keys: ['Ctrl', '1-9'] },
    ],
  },
  {
    name: '视图',
    items: [
      { name: '分屏', keys: ['Ctrl', 'Shift', 'D'] },
      { name: '切换侧边栏', keys: ['Ctrl', 'B'] },
      { name: '全屏', keys: ['F11'] },
      { name: '字体放大', keys: ['Ctrl', '+'] },
      { name: '字体缩小', keys: ['Ctrl', '-'] },
    ],
  },
  {
    name: '其他',
    items: [
      { name: '快捷键帮助', keys: ['?'] },
      { name: '设置', keys: ['Ctrl', ','] },
      { name: '新连接', keys: ['Ctrl', 'N'] },
    ],
  },
]
</script>
