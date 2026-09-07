<template>
  <div class="sequence-view h-full flex flex-col">
    <div class="view-header">
      <ListOrdered :size="15" :stroke-width="1.8" />
      <span>命令序列</span>
      <div class="flex-1" />
      <span v-if="replaying" class="replay-status">{{ replayIndex }}/{{ replayTotal }}</span>
      <button v-if="replaying" type="button" class="secondary-action danger" @click="stopReplay">
        <Square :size="12" fill="currentColor" />
        <span>停止</span>
      </button>
      <button v-else type="button" class="primary-action" :disabled="drafting" @click="startDraft">
        <Plus :size="14" :stroke-width="1.8" />
        <span>新建序列</span>
      </button>
    </div>

    <div v-if="drafting" class="draft-panel">
      <div class="draft-header">
        <ListPlus :size="14" :stroke-width="1.8" />
        <span>编辑序列</span>
        <span class="draft-count">{{ draftCommands.length }} 条</span>
        <div class="flex-1" />
        <button type="button" class="text-action" @click="cancelDraft">取消</button>
        <button type="button" class="primary-small" :disabled="draftCommands.length === 0" @click="openSaveDialog">完成</button>
      </div>
      <div class="command-entry">
        <input
          ref="commandInputRef"
          v-model="commandInput"
          class="font-mono"
          placeholder="输入一条命令，Enter 添加"
          @keydown.enter="addCommand"
        />
        <button type="button" :disabled="!commandInput.trim()" @click="addCommand">
          <Plus :size="13" :stroke-width="1.8" />
          <span>添加</span>
        </button>
      </div>
      <div v-if="draftCommands.length" class="draft-list">
        <div v-for="(item, index) in draftCommands" :key="item.id" class="draft-row">
          <span class="sequence-index">{{ index + 1 }}</span>
          <code>{{ item.command }}</code>
          <button type="button" :aria-label="`移除第 ${index + 1} 条命令`" @click="removeDraftCommand(item.id)">
            <X :size="13" :stroke-width="1.8" />
          </button>
        </div>
      </div>
    </div>

    <div class="flex-1 overflow-y-auto p-3">
      <div v-if="sequences.length === 0 && !drafting" class="empty-state">
        <ListOrdered :size="28" :stroke-width="1.4" />
        <div>暂无命令序列</div>
        <span>把多条常用命令保存为序列，可按顺序发送到当前终端。</span>
      </div>

      <div v-for="sequence in sequences" v-else :key="sequence.id" class="sequence-card">
        <div class="min-w-0 flex-1">
          <div class="sequence-name">{{ sequence.name }}</div>
          <div class="sequence-meta">{{ sequence.commands.length }} 条命令 · {{ formatDate(sequence.createdAt) }}</div>
          <div class="sequence-preview">
            <code v-for="(item, index) in sequence.commands.slice(0, 3)" :key="index">
              <span>{{ index + 1 }}.</span> {{ item.command }}
            </code>
            <span v-if="sequence.commands.length > 3">还有 {{ sequence.commands.length - 3 }} 条</span>
          </div>
        </div>
        <div class="sequence-actions">
          <button
            type="button"
            title="执行序列"
            :disabled="drafting || replaying || !sessionId"
            @click="replaySequence(sequence)"
          >
            <Play :size="14" :stroke-width="1.8" />
          </button>
          <button
            type="button"
            class="danger"
            title="删除"
            :disabled="drafting || replaying"
            @click="requestDeleteSequence(sequence.id)"
          >
            <Trash2 :size="14" :stroke-width="1.8" />
          </button>
        </div>
      </div>
    </div>

    <BaseModal :show="showSave" width="340px" title="保存命令序列" @close="showSave = false">
      <div class="save-dialog-content">
        <div class="dialog-title">保存命令序列</div>
        <div class="dialog-description">{{ draftCommands.length }} 条命令</div>
        <input ref="nameInputRef" v-model="sequenceName" placeholder="序列名称" @keydown.enter="saveSequence" />
        <div class="dialog-actions">
          <button type="button" class="secondary-button" @click="showSave = false">返回编辑</button>
          <button type="button" class="primary-button" :disabled="!sequenceName.trim()" @click="saveSequence">保存</button>
        </div>
      </div>
    </BaseModal>

    <ConfirmDialog
      :show="deleteSequenceId != null"
      title="删除命令序列"
      message="确认删除此命令序列？此操作无法撤销。"
      confirm-label="删除"
      danger
      @close="deleteSequenceId = null"
      @confirm="confirmDeleteSequence"
    />
  </div>
</template>

<script setup>
import { nextTick, onMounted, ref, watch } from 'vue'
import { ListOrdered, ListPlus, Play, Plus, Square, Trash2, X } from 'lucide-vue-next'
import { invoke } from '../utils/tauri.js'
import { secureStore } from '../utils/secure-store-browser'
import { STORAGE_KEYS } from '@/utils/storage-keys'
import BaseModal from './BaseModal.vue'
import ConfirmDialog from './ConfirmDialog.vue'

const props = defineProps({
  sessionId: String,
  isLocal: Boolean,
})

const emit = defineEmits(['status'])

const sequences = ref([])
const drafting = ref(false)
const draftCommands = ref([])
const commandInput = ref('')
const commandInputRef = ref(null)
const showSave = ref(false)
const sequenceName = ref('')
const nameInputRef = ref(null)
const deleteSequenceId = ref(null)
const replaying = ref(false)
const replayIndex = ref(0)
const replayTotal = ref(0)
let replayAbort = false

function normalizeSequences(value) {
  if (!Array.isArray(value)) return []
  return value.filter(sequence =>
    sequence
    && typeof sequence.name === 'string'
    && Array.isArray(sequence.commands),
  ).map(sequence => ({
    ...sequence,
    commands: sequence.commands
      .map(item => typeof item === 'string' ? { command: item } : item)
      .filter(item => item && typeof item.command === 'string' && item.command.trim()),
  }))
}

async function loadSequences() {
  const raw = localStorage.getItem(STORAGE_KEYS.MACROS)
  if (raw != null) {
    try {
      sequences.value = normalizeSequences(JSON.parse(raw))
      return
    } catch {
      sequences.value = []
    }
  }

  // 历史版本把非敏感宏数据误放进 browser secure fallback。
  // 当前会话仍可解密时迁移到稳定普通持久化，旧密文不主动删除。
  try {
    const legacy = normalizeSequences(await secureStore.get(STORAGE_KEYS.MACROS))
    if (legacy.length > 0) {
      sequences.value = legacy
      persistSequences()
      return
    }
  } catch {}

  sequences.value = []
}

function persistSequences() {
  localStorage.setItem(STORAGE_KEYS.MACROS, JSON.stringify(sequences.value))
}

function startDraft() {
  draftCommands.value = []
  commandInput.value = ''
  drafting.value = true
  nextTick(() => commandInputRef.value?.focus())
}

function cancelDraft() {
  drafting.value = false
  showSave.value = false
  draftCommands.value = []
  commandInput.value = ''
}

function addCommand() {
  const command = commandInput.value.trim()
  if (!drafting.value || !command) return
  draftCommands.value.push({ id: crypto.randomUUID(), command })
  commandInput.value = ''
}

function removeDraftCommand(id) {
  draftCommands.value = draftCommands.value.filter(item => item.id !== id)
}

function openSaveDialog() {
  if (draftCommands.value.length === 0) return
  sequenceName.value = `序列 ${new Date().toLocaleString('zh-CN', {
    month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit',
  })}`
  showSave.value = true
  nextTick(() => nameInputRef.value?.focus())
}

function saveSequence() {
  const name = sequenceName.value.trim()
  if (!name || draftCommands.value.length === 0) return

  sequences.value.push({
    id: crypto.randomUUID(),
    name,
    commands: draftCommands.value.map(item => ({ command: item.command })),
    createdAt: new Date().toISOString(),
  })
  persistSequences()
  cancelDraft()
  sequenceName.value = ''
}

function requestDeleteSequence(id) {
  deleteSequenceId.value = id
}

function confirmDeleteSequence() {
  const id = deleteSequenceId.value
  if (!id) return
  deleteSequenceId.value = null
  sequences.value = sequences.value.filter(sequence => sequence.id !== id)
  persistSequences()
}

async function replaySequence(sequence) {
  if (!props.sessionId) {
    emit('status', '请先打开一个终端会话')
    return
  }
  if (replaying.value || drafting.value) return

  replaying.value = true
  replayAbort = false
  replayTotal.value = sequence.commands.length
  replayIndex.value = 0
  const commandName = props.isLocal ? 'local_input' : 'ssh_shell_input'

  try {
    for (let index = 0; index < sequence.commands.length; index++) {
      if (replayAbort) break
      replayIndex.value = index + 1
      await invoke(commandName, {
        sessionId: props.sessionId,
        data: `${sequence.commands[index].command}\r`,
      })
      if (index < sequence.commands.length - 1 && !replayAbort) await delay(500)
    }
    if (!replayAbort) emit('status', `命令序列“${sequence.name}”已发送`)
  } catch (error) {
    emit('status', `命令序列执行失败：${error}`)
  } finally {
    replaying.value = false
    replayIndex.value = 0
    replayTotal.value = 0
  }
}

function stopReplay() {
  replayAbort = true
}

function delay(ms) {
  return new Promise(resolve => setTimeout(resolve, ms))
}

function formatDate(value) {
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return '-'
  return date.toLocaleString('zh-CN', {
    month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit',
  })
}

watch(() => props.sessionId, () => {
  if (replaying.value) {
    replayAbort = true
    emit('status', '会话已切换，命令序列已停止')
  }
})

onMounted(loadSequences)
</script>

<style scoped>
.sequence-view { position: relative; background: var(--bg-base); color: var(--fg-secondary); }
.view-header { height: 36px; display: flex; align-items: center; gap: 7px; padding: 0 10px; flex-shrink: 0; border-bottom: 1px solid var(--border-subtle); background: var(--bg-surface); color: var(--fg-secondary); font-size: 12px; font-weight: 500; }
.primary-action, .secondary-action, .primary-small, .text-action, .secondary-button, .primary-button { height: 28px; display: inline-flex; align-items: center; justify-content: center; gap: 5px; padding: 0 9px; border: 0; border-radius: var(--radius-sm); font-size: 11px; }
.primary-action, .primary-small, .primary-button { background: var(--accent); color: white; }
.secondary-action, .secondary-button { background: var(--bg-hover); color: var(--fg-secondary); }
.secondary-action.danger { color: var(--danger); }
.text-action { background: transparent; color: var(--fg-muted); }
.replay-status { color: var(--warning); font-family: var(--font-mono); font-size: 10px; }
.draft-panel { flex-shrink: 0; border-bottom: 1px solid var(--border-subtle); background: var(--bg-surface); }
.draft-header { height: 34px; display: flex; align-items: center; gap: 7px; padding: 0 10px; color: var(--fg-secondary); font-size: 11px; }
.draft-count { padding: 1px 5px; border-radius: 999px; background: var(--bg-hover); color: var(--fg-muted); font-size: 9px; }
.command-entry { display: flex; gap: 6px; padding: 0 10px 9px; }
.command-entry input { flex: 1; min-width: 0; height: 31px; padding: 0 9px; border: 1px solid var(--border); border-radius: var(--radius-sm); outline: none; background: var(--bg-base); color: var(--fg-primary); font-size: 12px; }
.command-entry input:focus { border-color: var(--accent); }
.command-entry button { height: 31px; display: inline-flex; align-items: center; gap: 5px; padding: 0 9px; border: 0; border-radius: var(--radius-sm); background: var(--accent-hover); color: var(--accent); font-size: 11px; }
.draft-list { max-height: 150px; overflow-y: auto; border-top: 1px solid var(--border-subtle); }
.draft-row { min-height: 29px; display: flex; align-items: center; gap: 8px; padding: 0 9px; color: var(--fg-secondary); }
.sequence-index { width: 20px; flex-shrink: 0; color: var(--fg-muted); font-family: var(--font-mono); font-size: 9px; text-align: right; }
.draft-row code { flex: 1; min-width: 0; overflow: hidden; color: var(--fg-secondary); font-size: 10px; text-overflow: ellipsis; white-space: nowrap; }
.draft-row button { width: 23px; height: 23px; display: inline-flex; align-items: center; justify-content: center; border: 0; border-radius: 5px; background: transparent; color: var(--fg-muted); }
.draft-row button:hover { background: var(--bg-hover); color: var(--danger); }
.empty-state { min-height: 220px; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 7px; color: var(--fg-muted); font-size: 12px; text-align: center; }
.empty-state span { max-width: 360px; font-size: 11px; }
.sequence-card { min-height: 76px; display: flex; align-items: flex-start; gap: 10px; margin-bottom: 7px; padding: 10px; border: 1px solid var(--border-subtle); border-radius: var(--radius); background: var(--bg-surface); }
.sequence-name { overflow: hidden; color: var(--fg-primary); font-size: 12px; font-weight: 500; text-overflow: ellipsis; white-space: nowrap; }
.sequence-meta { margin-top: 3px; color: var(--fg-muted); font-size: 9px; }
.sequence-preview { display: flex; flex-direction: column; gap: 2px; margin-top: 7px; }
.sequence-preview code, .sequence-preview > span { overflow: hidden; color: var(--fg-muted); font-size: 9px; text-overflow: ellipsis; white-space: nowrap; }
.sequence-preview code span { color: var(--fg-muted); }
.sequence-actions { display: flex; gap: 2px; }
.sequence-actions button { width: 27px; height: 27px; display: inline-flex; align-items: center; justify-content: center; border: 0; border-radius: 5px; background: transparent; color: var(--fg-muted); }
.sequence-actions button:hover:not(:disabled) { background: var(--bg-hover); color: var(--fg-primary); }
.sequence-actions button.danger:hover:not(:disabled) { color: var(--danger); }
.save-dialog-content .dialog-title { color: var(--fg-primary); font-size: 13px; font-weight: 600; }
.dialog-description { margin-top: 4px; color: var(--fg-muted); font-size: 10px; }
.save-dialog-content > input { width: 100%; height: 31px; margin-top: 11px; padding: 0 9px; border: 1px solid var(--border); border-radius: var(--radius-sm); outline: none; background: var(--bg-base); color: var(--fg-primary); font-size: 12px; }
.save-dialog-content > input:focus { border-color: var(--accent); }
.dialog-actions { display: flex; justify-content: flex-end; gap: 7px; margin-top: 14px; }
</style>
