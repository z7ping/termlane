/** @type {import('tailwindcss').Config} */
export default {
  content: ['./index.html', './src/**/*.{vue,js,ts}'],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        terminal: {
          bg: '#1e1e1e',
          fg: '#d4d4d4',
          accent: '#007acc',
        }
      }
    }
  },
  plugins: [],
}
