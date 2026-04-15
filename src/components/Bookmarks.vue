<template>
  <div class="h-full flex flex-col bg-gray-900">
    <div class="h-9 bg-gray-800 border-b border-gray-700 flex items-center px-3 gap-2">
      <span class="text-sm font-medium text-gray-300">🔖 书签</span>
      <div class="flex-1" />
      <button @click="addBookmark" class="text-xs px-2 py-0.5 bg-blue-600 hover:bg-blue-500 rounded text-white">+ 添加</button>
    </div>

    <div class="flex-1 overflow-y-auto p-2">
      <div v-for="(group, gIdx) in groupedBookmarks" :key="gIdx" class="mb-3">
        <div class="text-xs text-gray-500 px-2 py-1 flex items-center gap-1">
          <span>{{ group.icon }}</span> {{ group.label }}
        </div>
        <div v-for="bm in group.items" :key="bm.id" @click="$emit('navigate', bm)"
          @contextmenu.prevent="showMenu($event, bm)"
          class="flex items-center gap-2 px-3 py-1.5 rounded cursor-pointer text-sm hover:bg-gray-700 group">
          <span class="text-gray-400">{{ bm.icon || '📁' }}</span>
          <div class="flex-1 min-w-0">
            <div class="truncate text-gray-200">{{ bm.name }}</div>
            <div class="truncate text-xs text-gray-500 font-mono">{{ bm.path }}</div>
          </div>
          <span class="text-xs text-gray-600">{{ bm.host }}</span>
        </div>
      </div>

      <div v-if="bookmarks.length === 0" class="text-center text-gray-500 text-sm mt-10">
        暂无书签<br/><span class="text-xs">SFTP 中右键添加常用目录</span>
      </div>
    </div>

    <!-- Add Dialog -->
    <div v-if="showAdd" class="fixed inset-0 bg-black/60 flex items-center justify-center z-50" @click.self="showAdd = false">
      <div class="bg-gray-800 rounded-lg w-80 border border-gray-600 p-4 space-y-3">
        <h3 class="text-sm font-medium">添加书签</h3>
        <input v-model="newBm.name" class="w-full bg-gray-900 border border-gray-600 rounded px-3 py-1.5 text-sm focus:outline-none focus:border-blue-500" placeholder="名称（如：项目根目录）" />
        <input v-model="newBm.path" class="w-full bg-gray-900 border border-gray-600 rounded px-3 py-1.5 text-sm font-mono focus:outline-none focus:border-blue-500" placeholder="/var/www/html" />
        <input v-model="newBm.host" class="w-full bg-gray-900 border border-gray-600 rounded px-3 py-1.5 text-sm focus:outline-none focus:border-blue-500" placeholder="服务器名（可选）" />
        <div class="flex justify-end gap-2">
          <button @click="showAdd = false" class="px-3 py-1 text-sm text-gray-400">取消</button>
          <button @click="saveBookmark" :disabled="!newBm.path" class="px-3 py-1 text-sm bg-blue-600 rounded text-white disabled:opacity-50">添加</button>
        </div>
      </div>
    </div>

    <!-- Context Menu -->
    <div v-if="ctx.show" :style="{ left: ctx.x + 'px', top: ctx.y + 'px' }" class="fixed bg-gray-800 border border-gray-600 rounded shadow-lg z-50 py-1 text-xs min-w-[120px]" @click.stop>
      <div @click="$emit('navigate', ctx.bm); ctx.show = false" class="px-4 py-1.5 hover:bg-gray-600 cursor-pointer text-gray-300">📂 打开</div>
      <div @click="copyPath" class="px-4 py-1.5 hover:bg-gray-600 cursor-pointer text-gray-300">📋 复制路径</div>
      <div class="border-t border-gray-700 my-1" />
      <div @click="delBookmark" class="px-4 py-1.5 hover:bg-gray-600 cursor-pointer text-red-400">🗑 删除</div>
    </div>
    <div v-if="ctx.show" class="fixed inset-0 z-40" @click="ctx.show = false" />
  </div>
</template>

<script setup>
import { ref, reactive, computed, onMounted } from 'vue'
import { getBookmarks, storeBookmarks } from '../utils/secure-store'

defineEmits(['navigate'])

const bookmarks = ref([])
const showAdd = ref(false)
const newBm = reactive({ name: '', path: '', host: '' })
const ctx = reactive({ show: false, x: 0, y: 0, bm: null })

const groupedBookmarks = computed(() => {
  const groups = {}
  for (const bm of bookmarks.value) {
    const key = bm.host || '通用'
    if (!groups[key]) groups[key] = { label: key, icon: '🖥️', items: [] }
    groups[key].items.push(bm)
  }
  return Object.values(groups)
})

async function loadBookmarks() {
  try {
    const data = await getBookmarks()
    bookmarks.value = data || []
  } catch {
    bookmarks.value = []
  }
}

async function saveBookmarks() {
  try {
    await storeBookmarks(bookmarks.value)
  } catch (error) {
    console.error('Failed to save bookmarks:', error)
  }
}

function addBookmark() { newBm.name = ''; newBm.path = ''; newBm.host = ''; showAdd.value = true }

async function saveBookmark() {
  if (!newBm.path) return
  bookmarks.value.push({
    id: `bm_${Date.now()}`,
    name: newBm.name || newBm.path.split('/').pop(),
    path: newBm.path,
    host: newBm.host || '',
    icon: '📁',
  })
  await saveBookmarks()
  showAdd.value = false
}

function showMenu(e, bm) { ctx.show = true; ctx.x = e.clientX; ctx.y = e.clientY; ctx.bm = bm }
function copyPath() { navigator.clipboard.writeText(ctx.bm?.path || '').catch(() => {}); ctx.show = false }
async function delBookmark() { bookmarks.value = bookmarks.value.filter(b => b.id !== ctx.bm?.id); await saveBookmarks(); ctx.show = false }

onMounted(() => loadBookmarks())
</script>
