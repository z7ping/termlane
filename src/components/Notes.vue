<template>
  <div class="notes-view h-full flex flex-col">
    <div class="view-header">
      <NotebookPen :size="15" :stroke-width="1.8" />
      <span>笔记</span>
      <span class="scope-label">{{ connectionName || '全局' }}</span>
      <div class="flex-1" />
      <button type="button" class="primary-action" @click="createNote">
        <Plus :size="14" :stroke-width="1.8" />
        <span>新建</span>
      </button>
    </div>

    <div class="flex-1 flex min-h-0 overflow-hidden">
      <aside class="note-list">
        <div v-if="notes.length === 0" class="list-empty">暂无笔记</div>
        <div
          v-for="note in notes"
          v-else
          :key="note.id"
          class="note-row"
          :class="{ active: activeNote?.id === note.id }"
          role="button"
          tabindex="0"
          @click="selectNote(note)"
          @keydown.enter="selectNote(note)"
          @keydown.space.prevent="selectNote(note)"
        >
          <div class="min-w-0 flex-1 text-left">
            <div class="note-title">{{ note.title || '无标题' }}</div>
            <div class="note-updated">{{ formatUpdated(note.updated) }}</div>
          </div>
          <button
            type="button"
            class="delete-button"
            :aria-label="`删除 ${note.title || '无标题'} 笔记`"
            title="删除"
            @click.stop="requestDelete(note)"
          >
            <Trash2 :size="13" :stroke-width="1.8" />
          </button>
        </div>
      </aside>

      <main class="flex-1 min-w-0 flex flex-col">
        <div v-if="activeNote" class="flex-1 min-h-0 flex flex-col">
          <input
            v-model="activeNote.title"
            class="title-input"
            placeholder="笔记标题"
            @input="saveActive"
          />
          <textarea
            v-model="activeNote.content"
            class="note-editor"
            placeholder="记录与当前连接相关的信息…"
            spellcheck="false"
            @input="saveActive"
          />
        </div>
        <div v-else class="editor-empty">
          <NotebookPen :size="30" :stroke-width="1.4" />
          <div>选择或创建一个笔记</div>
          <span>笔记按连接分别保存，不会跨服务器混用。</span>
        </div>
      </main>
    </div>

    <div v-if="deleteTarget" class="dialog-backdrop" @click.self="deleteTarget = null">
      <div class="confirm-dialog" role="alertdialog" aria-modal="true" aria-label="删除笔记" @keydown.esc="deleteTarget = null">
        <div class="dialog-title">删除笔记</div>
        <div class="dialog-message">确认删除“{{ deleteTarget.title || '无标题' }}”？此操作无法撤销。</div>
        <div class="dialog-actions">
          <button type="button" class="secondary-button" @click="deleteTarget = null">取消</button>
          <button type="button" class="danger-button" @click="deleteNote">删除</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { onMounted, ref, watch } from 'vue'
import { NotebookPen, Plus, Trash2 } from 'lucide-vue-next'
import { STORAGE_KEYS } from '@/utils/storage-keys'

const props = defineProps({ connectionName: String })

const notes = ref([])
const activeNote = ref(null)
const deleteTarget = ref(null)
const loadedStorageKey = ref('')

function storageKeyFor(connectionName) {
  return `${STORAGE_KEYS.NOTES_PREFIX}${connectionName || 'global'}`
}

function loadNotes() {
  const storageKey = storageKeyFor(props.connectionName)
  loadedStorageKey.value = storageKey
  activeNote.value = null
  deleteTarget.value = null

  try {
    const parsed = JSON.parse(localStorage.getItem(storageKey) || '[]')
    notes.value = Array.isArray(parsed) ? parsed : []
  } catch {
    notes.value = []
  }
}

function saveNotes() {
  if (!loadedStorageKey.value) return
  localStorage.setItem(loadedStorageKey.value, JSON.stringify(notes.value))
}

function createNote() {
  const note = {
    id: crypto.randomUUID(),
    title: '',
    content: '',
    connection: props.connectionName || '全局',
    updated: new Date().toISOString(),
  }
  notes.value.unshift(note)
  activeNote.value = note
  saveNotes()
}

function selectNote(note) {
  activeNote.value = note
}

function saveActive() {
  if (!activeNote.value) return
  activeNote.value.updated = new Date().toISOString()
  const index = notes.value.findIndex(note => note.id === activeNote.value.id)
  if (index >= 0) notes.value[index] = { ...activeNote.value }
  saveNotes()
}

function requestDelete(note) {
  deleteTarget.value = note
}

function deleteNote() {
  const id = deleteTarget.value?.id
  if (!id) return
  notes.value = notes.value.filter(note => note.id !== id)
  if (activeNote.value?.id === id) activeNote.value = null
  deleteTarget.value = null
  saveNotes()
}

function normalizeStoredDates() {
  let changed = false
  for (const note of notes.value) {
    if (!note.updated) {
      note.updated = new Date().toISOString()
      changed = true
    }
  }
  if (changed) saveNotes()
}

function formatUpdated(value) {
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return '-'
  return date.toLocaleString('zh-CN', {
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  })
}

watch(() => props.connectionName, () => {
  loadNotes()
  normalizeStoredDates()
})

onMounted(() => {
  loadNotes()
  normalizeStoredDates()
})
</script>

<style scoped>
.notes-view { position: relative; background: var(--bg-base); color: var(--fg-secondary); }
.view-header { height: 36px; display: flex; align-items: center; gap: 7px; padding: 0 10px; flex-shrink: 0; border-bottom: 1px solid var(--border-subtle); background: var(--bg-surface); color: var(--fg-secondary); font-size: 12px; font-weight: 500; }
.scope-label { max-width: 180px; overflow: hidden; padding: 1px 6px; border-radius: 8px; background: var(--bg-hover); color: var(--fg-muted); font-size: 10px; font-weight: 400; text-overflow: ellipsis; white-space: nowrap; }
.primary-action, .secondary-button, .danger-button { height: 28px; display: inline-flex; align-items: center; justify-content: center; gap: 5px; padding: 0 9px; border: 0; border-radius: 6px; font-size: 11px; }
.primary-action { background: var(--accent); color: white; }
.secondary-button { background: var(--bg-hover); color: var(--fg-secondary); }
.danger-button { background: var(--danger); color: white; }
.note-list { width: 210px; flex-shrink: 0; overflow-y: auto; border-right: 1px solid var(--border-subtle); background: var(--bg-surface); }
.list-empty { padding: 28px 12px; color: var(--fg-muted); font-size: 11px; text-align: center; }
.note-row { width: 100%; min-height: 52px; display: flex; align-items: center; gap: 5px; padding: 7px 6px 7px 10px; border-bottom: 1px solid var(--border-subtle); background: transparent; color: var(--fg-secondary); cursor: default; }
.note-row:hover { background: var(--bg-hover); }
.note-row:focus-visible { outline: 1px solid var(--accent); outline-offset: -1px; }
.note-row.active { background: var(--accent-hover); }
.note-title { overflow: hidden; color: var(--fg-primary); font-size: 12px; font-weight: 500; text-overflow: ellipsis; white-space: nowrap; }
.note-updated { margin-top: 3px; color: var(--fg-muted); font-size: 9px; }
.delete-button { width: 25px; height: 25px; display: inline-flex; align-items: center; justify-content: center; flex-shrink: 0; border: 0; border-radius: 5px; background: transparent; color: var(--fg-muted); opacity: 0; }
.note-row:hover .delete-button, .delete-button:focus-visible { opacity: 1; }
.delete-button:hover { background: color-mix(in srgb, var(--danger) 12%, transparent); color: var(--danger); }
.title-input { height: 46px; padding: 0 14px; border: 0; border-bottom: 1px solid var(--border-subtle); outline: none; background: var(--bg-base); color: var(--fg-primary); font-size: 16px; font-weight: 600; }
.note-editor { flex: 1; min-height: 0; padding: 14px; resize: none; border: 0; outline: none; background: var(--bg-base); color: var(--fg-primary); font-family: 'Cascadia Code', 'Cascadia Mono', 'JetBrains Mono', Consolas, monospace; font-size: 12px; line-height: 1.6; }
.editor-empty { height: 100%; min-height: 220px; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 7px; color: var(--fg-muted); font-size: 12px; text-align: center; }
.editor-empty span { font-size: 11px; }
.dialog-backdrop { position: fixed; inset: 0; z-index: 60; display: flex; align-items: center; justify-content: center; padding: 20px; background: rgba(0, 0, 0, 0.62); }
.confirm-dialog { width: min(340px, 100%); padding: 14px; border: 1px solid var(--border); border-radius: 9px; background: var(--bg-elevated); box-shadow: var(--shadow-lg); }
.dialog-title { color: var(--fg-primary); font-size: 13px; font-weight: 600; }
.dialog-message { margin-top: 8px; color: var(--fg-secondary); font-size: 12px; line-height: 1.5; }
.dialog-actions { display: flex; justify-content: flex-end; gap: 7px; margin-top: 14px; }
@media (max-width: 760px) { .note-list { width: 170px; } .scope-label { display: none; } }
</style>
