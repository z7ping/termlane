<template>
  <div class="h-full flex flex-col" style="background: var(--bg-base)">
    <div class="h-9 border-b flex items-center px-3 gap-2" style="background: var(--bg-surface); border-color: var(--border)">
      <span class="text-sm font-medium" style="color: var(--fg-secondary)">🔖 书签</span>
      <div class="flex-1" />
      <button @click="addBookmark" class="text-xs px-2 py-0.5 rounded" style="background: var(--accent); color: white">+ 添加</button>
    </div>

    <div class="flex-1 overflow-y-auto p-2">
      <div v-for="(group, gIdx) in groupedBookmarks" :key="gIdx" class="mb-3">
        <div class="text-xs px-2 py-1 flex items-center gap-1" style="color: var(--fg-muted)">
          <span>{{ group.icon }}</span> {{ group.label }}
        </div>
        <div v-for="bm in group.items" :key="bm.id" @click="$emit('navigate', bm)"
          @contextmenu.prevent="showMenu($event, bm)"
          class="flex items-center gap-2 px-3 py-1.5 rounded cursor-pointer text-sm group" style="background: var(--bg-surface)">
          <span class="" style="color: var(--fg-muted)">{{ bm.icon || '📁' }}</span>
          <div class="flex-1 min-w-0">
            <div class="truncate" style="color: var(--fg-primary)">{{ bm.name }}</div>
            <div class="truncate text-xs font-mono" style="color: var(--fg-muted)">{{ bm.path }}</div>
          </div>
          <span class="text-xs" style="color: var(--fg-muted)">{{ bm.host }}</span>
        </div>
      </div>

      <div v-if="bookmarks.length === 0" class="text-center text-sm mt-10" style="color: var(--fg-muted)">
        暂无书签<br/><span class="text-xs">SFTP 中右键添加常用目录</span>
      </div>
    </div>

    <!-- Add Dialog -->
    <BaseModal :show="showAdd" width="320px" @close="showAdd = false">
      <h3 class="text-sm font-medium">添加书签</h3>
      <input v-model="newBm.name" class="w-full rounded px-3 py-1.5 text-sm focus:outline-none" style="background: var(--bg-base); border-color: var(--border-subtle)" placeholder="名称（如：项目根目录）" />
      <input v-model="newBm.path" class="w-full rounded px-3 py-1.5 text-sm font-mono focus:outline-none" style="background: var(--bg-base); border-color: var(--border-subtle)" placeholder="/var/www/html" />
      <input v-model="newBm.host" class="w-full rounded px-3 py-1.5 text-sm focus:outline-none" style="background: var(--bg-base); border-color: var(--border-subtle)" placeholder="服务器名（可选）" />
      <div class="flex justify-end gap-2">
        <button @click="showAdd = false" class="px-3 py-1 text-sm" style="color: var(--fg-muted)">取消</button>
        <button @click="saveBookmark" :disabled="!newBm.path" class="px-3 py-1 text-sm rounded disabled:opacity-50" style="background: var(--accent); color: white">添加</button>
      </div>
    </BaseModal>

    <!-- Context Menu -->
    <div v-if="ctx.show" :style="{ left: ctx.x + 'px', top: ctx.y + 'px' }" class="fixed rounded shadow-lg z-50 py-1 text-xs min-w-[120px]" style="background: var(--bg-surface); border-color: var(--border-subtle)" @click.stop>
      <div @click="$emit('navigate', ctx.bm); ctx.show = false" class="px-4 py-1.5 cursor-pointer" style="color: var(--fg-secondary)">📂 打开</div>
      <div @click="copyPath" class="px-4 py-1.5 cursor-pointer" style="color: var(--fg-secondary)"">📋 复制路径</div>
      <div class="border-t my-1" style="border-color: var(--border)" />
      <div @click="delBookmark" class="px-4 py-1.5 cursor-pointer" style="color: var(--danger)">🗑 删除</div>
    </div>
    <div v-if="ctx.show" class="fixed inset-0 z-40" @click="ctx.show = false" />
  </div>
</template>

<script setup>
import { ref, reactive, computed, onMounted } from 'vue'
import { getBookmarks, storeBookmarks } from '../utils/secure-store-browser'
import BaseModal from './BaseModal.vue'

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
