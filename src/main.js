import { createApp } from 'vue'
import App from './App.vue'
import './style.css'

// Apply theme immediately to prevent FOUC
const theme = localStorage.getItem('xterminal-theme') || 'dark'
document.documentElement.setAttribute('data-theme', theme)

createApp(App).mount('#app')
