<template>
  <div class="bookmark-view h-full flex flex-col">
    <div class="view-header">
      <Bookmark :size="15" :stroke-width="1.8" />
      <span>书签</span>
      <div class="flex-1" />
      <button type="button" class="primary-action" @click="openAddDialog">
        <Plus :size="14" :stroke-width="1.8" />
        <span>添加</span>
      </button>
    </div>

    <div class="flex-1 overflow-y-auto p-3">
      <div v-if="bookmarks.length === 0" class="empty-state">
        <Bookmark :size="28" :stroke-width="1.5" />
        <div>暂无书签</div>
        <span>保存常用远程目录，之后可直接跳转。</span>
      </div>

      <div v-for="group in groupedBookmarks" v-else :key="group.label" class="bookmark-group">
        <div class="group-title">
          <Server :size="13" :stroke-width="1.8" />
          <span>{{ group.label }}</span>
          <span class="group-count">{{ group.items.length }}</span>
        </div>

        <button
          v-for="bookmark in group.items"
          :key="bookmark.id"
          type="button"
          class="bookmark-row"
          @click="navigate(bookmark)"
          @contextmenu.prevent="showMenu($event, bookmark)"
        >
          <FolderOpen :size="15" :stroke-width="1.8" />
          <div class="flex-1 min-w-0 text-left">
            <div class="bookmark-name">{{ bookmark.name }}</div>
            <div class="bookmark-path">{{ bookmark.path }}</div>
          </div>
          <span v-if="bookmark.host" class="bookmark-host">{{ bookmark.host }}</span>
        </button>
      </div>
    </div>

    <BaseModal :show="showAdd" width="340px" @close="showAdd = false">
      <div class="dialog-title">添加书签</div>
      <div class="dialog-field">
        <label>名称</label>
        <input v-model="newBookmark.name" placeholder="例如：项目根目录" />
      </div>
      <div class="dialog-field">
        <label>远程路径</label>
        <input v-model="newBookmark.path" class="font-mono" placeholder="/var/www/html" />
      </div>
      <div class="dialog-field">
        <label>连接名称或主机</label>
        <input v-model="newBookmark.host" placeholder="可选；留空时使用当前连接" />
      </div>
      <div class="dialog-actions">
        <button type="button" class="secondary-button" @click="showAdd = false">取消</button>
        <button type="button" class="primary-button" :disabled="!newBookmark.path.trim()" @click="saveBookmark">添加</button>
      </div>
    </BaseModal>

    <div v-if="context.show" class="context-backdrop" @pointerdown="closeMenu" />
    <div
      v-if="context.show"
      class="context-menu"
      role="menu"
      :style="{ left: context.x + 'px', top: context.y + 'px' }"
      @pointerdown.stop
    >
      <button type="button" role="menuitem" @click="navigate(context.bookmark)">
        <FolderOpen :size="14" :stroke-width="1.8" /><span>打开</span>
      </button>
      <button type="button" role="menuitem" @click="copyPath">
        <Copy :size="14" :stroke-width="1.8" /><span>复制路径</span>
      </button>
      <div class="menu-separator" />
      <button type="button" role="menuitem" class="danger" @click="deleteBookmark">
        <Trash2 :size="14" :stroke-width="1.8" /><span>删除</span>
      </button>
    </div>
  </div>
</template>

<script setup>
import { computed, onMounted, reactive, ref } from 'vue'
import { Bookmark, Copy, FolderOpen, Plus, Server, Trash2 } from 'lucide-vue-next'
import { getBookmarks as getLegacyBookmarks } from '../utils/secure-store-browser'
import { STORAGE_KEYS } from '@/utils/storage-keys'
import BaseModal from './BaseModal.vue'

const emit = defineEmits(['navigate'])

const bookmarks = ref([])
const showAdd = ref(false)
const newBookmark = reactive({ name: '', path: '', host: '' })
const context = reactive({ show: false, x: 0, y: 0, bookmark: null })

const groupedBookmarks = computed(() => {
  const groups = new Map()
  for (const bookmark of bookmarks.value) {
    const label = bookmark.host?.trim() || '当前连接 / 通用'
    if (!groups.has(label)) groups.set(label, { label, items: [] })
    groups.get(label).items.push(bookmark)
  }
  return [...groups.values()]
})

function normalizeBookmarks(value) {
  if (!Array.isArray(value)) return []
  return value.filter(bookmark =>
    bookmark
    && typeof bookmark.path === 'string'
    && typeof bookmark.name === 'string',
  )
}

async function loadBookmarks() {
  const raw = localStorage.getItem(STORAGE_KEYS.BOOKMARKS)
  if (raw != null) {
    try {
      bookmarks.value = normalizeBookmarks(JSON.parse(raw))
      return
    } catch {
      bookmarks.value = []
    }
  }

  // 历史版本把普通书签误存进 browser secure fallback。
  // 当前会话若仍能解密，则迁移到稳定普通持久化；旧密文不主动删除。
  try {
    const legacy = normalizeBookmarks(await getLegacyBookmarks())
    if (legacy.length > 0) {
      bookmarks.value = legacy
      persistBookmarks()
      return
    }
  } catch {}

  bookmarks.value = []
}

function persistBookmarks() {
  localStorage.setItem(STORAGE_KEYS.BOOKMARKS, JSON.stringify(bookmarks.value))
}

function openAddDialog() {
  newBookmark.name = ''
  newBookmark.path = ''
  newBookmark.host = ''
  showAdd.value = true
}

function saveBookmark() {
  const path = newBookmark.path.trim()
  if (!path) return

  bookmarks.value.push({
    id: crypto.randomUUID(),
    name: newBookmark.name.trim() || path.split('/').filter(Boolean).pop() || path,
    path,
    host: newBookmark.host.trim(),
  })
  persistBookmarks()
  showAdd.value = false
}

function navigate(bookmark) {
  if (!bookmark) return
  closeMenu()
  emit('navigate', bookmark)
}

function showMenu(event, bookmark) {
  context.show = true
  context.x = Math.min(event.clientX, window.innerWidth - 160)
  context.y = Math.min(event.clientY, window.innerHeight - 120)
  context.bookmark = bookmark
}

function closeMenu() {
  context.show = false
}

async function copyPath() {
  const path = context.bookmark?.path
  closeMenu()
  if (!path) return
  await navigator.clipboard.writeText(path).catch(() => {})
}

function deleteBookmark() {
  const id = context.bookmark?.id
  closeMenu()
  if (!id) return
  bookmarks.value = bookmarks.value.filter(bookmark => bookmark.id !== id)
  persistBookmarks()
}

onMounted(loadBookmarks)
</script>

<style scoped>
.bookmark-view {
  position: relative;
  background: var(--bg-base);
  color: var(--fg-secondary);
}

.view-header {
  height: 36px;
  display: flex;
  align-items: center;
  gap: 7px;
  padding: 0 10px;
  flex-shrink: 0;
  border-bottom: 1px solid var(--border-subtle);
  background: var(--bg-surface);
  color: var(--fg-secondary);
  font-size: 12px;
  font-weight: 500;
}

.primary-action,
.secondary-button,
.primary-button {
  height: 28px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 5px;
  padding: 0 9px;
  border: 0;
  border-radius: 6px;
  font-size: 11px;
}

.primary-action,
.primary-button {
  background: var(--accent);
  color: white;
}

.primary-button:disabled {
  cursor: not-allowed;
  opacity: 0.4;
}

.secondary-button {
  background: var(--bg-hover);
  color: var(--fg-secondary);
}

.empty-state {
  min-height: 220px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 7px;
  color: var(--fg-muted);
  font-size: 12px;
}

.empty-state span {
  font-size: 11px;
}

.bookmark-group {
  margin-bottom: 14px;
}

.group-title {
  height: 28px;
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 0 7px;
  color: var(--fg-muted);
  font-size: 10px;
  font-weight: 600;
}

.group-count {
  margin-left: 2px;
  padding: 0 5px;
  border-radius: 8px;
  background: var(--bg-hover);
  font-weight: 400;
}

.bookmark-row {
  width: 100%;
  min-height: 44px;
  display: flex;
  align-items: center;
  gap: 9px;
  padding: 6px 9px;
  border: 0;
  border-radius: 7px;
  background: transparent;
  color: var(--fg-muted);
  transition: background-color var(--transition-fast), color var(--transition-fast);
}

.bookmark-row:hover {
  background: var(--bg-hover);
  color: var(--accent);
}

.bookmark-name {
  overflow: hidden;
  color: var(--fg-primary);
  font-size: 12px;
  font-weight: 500;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.bookmark-path {
  margin-top: 2px;
  overflow: hidden;
  color: var(--fg-muted);
  font-family: monospace;
  font-size: 10px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.bookmark-host {
  max-width: 150px;
  overflow: hidden;
  color: var(--fg-muted);
  font-size: 10px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.dialog-title {
  color: var(--fg-primary);
  font-size: 13px;
  font-weight: 600;
}

.dialog-field {
  display: flex;
  flex-direction: column;
  gap: 5px;
}

.dialog-field label {
  color: var(--fg-muted);
  font-size: 10px;
}

.dialog-field input {
  width: 100%;
  height: 31px;
  padding: 0 9px;
  border: 1px solid var(--border);
  border-radius: 6px;
  outline: none;
  background: var(--bg-base);
  color: var(--fg-primary);
  font-size: 12px;
}

.dialog-field input:focus {
  border-color: var(--accent);
}

.dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 7px;
}

.context-backdrop {
  position: fixed;
  inset: 0;
  z-index: 40;
}

.context-menu {
  position: fixed;
  z-index: 50;
  min-width: 150px;
  padding: 5px;
  border: 1px solid var(--border);
  border-radius: 8px;
  background: var(--bg-elevated);
  box-shadow: var(--shadow-lg);
}

.context-menu button {
  width: 100%;
  height: 30px;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 8px;
  border: 0;
  border-radius: 5px;
  background: transparent;
  color: var(--fg-secondary);
  font-size: 12px;
  text-align: left;
}

.context-menu button:hover {
  background: var(--bg-hover);
  color: var(--fg-primary);
}

.context-menu button.danger {
  color: var(--danger);
}

.menu-separator {
  height: 1px;
  margin: 4px 2px;
  background: var(--border-subtle);
}
</style>
