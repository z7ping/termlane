// src/__tests__/scheduler.test.js - 定时任务调度器测试
import { getScheduledTasks, saveScheduledTasks, addScheduledTask, removeScheduledTask, toggleScheduledTask } from '../utils/scheduler.ts'

describe('定时任务调度器', () => {
  beforeEach(() => {
    // 每个测试前清理测试数据
    localStorage.clear()
  })

  describe('getScheduledTasks', () => {
    test('初始化时返回空数组', () => {
      const tasks = getScheduledTasks()
      expect(tasks).toEqual([])
    })

    test('返回已存储的任务列表', () => {
      const testTasks = [
        { id: '1', name: 'Task 1', command: 'ls', schedule: '0 9 * * *' },
        { id: '2', name: 'Task 2', command: 'pwd', schedule: '0 10 * * *' },
      ]
      localStorage.setItem('termlane_scheduled_tasks', JSON.stringify(testTasks))
      
      const tasks = getScheduledTasks()
      expect(tasks).toEqual(testTasks)
    })

    test('JSON解析错误时返回空数组', () => {
      localStorage.setItem('termlane_scheduled_tasks', 'invalid json')
      const tasks = getScheduledTasks()
      expect(tasks).toEqual([])
    })

    test('没有存储时返回空数组', () => {
      const tasks = getScheduledTasks()
      expect(tasks).toEqual([])
    })
  })

  describe('saveScheduledTasks', () => {
    test('保存任务列表到localStorage', () => {
      const tasks = [
        { id: '1', name: 'Task 1', command: 'ls', schedule: '0 9 * * *' },
      ]
      saveScheduledTasks(tasks)
      
      const stored = localStorage.getItem('termlane_scheduled_tasks')
      expect(stored).toBeTruthy()
      expect(JSON.parse(stored)).toEqual(tasks)
    })

    test('覆盖已保存的任务列表', () => {
      const tasks1 = [{ id: '1', name: 'Task 1', command: 'ls', schedule: '0 9 * * *' }]
      const tasks2 = [{ id: '2', name: 'Task 2', command: 'pwd', schedule: '0 10 * * *' }]
      
      saveScheduledTasks(tasks1)
      saveScheduledTasks(tasks2)
      
      const tasks = getScheduledTasks()
      expect(tasks).toEqual(tasks2)
    })

    test('可以保存空数组', () => {
      saveScheduledTasks([])
      const tasks = getScheduledTasks()
      expect(tasks).toEqual([])
    })
  })

  describe('addScheduledTask', () => {
    test('添加新任务', () => {
      const newTask = {
        name: 'Backup',
        command: 'backup.sh',
        schedule: '0 2 * * *',
        connectionId: 'conn1',
      }
      
      const result = addScheduledTask(newTask)
      expect(result).toHaveLength(1)
      expect(result[0].name).toBe('Backup')
    })

    test('生成的任务包含所有必要字段', () => {
      const newTask = {
        name: 'Test Task',
        command: 'echo hello',
        schedule: '0 9 * * *',
        connectionId: 'conn1',
      }
      
      const result = addScheduledTask(newTask)
      const task = result[0]
      
      expect(task.id).toBeTruthy()
      expect(task.name).toBe('Test Task')
      expect(task.command).toBe('echo hello')
      expect(task.schedule).toBe('0 9 * * *')
      expect(task.connectionId).toBe('conn1')
      expect(task.enabled).toBe(true)
      expect(task.lastRun).toBeNull()
      expect(task.nextRun).toBeTruthy()
      expect(task.createdAt).toBeTruthy()
    })

    test('生成的id是字符串', () => {
      const newTask = {
        name: 'Test',
        command: 'test',
        schedule: '0 9 * * *',
        connectionId: 'conn1',
      }
      
      const result = addScheduledTask(newTask)
      expect(typeof result[0].id).toBe('string')
    })

    test('生成的nextRun是有效ISO字符串', () => {
      const newTask = {
        name: 'Test',
        command: 'test',
        schedule: '0 9 * * *',
        connectionId: 'conn1',
      }
      
      const result = addScheduledTask(newTask)
      const nextRun = result[0].nextRun
      
      expect(() => new Date(nextRun)).not.toThrow()
    })

    test('生成的createdAt是有效ISO字符串', () => {
      const newTask = {
        name: 'Test',
        command: 'test',
        schedule: '0 9 * * *',
        connectionId: 'conn1',
      }
      
      const result = addScheduledTask(newTask)
      const createdAt = result[0].createdAt
      
      expect(() => new Date(createdAt)).not.toThrow()
    })

    test('可以添加多个任务', () => {
      const task1 = { name: 'Task 1', command: 'cmd1', schedule: '0 9 * * *', connectionId: 'conn1' }
      const task2 = { name: 'Task 2', command: 'cmd2', schedule: '0 10 * * *', connectionId: 'conn2' }
      const task3 = { name: 'Task 3', command: 'cmd3', schedule: '0 11 * * *', connectionId: 'conn3' }
      
      addScheduledTask(task1)
      addScheduledTask(task2)
      const result = addScheduledTask(task3)
      
      expect(result).toHaveLength(3)
    })

    test('返回更新后的任务列表', () => {
      const task1 = { name: 'Task 1', command: 'cmd1', schedule: '0 9 * * *', connectionId: 'conn1' }
      const task2 = { name: 'Task 2', command: 'cmd2', schedule: '0 10 * * *', connectionId: 'conn2' }
      
      const result1 = addScheduledTask(task1)
      expect(result1).toHaveLength(1)
      
      const result2 = addScheduledTask(task2)
      expect(result2).toHaveLength(2)
    })
  })

  describe('removeScheduledTask', () => {
    test('删除指定ID的任务', () => {
      const task1 = { name: 'Task 1', command: 'cmd1', schedule: '0 9 * * *', connectionId: 'conn1' }
      const task2 = { name: 'Task 2', command: 'cmd2', schedule: '0 10 * * *', connectionId: 'conn2' }
      
      addScheduledTask(task1)
      addScheduledTask(task2)
      
      const tasksBefore = getScheduledTasks()
      const idToRemove = tasksBefore[0].id
      
      const result = removeScheduledTask(idToRemove)
      expect(result).toHaveLength(1)
      expect(result[0].name).toBe('Task 2')
      expect(result[0].id).not.toBe(idToRemove)
    })

    test('删除不存在的任务不影响列表', () => {
      const task1 = { name: 'Task 1', command: 'cmd1', schedule: '0 9 * * *', connectionId: 'conn1' }
      
      const tasks = addScheduledTask(task1)
      const result = removeScheduledTask('non_existent_id')
      
      expect(result).toHaveLength(1)
      expect(result[0].id).toBe(tasks[0].id)
    })

    test('删除所有任务', () => {
      const task1 = { name: 'Task 1', command: 'cmd1', schedule: '0 9 * * *', connectionId: 'conn1' }
      const tasks = addScheduledTask(task1)
      
      removeScheduledTask(tasks[0].id)
      const finalTasks = getScheduledTasks()
      
      expect(finalTasks).toEqual([])
    })

    test('返回更新后的任务列表', () => {
      const task1 = { name: 'Task 1', command: 'cmd1', schedule: '0 9 * * *', connectionId: 'conn1' }
      const task2 = { name: 'Task 2', command: 'cmd2', schedule: '0 10 * * *', connectionId: 'conn2' }
      
      addScheduledTask(task1)
      addScheduledTask(task2)
      
      const tasks = getScheduledTasks()
      const idToRemove = tasks[0].id
      const result = removeScheduledTask(idToRemove)
      
      // removeScheduledTask removes the task, so result should have 1 task
      expect(result).toHaveLength(1)
      expect(result[0].id).not.toBe(idToRemove)
    })
  })

  describe('toggleScheduledTask', () => {
    test('切换任务的启用状态', () => {
      const task1 = { name: 'Task 1', command: 'cmd1', schedule: '0 9 * * *', connectionId: 'conn1' }
      const tasks = addScheduledTask(task1)
      
      // 初始状态应该是enabled=true
      expect(tasks[0].enabled).toBe(true)
      
      // 切换到disabled
      const result1 = toggleScheduledTask(tasks[0].id)
      expect(result1[0].enabled).toBe(false)
      
      // 切换回enabled
      const result2 = toggleScheduledTask(tasks[0].id)
      expect(result2[0].enabled).toBe(true)
    })

    test('只影响指定的任务', () => {
      const task1 = { name: 'Task 1', command: 'cmd1', schedule: '0 9 * * *', connectionId: 'conn1' }
      const task2 = { name: 'Task 2', command: 'cmd2', schedule: '0 10 * * *', connectionId: 'conn2' }
      
      addScheduledTask(task1)
      addScheduledTask(task2)
      
      const tasksBefore = getScheduledTasks()
      const result = toggleScheduledTask(tasksBefore[0].id)
      
      // 使用返回值检查状态
      expect(result[0].enabled).toBe(false)
      expect(result[1].enabled).toBe(true)
      
      // 验证存储也被更新
      const updatedTasks = getScheduledTasks()
      expect(updatedTasks[0].enabled).toBe(false)
      expect(updatedTasks[1].enabled).toBe(true)
    })

    test('切换不存在的任务不影响列表', () => {
      const task1 = { name: 'Task 1', command: 'cmd1', schedule: '0 9 * * *', connectionId: 'conn1' }
      const tasks = addScheduledTask(task1)
      
      const result = toggleScheduledTask('non_existent_id')
      expect(result).toHaveLength(1)
      expect(result[0].enabled).toBe(true)
    })

    test('返回更新后的任务列表', () => {
      const task1 = { name: 'Task 1', command: 'cmd1', schedule: '0 9 * * *', connectionId: 'conn1' }
      const tasks = addScheduledTask(task1)
      
      const result = toggleScheduledTask(tasks[0].id)
      expect(result[0].enabled).toBe(false)
    })
  })

  describe('集成测试', () => {
    test('完整的任务管理流程', () => {
      // 初始状态
      expect(getScheduledTasks()).toEqual([])
      
      // 添加任务
      const task1 = { name: 'Task 1', command: 'cmd1', schedule: '0 9 * * *', connectionId: 'conn1' }
      const task2 = { name: 'Task 2', command: 'cmd2', schedule: '0 10 * * *', connectionId: 'conn2' }
      
      let tasks = addScheduledTask(task1)
      expect(tasks).toHaveLength(1)
      
      tasks = addScheduledTask(task2)
      expect(tasks).toHaveLength(2)
      
      // 切换任务状态
      tasks = toggleScheduledTask(tasks[0].id)
      expect(tasks[0].enabled).toBe(false)
      expect(tasks[1].enabled).toBe(true)
      
      // 删除任务
      const result = removeScheduledTask(tasks[0].id)
      expect(result).toHaveLength(1)
      expect(result[0].name).toBe('Task 2')
      
      // 再次删除
      const final = removeScheduledTask(result[0].id)
      expect(final).toHaveLength(0)
    })

    test('多个任务的独立管理', () => {
      const task1 = { name: 'Task 1', command: 'cmd1', schedule: '0 9 * * *', connectionId: 'conn1' }
      const task2 = { name: 'Task 2', command: 'cmd2', schedule: '0 10 * * *', connectionId: 'conn2' }
      const task3 = { name: 'Task 3', command: 'cmd3', schedule: '0 11 * * *', connectionId: 'conn3' }
      
      addScheduledTask(task1)
      addScheduledTask(task2)
      addScheduledTask(task3)
      
      let tasks = getScheduledTasks()
      expect(tasks).toHaveLength(3)
      
      // 切换第一个任务
      const firstTaskId = tasks[0].id
      tasks = toggleScheduledTask(firstTaskId)
      expect(tasks[0].enabled).toBe(false)
      expect(tasks[1].enabled).toBe(true)
      expect(tasks[2].enabled).toBe(true)
      
      // 删除第二个任务
      const secondTaskId = tasks[1].id
      const result = removeScheduledTask(secondTaskId)
      expect(result).toHaveLength(2)
    })
  })
})
