// 定时任务调度器
import { STORAGE_KEYS } from './storage-keys'

export function getScheduledTasks() {
  try { return JSON.parse(localStorage.getItem(STORAGE_KEY)) || [] }
  catch { return [] }
}

export function saveScheduledTasks(tasks) {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(tasks))
}

export function addScheduledTask(task) {
  const tasks = getScheduledTasks()
  const newTask = {
    id: Date.now().toString() + '_' + Math.random().toString(36).substr(2, 9),
    name: task.name,
    command: task.command,
    schedule: task.schedule, // cron-like: '0 9 * * *' = every day at 9am
    connectionId: task.connectionId,
    enabled: true,
    lastRun: null,
    nextRun: calculateNextRun(task.schedule),
    createdAt: new Date().toISOString(),
  }
  tasks.push(newTask)
  saveScheduledTasks(tasks)
  return tasks
}

export function removeScheduledTask(id) {
  const tasks = getScheduledTasks().filter(t => t.id !== id)
  saveScheduledTasks(tasks)
  return tasks
}

export function toggleScheduledTask(id) {
  const tasks = getScheduledTasks().map(t => {
    if (t.id === id) {
      return { ...t, enabled: !t.enabled }
    }
    return { ...t }
  })
  saveScheduledTasks(tasks)
  return tasks
}

function calculateNextRun(schedule) {
  // Simple implementation - in production would use a proper cron parser
  const now = new Date()
  const parts = schedule.split(' ')
  if (parts.length >= 2) {
    const hour = parseInt(parts[1]) || 0
    const next = new Date(now)
    next.setHours(hour, 0, 0, 0)
    if (next <= now) next.setDate(next.getDate() + 1)
    return next.toISOString()
  }
  return new Date(now.getTime() + 60 * 60 * 1000).toISOString() // default: 1 hour
}
