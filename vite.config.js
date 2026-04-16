import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

export default defineConfig({
  plugins: [vue()],
  server: {
    port: 1420,
    strictPort: true,
    host: 'localhost'
  },
  clearScreen: false,
  envPrefix: ['VITE_', 'TAURI_']
})