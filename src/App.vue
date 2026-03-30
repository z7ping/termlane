<template>
  <div class="h-screen flex flex-col" :class="{ dark: isDark }">
    <TitleBar @toggle-sidebar="sidebarOpen = !sidebarOpen" />

    <div class="flex flex-1 overflow-hidden">
      <Sidebar
        v-if="sidebarOpen"
        :connections="connections"
        :active-id="activeConnectionId"
        @select="onSelectConnection"
        @add="showAddConnection = true"
        @delete="onDeleteConnection"
      />

      <div class="flex-1 flex flex-col overflow-hidden bg-gray-900">
        <!-- View Mode Tabs -->
        <div class="h-9 bg-gray-800 border-b border-gray-700 flex items-center px-2">
          <TabBar
            :tabs="tabs"
            :active-id="activeTabId"
            @select="activeTabId = $event"
            @close="closeTab"
            @new="openLocalTerminal"
            class="flex-1"
          />
          <!-- View Mode Switch -->
          <div v-if="activeTab" class="flex gap-1 ml-2">
            <button
              v-for="vm in viewModes"
              :key="vm.value"
              @click="viewMode = vm.value"
              class="px-2 py-0.5 text-xs rounded"
              :class="viewMode === vm.value ? 'bg-blue-600 text-white' : 'bg-gray-700 text-gray-400 hover:bg-gray-600'"
            >{{ vm.label }}</button>
          </div>
        </div>

        <!-- Content Area -->
        <div class="flex-1 relative overflow-hidden">
          <!-- Terminal View -->
          <template v-if="viewMode === 'terminal'">
            <TerminalPanel
              v-for="tab in tabs"
              :key="tab.id"
              :tab="tab"
              :active="tab.id === activeTabId"
              @connected="onSessionConnected(tab.id, $event)"
              @disconnected="onSessionDisconnected(tab.id)"
            />
          </template>

          <!-- SFTP View -->
          <SftpPanel
            v-if="viewMode === 'sftp'"
            :connection="activeConnection"
            :session-id="activeSessionId"
            :active="true"
          />

          <!-- Empty State -->
          <div v-if="tabs.length === 0" class="h-full flex items-center justify-center text-gray-500">
            <div class="text-center">
              <div class="text-6xl mb-4">⌨️</div>
              <div class="text-lg">XTerminal Pro</div>
              <div class="text-sm mt-2">从左侧选择一个连接，或按 + 打开本地终端</div>
              <div class="text-xs mt-4 text-gray-600">Tauri + Vue 3 + xterm.js</div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <StatusBar :connection="activeConnection" :session-id="activeSessionId" />

    <ConnectionDialog
      v-if="showAddConnection"
      @save="onSaveConnection"
      @close="showAddConnection = false"
    />
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { invoke } from './utils/tauri.js'
import TitleBar from './components/TitleBar.vue'
import Sidebar from './components/Sidebar.vue'
import TabBar from './components/TabBar.vue'
import TerminalPanel from './components/TerminalPanel.vue'
import SftpPanel from './components/SftpPanel.vue'
import StatusBar from './components/StatusBar.vue'
import ConnectionDialog from './components/ConnectionDialog.vue'
import UpdateNotifier from './components/UpdateNotifier.vue'

const isDark = ref(true)
const sidebarOpen = ref(true)
const showAddConnection = ref(false)
const viewMode = ref('terminal')

const viewModes = [
  { value: 'terminal', label: '⌨️ 终端' },
  { value: 'sftp', label: '📁 文件' },
]

const connections = ref([
  { id: 'local', name: '本地终端', host: 'localhost', port: 22, username: 'local', authType: 'local', group: '本地', icon: '💻' },
])

const tabs = ref([])
const activeTabId = ref(null)
const sessionMap = ref({})

const activeTab = computed(() => tabs.value.find(t => t.id === activeTabId.value))
const activeConnectionId = ref(null)
const activeConnection = computed(() => connections.value.find(c => c.id === activeConnectionId.value))
const activeSessionId = computed(() => sessionMap.value[activeTabId.value] || null)

onMounted(async () => {
  try {
    const saved = await invoke('load_connections')
    if (saved && saved.length > 0) {
      const localExists = saved.some(c => c.id === 'local')
      connections.value = localExists ? saved : [
        { id: 'local', name: '本地终端', host: 'localhost', port: 22, username: 'local', authType: 'local', group: '本地', icon: '💻' },
        ...saved,
      ]
    }
  } catch (e) { /* ignore */ }
})

function onSelectConnection(conn) {
  activeConnectionId.value = conn.id
  const existing = tabs.value.find(t => t.connectionId === conn.id)
  if (existing) {
    activeTabId.value = existing.id
  } else {
    const tab = {
      id: `tab_${Date.now()}`,
      name: conn.name,
      connectionId: conn.id,
      connection: conn,
      type: 'terminal',
    }
    tabs.value.push(tab)
    activeTabId.value = tab.id
  }
}

function closeTab(tabId) {
  const sid = sessionMap.value[tabId]
  if (sid) {
    invoke('ssh_disconnect', { sessionId: sid }).catch(() => {})
    delete sessionMap.value[tabId]
  }
  const idx = tabs.value.findIndex(t => t.id === tabId)
  tabs.value.splice(idx, 1)
  if (activeTabId.value === tabId) {
    activeTabId.value = tabs.value[Math.min(idx, tabs.value.length - 1)]?.id || null
  }
}

function openLocalTerminal() {
  const local = connections.value.find(c => c.id === 'local')
  if (local) onSelectConnection(local)
}

function onSaveConnection(conn) {
  const newConn = { ...conn, id: `conn_${Date.now()}` }
  connections.value.push(newConn)
  showAddConnection.value = false
  invoke('save_connection', { conn: newConn }).catch(() => {})
}

function onDeleteConnection(id) {
  connections.value = connections.value.filter(c => c.id !== id)
  tabs.value = tabs.value.filter(t => t.connectionId !== id)
  invoke('delete_connection', { id }).catch(() => {})
}

function onSessionConnected(tabId, sid) {
  sessionMap.value[tabId] = sid
}

function onSessionDisconnected(tabId) {
  delete sessionMap.value[tabId]
}
</script>
