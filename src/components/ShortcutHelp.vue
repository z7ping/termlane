<template>
  <div v-if="visible" class="fixed inset-0 bg-black/60 flex items-center justify-center z-50" @click.self="$emit('close')">
    <div class="bg-gray-800 rounded-lg w-[500px] border border-gray-600 shadow-2xl">
      <div class="p-4 border-b border-gray-700 flex items-center justify-between">
        <h3 class="text-sm font-medium text-gray-200">⌨️ 快捷键</h3>
        <button @click="$emit('close')" class="text-gray-400 hover:text-white">✕</button>
      </div>

      <div class="p-4 space-y-4 max-h-[60vh] overflow-y-auto">
        <div v-for="group in shortcutGroups" :key="group.name">
          <h4 class="text-xs text-gray-500 uppercase mb-2">{{ group.name }}</h4>
          <div class="space-y-1">
            <div v-for="s in group.items" :key="s.name" class="flex items-center justify-between py-1">
              <span class="text-sm text-gray-300">{{ s.name }}</span>
              <div class="flex gap-1">
                <kbd
                  v-for="(key, i) in s.keys"
                  :key="i"
                  class="px-1.5 py-0.5 bg-gray-700 border border-gray-600 rounded text-xs text-gray-400 font-mono"
                >{{ key }}</kbd>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div class="p-3 border-t border-gray-700 text-center">
        <span class="text-xs text-gray-500">按 <kbd class="px-1 py-0.5 bg-gray-700 rounded text-gray-400">?</kbd> 或 <kbd class="px-1 py-0.5 bg-gray-700 rounded text-gray-400">F1</kbd> 打开此面板</span>
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
