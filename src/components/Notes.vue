<template>
  <div class="h-full flex flex-col bg-gray-900">
    <div class="h-9 bg-gray-800 border-b border-gray-700 flex items-center px-3 gap-2">
      <span class="text-sm font-medium text-gray-300">📝 笔记</span>
      <div class="flex-1" />
      <button @click="createNote" class="text-xs px-2 py-0.5 bg-blue-600 hover:bg-blue-500 rounded text-white">+ 新建</button>
    </div>

    <div class="flex-1 flex overflow-hidden">
      <!-- Note list -->
      <div class="w-48 border-r border-gray-700 overflow-y-auto" style="background: #1e1e1e;">
        <div v-for="note in notes" :key="note.id" @click="selectNote(note)" @contextmenu.prevent="deleteNote(note.id)"
          class="px-3 py-2 cursor-pointer text-sm border-b border-gray-800"
          :class="activeNote?.id === note.id ? 'bg-blue-600/20 text-blue-300' : 'text-gray-300 hover:bg-gray-800'">
          <div class="truncate font-medium">{{ note.title || '无标题' }}</div>
          <div class="text-xs text-gray-500 mt-0.5">{{ note.connection || '全局' }} · {{ note.updated }}</div>
        </div>
        <div v-if="notes.length === 0" class="p-4 text-center text-gray-500 text-xs">暂无笔记</div>
      </div>

      <!-- Editor -->
      <div class="flex-1 flex flex-col">
        <div v-if="activeNote" class="flex-1 flex flex-col">
          <input v-model="activeNote.title" @input="saveActive" class="bg-transparent text-gray-200 text-lg font-medium px-4 py-2 focus:outline-none border-b border-gray-700" placeholder="笔记标题" />
          <textarea v-model="activeNote.content" @input="saveActive" class="flex-1 bg-gray-900 text-gray-200 text-sm p-4 font-mono resize-none focus:outline-none" placeholder="支持 Markdown..." spellcheck="false" />
        </div>
        <div v-else class="flex-1 flex items-center justify-center text-gray-500 text-sm">
          选择或创建一个笔记
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'

const props = defineProps({ connectionName: String })

const notes = ref([])
const activeNote = ref(null)

function loadNotes() {
  try {
    const key = `xterminal_notes_${props.connectionName || 'global'}`
    notes.value = JSON.parse(localStorage.getItem(key) || '[]')
  } catch { notes.value = [] }
}

function saveNotes() {
  const key = `xterminal_notes_${props.connectionName || 'global'}`
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
