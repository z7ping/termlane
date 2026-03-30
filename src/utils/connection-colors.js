// utils/connection-colors.js - 连接颜色标签

export const connectionColors = [
  { value: 'none', label: '默认', class: '' },
  { value: 'red', label: '生产环境', class: 'text-red-400', bg: 'bg-red-600/20' },
  { value: 'yellow', label: '测试环境', class: 'text-yellow-400', bg: 'bg-yellow-600/20' },
  { value: 'green', label: '开发环境', class: 'text-green-400', bg: 'bg-green-600/20' },
  { value: 'blue', label: '数据库', class: 'text-blue-400', bg: 'bg-blue-600/20' },
  { value: 'purple', label: '容器', class: 'text-purple-400', bg: 'bg-purple-600/20' },
]

export function getColorClass(colorValue) {
  return connectionColors.find(c => c.value === colorValue) || connectionColors[0]
}

export function getColorDot(colorValue) {
  const map = {
    red: '🔴', yellow: '🟡', green: '🟢', blue: '🔵', purple: '🟣', none: ''
  }
  return map[colorValue] || ''
}
