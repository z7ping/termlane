<template>
  <div class="commands-view h-full flex flex-col">
    <div class="view-header">
      <Command :size="15" :stroke-width="1.8" />
      <span>快捷命令</span>
      <div class="flex-1" />
      <button type="button" class="primary-action" @click="openAddDialog">
        <Plus :size="14" :stroke-width="1.8" />
        <span>新增</span>
      </button>
    </div>

    <div class="flex-1 overflow-y-auto p-3">
      <div v-if="commands.length === 0" class="empty-state">
        <Command :size="28" :stroke-width="1.4" />
        <div>暂无快捷命令</div>
        <span>添加常用命令后可一键发送到当前终端。</span>
      </div>

      <section v-for="group in groupedCommands" v-else :key="group.name" class="command-group">
        <div class="group-title">
          <Folder :size="13" :stroke-width="1.8" />
          <span>{{ group.name }}</span>
          <span class="group-count">{{ group.items.length }}</span>
        </div>

        <div class="command-grid">
          <div
            v-for="commandItem in group.items"
            :key="commandItem.id"
            class="command-card"
            role="button"
            tabindex="0"
            @click="$emit('run', commandItem.command)"
            @keydown.enter="$emit('run', commandItem.command)"
            @keydown.space.prevent="$emit('run', commandItem.command)"
          >
            <div class="min-w-0 flex-1 text-left">
              <div class="command-name">{{ commandItem.name }}</div>
              <div class="command-text">{{ commandItem.command }}</div>
            </div>
            <button
              type="button"
              class="delete-button"
              :aria-label="`删除 ${commandItem.name}`"
              title="删除"
              @click.stop="deleteCommand(commandItem.id)"
            >
              <Trash2 :size="13" :stroke-width="1.8" />
            </button>
          </div>
        </div>
      </section>
    </div>

    <div v-if="showAdd" class="dialog-backdrop" @click.self="showAdd = false">
      <div class="command-dialog" role="dialog" aria-modal="true" aria-label="新增快捷命令" @keydown.esc="showAdd = false">
        <div class="dialog-title">新增快捷命令</div>
        <label>
          <span>名称</span>
          <input v-model="newCommand.name" placeholder="例如：查看磁盘" />
        </label>
        <label>
          <span>命令</span>
          <input v-model="newCommand.command" class="font-mono" placeholder="df -h" @keydown.enter="addCommand" />
        </label>
        <label>
          <span>分组</span>
          <input v-model="newCommand.group" placeholder="自定义" />
        </label>
        <div class="dialog-actions">
          <button type="button" class="secondary-button" @click="showAdd = false">取消</button>
          <button
            type="button"
            class="primary-button"
            :disabled="!newCommand.name.trim() || !newCommand.command.trim()"
            @click="addCommand"
          >添加</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed, onMounted, reactive, ref } from 'vue'
import { Command, Folder, Plus, Trash2 } from 'lucide-vue-next'
import { STORAGE_KEYS } from '@/utils/storage-keys'

defineEmits(['run'])

const DEFAULT_COMMANDS = [
  { id: 'builtin-system-info', name: '系统信息', command: 'uname -a', group: '系统' },
  { id: 'builtin-disk', name: '磁盘使用', command: 'df -h', group: '系统' },
  { id: 'builtin-memory', name: '内存使用', command: 'free -h', group: '系统' },
  { id: 'builtin-processes', name: '进程列表', command: 'ps aux --sort=-%mem | head -20', group: '系统' },
  { id: 'builtin-network', name: '网络连接', command: 'ss -tlnp', group: '网络' },
  { id: 'builtin-docker', name: 'Docker 容器', command: 'docker ps -a', group: 'Docker' },
  { id: 'builtin-docker-logs', name: 'Docker 日志', command: 'docker logs --tail 50', group: 'Docker' },
  { id: 'builtin-git-status', name: 'Git 状态', command: 'git status', group: 'Git' },
  { id: 'builtin-git-log', name: 'Git 日志', command: 'git log --oneline -10', group: 'Git' },
  { id: 'builtin-large-files', name: '查找大文件', command: 'find / -type f -size +100M 2>/dev/null | head -10', group: '工具' },
]

const commands = ref([])
const showAdd = ref(false)
const newCommand = reactive({ name: '', command: '', group: '自定义' })

const groupedCommands = computed(() => {
  const groups = new Map()
  for (const commandItem of commands.value) {
    const groupName = commandItem.group?.trim() || '其他'
    if (!groups.has(groupName)) groups.set(groupName, { name: groupName, items: [] })
    groups.get(groupName).items.push(commandItem)
  }
  return [...groups.values()]
})

function loadCommands() {
  const raw = localStorage.getItem(STORAGE_KEYS.QUICK_COMMANDS)
  if (raw == null) {
    commands.value = DEFAULT_COMMANDS.map(item => ({ ...item }))
    persistCommands()
    return
  }

  try {
    const parsed = JSON.parse(raw)
    commands.value = Array.isArray(parsed)
      ? parsed.filter(item => item && typeof item.name === 'string' && typeof item.command === 'string')
      : DEFAULT_COMMANDS.map(item => ({ ...item }))
  } catch {
    commands.value = DEFAULT_COMMANDS.map(item => ({ ...item }))
    persistCommands()
  }
}

function persistCommands() {
  localStorage.setItem(STORAGE_KEYS.QUICK_COMMANDS, JSON.stringify(commands.value))
}

function openAddDialog() {
  newCommand.name = ''
  newCommand.command = ''
  newCommand.group = '自定义'
  showAdd.value = true
}

function addCommand() {
  const name = newCommand.name.trim()
  const command = newCommand.command.trim()
  if (!name || !command) return

  commands.value.push({
    id: crypto.randomUUID(),
    name,
    command,
    group: newCommand.group.trim() || '自定义',
  })
  persistCommands()
  showAdd.value = false
}

function deleteCommand(id) {
  commands.value = commands.value.filter(item => item.id !== id)
  persistCommands()
}

onMounted(loadCommands)
</script>

<style scoped>
.commands-view { position: relative; background: var(--bg-base); color: var(--fg-secondary); }
.view-header { height: 36px; display: flex; align-items: center; gap: 7px; padding: 0 10px; flex-shrink: 0; border-bottom: 1px solid var(--border-subtle); background: var(--bg-surface); color: var(--fg-secondary); font-size: 12px; font-weight: 500; }
.primary-action, .secondary-button, .primary-button { height: 28px; display: inline-flex; align-items: center; justify-content: center; gap: 5px; padding: 0 9px; border: 0; border-radius: 6px; font-size: 11px; }
.primary-action, .primary-button { background: var(--accent); color: white; }
.primary-button:disabled { cursor: not-allowed; opacity: 0.4; }
.secondary-button { background: var(--bg-hover); color: var(--fg-secondary); }
.empty-state { min-height: 220px; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 7px; color: var(--fg-muted); font-size: 12px; text-align: center; }
.empty-state span { font-size: 11px; }
.command-group { margin-bottom: 15px; }
.group-title { height: 28px; display: flex; align-items: center; gap: 6px; padding: 0 4px; color: var(--fg-muted); font-size: 10px; font-weight: 600; }
.group-count { padding: 0 5px; border-radius: 8px; background: var(--bg-hover); font-weight: 400; }
.command-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 6px; }
.command-card { min-width: 0; min-height: 52px; display: flex; align-items: center; gap: 6px; padding: 8px 7px 8px 10px; border: 1px solid var(--border-subtle); border-radius: 8px; background: var(--bg-surface); color: var(--fg-secondary); cursor: default; transition: background-color var(--transition-fast), border-color var(--transition-fast); }
.command-card:hover { background: var(--bg-hover); border-color: var(--border); }
.command-card:focus-visible { outline: 1px solid var(--accent); outline-offset: 1px; }
.command-name { overflow: hidden; color: var(--fg-primary); font-size: 12px; font-weight: 500; text-overflow: ellipsis; white-space: nowrap; }
.command-text { margin-top: 3px; overflow: hidden; color: var(--fg-muted); font-family: monospace; font-size: 10px; text-overflow: ellipsis; white-space: nowrap; }
.delete-button { width: 25px; height: 25px; display: inline-flex; align-items: center; justify-content: center; flex-shrink: 0; border: 0; border-radius: 5px; background: transparent; color: var(--fg-muted); opacity: 0; transition: opacity var(--transition-fast), background-color var(--transition-fast), color var(--transition-fast); }
.command-card:hover .delete-button, .delete-button:focus-visible { opacity: 1; }
.delete-button:hover { background: color-mix(in srgb, var(--danger) 12%, transparent); color: var(--danger); }
.dialog-backdrop { position: fixed; inset: 0; z-index: 60; display: flex; align-items: center; justify-content: center; padding: 20px; background: rgba(0, 0, 0, 0.62); }
.command-dialog { width: min(340px, 100%); padding: 14px; border: 1px solid var(--border); border-radius: 9px; background: var(--bg-elevated); box-shadow: var(--shadow-lg); }
.dialog-title { margin-bottom: 12px; color: var(--fg-primary); font-size: 13px; font-weight: 600; }
.command-dialog label { display: flex; flex-direction: column; gap: 5px; margin-top: 9px; color: var(--fg-muted); font-size: 10px; }
.command-dialog input { height: 31px; padding: 0 9px; border: 1px solid var(--border); border-radius: 6px; outline: none; background: var(--bg-base); color: var(--fg-primary); font-size: 12px; }
.command-dialog input:focus { border-color: var(--accent); }
.dialog-actions { display: flex; justify-content: flex-end; gap: 7px; margin-top: 14px; }
@media (max-width: 760px) { .command-grid { grid-template-columns: minmax(0, 1fr); } }
</style>
