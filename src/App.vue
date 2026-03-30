<template>
  <div class="h-screen flex flex-col" :class="{ dark: isDark }">
    <!-- Title Bar -->
    <TitleBar @toggle-sidebar="sidebarOpen = !sidebarOpen" />

    <div class="flex flex-1 overflow-hidden">
      <!-- Sidebar -->
      <Sidebar
        v-if="sidebarOpen"
        :connections="connections"
        :active-id="activeConnectionId"
        @select="onSelectConnection"
        @add="showAddConnection = true"
        @delete="onDeleteConnection"
      />

      <!-- Main Content -->
      <div class="flex-1 flex flex-col overflow-hidden bg-gray-900">
        <!-- Tabs -->
        <TabBar
          :tabs="tabs"
          :active-id="activeTabId"
          @select="activeTabId = $event"
          @close="closeTab"
          @new="openLocalTerminal"
        />

        <!-- Terminal Area -->
        <div class="flex-1 relative overflow-hidden">
          <TerminalPanel
            v-for="tab in tabs"
            :key="tab.id"
            :tab="tab"
            :active="tab.id === activeTabId"
          />
          <div v-if="tabs.length === 0" class="h-full flex items-center justify-center text-gray-500">
            <div class="text-center">
              <div class="text-6xl mb-4">⌨️</div>
              <div class="text-lg">XTerminal Pro</div>
              <div class="text-sm mt-2">从左侧选择一个连接，或按 + 打开本地终端</div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Status Bar -->
    <StatusBar :connection="activeConnection" />

    <!-- Add Connection Dialog -->
    <ConnectionDialog
      v-if="showAddConnection"
      @save="onSaveConnection"
      @close="showAddConnection = false"
    />
  </div>
</template>

<script setup>
import { ref, computed } from 'vue'
import TitleBar from './components/TitleBar.vue'
import Sidebar from './components/Sidebar.vue'
import TabBar from './components/TabBar.vue'
import TerminalPanel from './components/TerminalPanel.vue'
import StatusBar from './components/StatusBar.vue'
import ConnectionDialog from './components/ConnectionDialog.vue'

const isDark = ref(true)
const sidebarOpen = ref(true)
const showAddConnection = ref(false)

// Connections
const connections = ref([
  { id: 'local', name: '本地终端', host: 'localhost', group: '本地', icon: '💻' },
])

// Tabs
const tabs = ref([])
const activeTabId = ref(null)

const activeTab = computed(() => tabs.value.find(t => t.id === activeTabId.value))
const activeConnectionId = ref(null)
const activeConnection = computed(() => connections.value.find(c => c.id === activeConnectionId.value))

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
  connections.value.push({ ...conn, id: `conn_${Date.now()}` })
  showAddConnection.value = false
}

function onDeleteConnection(id) {
  connections.value = connections.value.filter(c => c.id !== id)
  tabs.value = tabs.value.filter(t => t.connectionId !== id)
}
</script>
