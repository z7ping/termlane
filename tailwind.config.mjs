/** @type {import('tailwindcss').Config} */
export default {
  content: ['./index.html', './src/**/*.{vue,js,ts}'],
  darkMode: 'class',
  theme: {
    extend: {
      // 字体大小调整：基础字体从默认14px提升到16px
      fontSize: {
        xs: '0.75rem',   // 12px
        sm: '0.875rem',  // 14px
        base: '1rem',     // 16px
        lg: '1.125rem',  // 18px
        xl: '1.25rem',   // 20px
      },
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
