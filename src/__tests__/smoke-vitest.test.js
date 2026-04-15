// src/__tests__/smoke-vitest.test.js — 冒烟测试：验证关键模块导出和 mock 功能
// 转换为vitest格式

describe('冒烟测试', () => {
  
  // ─── 1. tauri.js ───
  describe('tauri.js — 导出验证', () => {
    test('invoke 导出为函数', async () => {
      const tauri = await import('../utils/tauri.js');
      expect(typeof tauri.invoke).toBe('function');
    });

    test('() 导出为函数', async () => {
      const tauri = await import('../utils/tauri.js');
      expect(typeof tauri.listen).toBe('function');
    });

    test('isTauri 导出为布尔值', async () => {
      const tauri = await import('../utils/tauri.js');
      expect(typeof tauri.isTauri).toBe('boolean');
    });

    test('Node.js 环境下 isTauri 应为 false', async () => {
      const tauri = await import('../utils/tauri.js');
      expect(tauri.isTauri).toBe(false);
    });
  });

  // ─── 2. Mock 命令 ───
  describe('Mock 命令 — 数据验证', () => {
    test('ssh_connect 返回字符串', async () => {
      const tauri = await import('../utils/tauri.js');
      const sessionId = await tauri.invoke('ssh_connect', { host: 'localhost' });
      expect(typeof sessionId).toBe('string');
    });

    test('ssh_connect 返回 mock_session_ 前缀', async () => {
      const tauri = await import('../utils/tauri.js');
      const sessionId = await tauri.invoke('ssh_connect', { host: 'localhost' });
      expect(sessionId.startsWith('mock_session_')).toBe(true);
    });

    test('load_connections 返回数组', async () => {
      const tauri = await import('../utils/tauri.js');
      const connections = await tauri.invoke('load_connections');
      expect(Array.isArray(connections)).toBe(true);
    });

    test('load_connections 至少包含 1 条连接', async () => {
      const tauri = await import('../utils/tauri.js');
      const connections = await tauri.invoke('load_connections');
      expect(connections.length).toBeGreaterThan(0);
    });

    test('连接对象包含 id 字段', async () => {
      const tauri = await import('../utils/tauri.js');
      const connections = await tauri.invoke('load_connections');
      expect(connections[0].id).toBeTruthy();
    });

    test('ssh_monitor 返回对象', async () => {
      const tauri = await import('../utils/tauri.js');
      const monitor = await tauri.invoke('ssh_monitor', { session_id: 'test' });
      expect(typeof monitor).toBe('object');
    });

    test('ssh_monitor 包含 cpu_usage (number)', async () => {
      const tauri = await import('../utils/tauri.js');
      const monitor = await tauri.invoke('ssh_monitor', { session_id: 'test' });
      expect(typeof monitor.cpu_usage).toBe('number');
    });

    test('cpu_usage 为正数', async () => {
      const tauri = await import('../utils/tauri.js');
      const monitor = await tauri.invoke('ssh_monitor', { session_id: 'test' });
      expect(monitor.cpu_usage).toBeGreaterThan(0);
    });

    test('ssh_monitor 包含 memory_total', async () => {
      const tauri = await import('../utils/tauri.js');
      const monitor = await tauri.invoke('ssh_monitor', { session_id: 'test' });
      expect(typeof monitor.memory_total).toBe('number');
    });

    test('tcp_ping 返回数字', async () => {
      const tauri = await import('../utils/tauri.js');
      const ping = await tauri.invoke('tcp_ping', { host: 'localhost' });
      expect(typeof ping).toBe('number');
    });

    test('tcp_ping 返回正数', async () => {
      const tauri = await import('../utils/tauri.js');
      const ping = await tauri.invoke('tcp_ping', { host: 'localhost' });
      expect(ping).toBeGreaterThan(0);
    });

    test('sftp_list_local 返回数组', async () => {
      const tauri = await import('../utils/tauri.js');
      const localFiles = await tauri.invoke('sftp_list_local', { path: '/tmp' });
      expect(Array.isArray(localFiles)).toBe(true);
    });

    test('sftp_list_local 至少返回 1 条记录', async () => {
      const tauri = await import('../utils/tauri.js');
      const localFiles = await tauri.invoke('sftp_list_local', { path: '/tmp' });
      expect(localFiles.length).toBeGreaterThan(0);
    });

    test('文件对象包含 name 字段', async () => {
      const tauri = await import('../utils/tauri.js');
      const localFiles = await tauri.invoke('sftp_list_local', { path: '/tmp' });
      expect(localFiles[0].name).toBeTruthy();
    });
  });

  // ─── 3. i18n.js ───
  describe('i18n.js — 导出验证', () => {
    test('t() 函数导出', async () => {
      const i18n = await import('../utils/i18n.js');
      expect(typeof i18n.t).toBe('function');
    });

    test('setLocale() 函数导出', async () => {
      const i18n = await import('../utils/i18n.js');
      expect(typeof i18n.setLocale).toBe('function');
    });

    test('getLocale() 函数导出', async () => {
      const i18n = await import('../utils/i18n.js');
      expect(typeof i18n.getLocale).toBe('function');
    });

    test('getLocales() 函数导出', async () => {
      const i18n = await import('../utils/i18n.js');
      expect(typeof i18n.getLocales).toBe('function');
    });

    test('getLocales() 返回数组', async () => {
      const i18n = await import('../utils/i18n.js');
      const locales = i18n.getLocales();
      expect(Array.isArray(locales)).toBe(true);
    });

    test('支持中文', async () => {
      const i18n = await import('../utils/i18n.js');
      const locales = i18n.getLocales();
      expect(locales.includes('zh')).toBe(true);
    });

    test('支持英文', async () => {
      const i18n = await import('../utils/i18n.js');
      const locales = i18n.getLocales();
      expect(locales.includes('en')).toBe(true);
    });

    test("t('app.name') 返回正确值", async () => {
      const i18n = await import('../utils/i18n.js');
      const appKey = i18n.t('app.name');
      expect(appKey).toBe('XTerminal Pro');
    });
  });

  // ─── 4. auto-reconnect.js ───
  describe('auto-reconnect.js — 类验证', () => {
    test('AutoReconnectManager 类可导入', async () => {
      const { AutoReconnectManager } = await import('../utils/auto-reconnect.js');
      expect(typeof AutoReconnectManager).toBe('function');
    });

    test('AutoReconnectManager 可实例化', async () => {
      const { AutoReconnectManager } = await import('../utils/auto-reconnect.js');
      const mgr = new AutoReconnectManager();
      expect(mgr instanceof AutoReconnectManager).toBe(true);
    });

    test('register 方法存在', async () => {
      const { AutoReconnectManager } = await import('../utils/auto-reconnect.js');
      const mgr = new AutoReconnectManager();
      expect(typeof mgr.register).toBe('function');
    });

    test('onDisconnect 方法存在', async () => {
      const { AutoReconnectManager } = await import('../utils/auto-reconnect.js');
      const mgr = new AutoReconnectManager();
      expect(typeof mgr.onDisconnect).toBe('function');
    });

    test('onConnect 方法存在', async () => {
      const { AutoReconnectManager } = await import('../utils/auto-reconnect.js');
      const mgr = new AutoReconnectManager();
      expect(typeof mgr.onConnect).toBe('function');
    });

    test('unregister 方法存在', async () => {
      const { AutoReconnectManager } = await import('../utils/auto-reconnect.js');
      const mgr = new AutoReconnectManager();
      expect(typeof mgr.unregister).toBe('function');
    });

    test('createReconnectManager 工厂函数存在', async () => {
      const { createReconnectManager } = await import('../utils/auto-reconnect.js');
      expect(typeof createReconnectManager).toBe('function');
    });

    test('工厂函数返回 AutoReconnectManager 实例', async () => {
      const { createReconnectManager, AutoReconnectManager } = await import('../utils/auto-reconnect.js');
      const mgr2 = createReconnectManager(5, 1000);
      expect(mgr2 instanceof AutoReconnectManager).toBe(true);
    });

    test('maxRetries 参数生效', async () => {
      const { createReconnectManager } = await import('../utils/auto-reconnect.js');
      const mgr2 = createReconnectManager(5, 1000);
      expect(mgr2.maxRetries).toBe(5);
    });

    test('retryDelay 参数生效', async () => {
      const { createReconnectManager } = await import('../utils/auto-reconnect.js');
      const mgr2 = createReconnectManager(5, 1000);
      expect(mgr2.retryDelay).toBe(1000);
    });
  });
});
