<template>
  <div class="h-full flex flex-col" style="background: var(--bg-base)">
    <div class="h-9 border-b flex items-center px-3 gap-2" style="background: var(--bg-surface); border-color: var(--border)">
      <span class="text-sm font-medium" style="color: var(--fg-secondary)">📝 笔记</span>
      <div class="flex-1" />
      <button @click="createNote" class="text-xs px-2 py-0.5 rounded" style="background: var(--accent); color: white">+ 新建</button>
    </div>

    <div class="flex-1 flex overflow-hidden">
      <!-- Note list -->
      <div class="w-48 border-r overflow-y-auto" style="background: var(--bg-surface); border-color: var(--border)">
        <div v-for="note in notes" :key="note.id" @click="selectNote(note)" @contextmenu.prevent="deleteNote(note.id)"
          class="px-3 py-2 cursor-pointer text-sm border-b" style="border-color: var(--border)"
          :style="activeNote?.id === note.id ? { background: 'color-mix(in srgb, var(--accent) 20%, transparent)', color: 'var(--accent)' } : { color: 'var(--fg-secondary)' }">
          <div class="truncate font-medium">{{ note.title || '无标题' }}</div>
          <div class="text-xs mt-0.5" style="color: var(--fg-muted)">{{ note.connection || '全局' }} · {{ note.updated }}</div>
        </div>
        <div v-if="notes.length === 0" class="p-4 text-center text-xs" style="color: var(--fg-muted)">暂无笔记</div>
      </div>

      <!-- Editor -->
      <div class="flex-1 flex flex-col">
        <div v-if="activeNote" class="flex-1 flex flex-col">
          <input v-model="activeNote.title" @input="saveActive" class="bg-transparent text-lg font-medium px-4 py-2 focus:outline-none border-b" style="color: var(--fg-primary); border-color: var(--border)" placeholder="笔记标题" />
          <textarea v-model="activeNote.content" @input="saveActive" class="flex-1 text-sm p-4 font-mono resize-none focus:outline-none" style="background: var(--bg-base); color: var(--fg-primary)" placeholder="支持 Markdown..." spellcheck="false" />
        </div>
        <div v-else class="flex-1 flex items-center justify-center text-sm" style="color: var(--fg-muted)">
          选择或创建一个笔记
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { STORAGE_KEYS } from '@/utils/storage-keys'
import { ref, onMounted } from 'vue'

const props = defineProps({ connectionName: String })

const notes = ref([])
const activeNote = ref(null)

function loadNotes() {
  try {
    const key = `${STORAGE_KEYS.NOTES_PREFIX}${props.connectionName || 'global'}`
    notes.value = JSON.parse(localStorage.getItem(key) || '[]')
  } catch { notes.value = [] }
}

function saveNotes() {
  const key = `${STORAGE_KEYS.NOTES_PREFIX}${props.connectionName || 'global'}`
  localStorage.setItem(key, JSON.stringify(notes.value))
}

function createNote() {
  const note = {
    id: `note_${Date.now()}`,
    title: '',
    content: '',
    connection: props.connectionName || '全局',
    updated: new Date().toLocaleDateString('zh-CN'),
  }
  notes.value.unshift(note)
  activeNote.value = note
  saveNotes()
}

function selectNote(note) { activeNote.value = note }

function saveActive() {
  if (!activeNote.value) return
  activeNote.value.updated = new Date().toLocaleDateString('zh-CN')
  const idx = notes.value.findIndex(n => n.id === activeNote.value.id)
  if (idx >= 0) notes.value[idx] = { ...activeNote.value }
  saveNotes()
}

function deleteNote(id) {
  if (!confirm('删除此笔记？')) return
  notes.value = notes.value.filter(n => n.id !== id)
  if (activeNote.value?.id === id) activeNote.value = null
  saveNotes()
}

onMounted(loadNotes)
</script>
