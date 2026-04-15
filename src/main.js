import { createApp } from 'vue'
import App from './App.vue'
import './style.css'
import { autoMigrate } from './utils/migrate-storage'

// Apply theme immediately to prevent FOUC
const theme = localStorage.getItem('xterminal-theme') || 'dark'
document.documentElement.setAttribute('data-theme', theme)

// Auto-migrate storage to secure version
autoMigrate().catch(error => {
  console.error('Storage migration failed:', error)
})

createApp(App).mount('#app')
