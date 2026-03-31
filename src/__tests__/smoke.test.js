// src/__tests__/smoke.test.js — 冒烟测试：验证关键模块导出和 mock 功能
// 运行: node src/__tests__/smoke.test.js

let passed = 0;
let failed = 0;

function assert(condition, msg) {
  if (condition) {
    console.log(`  ✅ ${msg}`);
    passed++;
  } else {
    console.error(`  ❌ ${msg}`);
    failed++;
  }
}

function assertEqual(actual, expected, msg) {
  assert(actual === expected, `${msg} (期望: ${expected}, 实际: ${actual})`);
}

function assertType(value, type, msg) {
  assert(typeof value === type, `${msg} (类型: ${typeof value}, 期望: ${type})`);
}

// ─── 1. tauri.js ───
console.log('\n📦 tauri.js — 导出验证');
const tauri = await import('../utils/tauri.js');
assertType(tauri.invoke, 'function', 'invoke 导出为函数');
assertType(tauri.listen, 'function', 'listen 导出为函数');
assertType(tauri.isTauri, 'boolean', 'isTauri 导出为布尔值');
assertEqual(tauri.isTauri, false, 'Node.js 环境下 isTauri 应为 false');

// ─── 2. Mock 命令 ───
console.log('\n🎭 Mock 命令 — 数据验证');

const sessionId = await tauri.invoke('ssh_connect', { host: 'localhost' });
assertType(sessionId, 'string', 'ssh_connect 返回字符串');
assert(sessionId.startsWith('mock_session_'), 'ssh_connect 返回 mock_session_ 前缀');

const connections = await tauri.invoke('load_connections');
assert(Array.isArray(connections), 'load_connections 返回数组');
assert(connections.length > 0, 'load_connections 至少包含 1 条连接');
assert(connections[0].id, '连接对象包含 id 字段');

const monitor = await tauri.invoke('ssh_monitor', { session_id: 'test' });
assertType(monitor, 'object', 'ssh_monitor 返回对象');
assertType(monitor.cpu_usage, 'number', 'ssh_monitor 包含 cpu_usage (number)');
assert(monitor.cpu_usage > 0, 'cpu_usage 为正数');
assertType(monitor.memory_total, 'number', 'ssh_monitor 包含 memory_total');

const ping = await tauri.invoke('tcp_ping', { host: 'localhost' });
assertType(ping, 'number', 'tcp_ping 返回数字');
assert(ping > 0, 'tcp_ping 返回正数');

const localFiles = await tauri.invoke('sftp_list_local', { path: '/tmp' });
assert(Array.isArray(localFiles), 'sftp_list_local 返回数组');
assert(localFiles.length > 0, 'sftp_list_local 至少返回 1 条记录');
assert(localFiles[0].name, '文件对象包含 name 字段');

// ─── 3. i18n.js ───
console.log('\n🌐 i18n.js — 导出验证');
// i18n.js 使用 localStorage，Node.js 中需要 mock
if (typeof globalThis.localStorage === 'undefined') {
  const store = {};
  globalThis.localStorage = {
    getItem: (k) => store[k] ?? null,
    setItem: (k, v) => { store[k] = String(v); },
    removeItem: (k) => { delete store[k]; },
  };
}
const i18n = await import('../utils/i18n.js');
assertType(i18n.t, 'function', 't() 函数导出');
assertType(i18n.setLocale, 'function', 'setLocale() 函数导出');
assertType(i18n.getLocale, 'function', 'getLocale() 函数导出');
assertType(i18n.getLocales, 'function', 'getLocales() 函数导出');

const locales = i18n.getLocales();
assert(Array.isArray(locales), 'getLocales() 返回数组');
assert(locales.includes('zh'), '支持中文 (zh)');
assert(locales.includes('en'), '支持英文 (en)');

const appKey = i18n.t('app.name');
assert(appKey === 'XTerminal Pro', `t('app.name') 返回正确值: ${appKey}`);

// ─── 4. auto-reconnect.js ───
console.log('\n🔄 auto-reconnect.js — 类验证');
const { AutoReconnectManager, createReconnectManager } = await import('../utils/auto-reconnect.js');
assertType(AutoReconnectManager, 'function', 'AutoReconnectManager 类可导入');

const mgr = new AutoReconnectManager();
assert(mgr instanceof AutoReconnectManager, 'AutoReconnectManager 可实例化');
assertType(mgr.register, 'function', 'register 方法存在');
assertType(mgr.onDisconnect, 'function', 'onDisconnect 方法存在');
assertType(mgr.onConnect, 'function', 'onConnect 方法存在');
assertType(mgr.unregister, 'function', 'unregister 方法存在');

assertType(createReconnectManager, 'function', 'createReconnectManager 工厂函数存在');
const mgr2 = createReconnectManager(5, 1000);
assert(mgr2 instanceof AutoReconnectManager, '工厂函数返回 AutoReconnectManager 实例');
assertEqual(mgr2.maxRetries, 5, 'maxRetries 参数生效');
assertEqual(mgr2.retryDelay, 1000, 'retryDelay 参数生效');

// ─── 结果汇总 ───
console.log('\n' + '─'.repeat(40));
console.log(`📊 结果: ${passed} 通过, ${failed} 失败, 共 ${passed + failed} 项`);
if (failed > 0) {
  console.error('❌ 存在失败测试');
  process.exit(1);
} else {
  console.log('✅ 全部通过');
}
