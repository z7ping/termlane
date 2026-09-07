# SSH Agent Forwarding 技术调研

> 调研日期：2026-06-20
> 状态：方案已确定，待排期实现

## 什么是 SSH Agent Forwarding

SSH Agent Forwarding 让你从 A 机器跳到 B 机器时，B 机器能用 A 机器上的密钥去连 C 机器，而不用把密钥复制到 B 上。

```
你的电脑(A) → 跳板机(B) → 生产服务器(C)
```

- 没开 agent forwarding：A 有密钥能连 C，但通过 B 跳转时 B 上没有密钥，连不了 C
- 开了 agent forwarding：A 的密钥"代理"过去，B 能临时用 A 的密钥连 C，密钥本身不离开 A

**典型场景**：运维人员通过跳板机管理生产服务器时几乎必用。FinalShell、Termius、Electerm 都支持。

## 调研结论

**ssh2-rs（libssh2）支持 agent forwarding**，但需要通过 FFI 桥接。

### libssh2 层

| API | 作用 |
|-----|------|
| `libssh2_channel_request_auth_agent(channel)` | 向服务器请求开启 agent forwarding |
| `LIBSSH2_CALLBACK_AUTHAGENT` | 处理远程服务器回连本地 agent 的回调 |
| `libssh2_session_callback_set2(session, type, callback)` | 设置回调函数 |

### ssh2 crate 层（v0.9.5）

| API | 状态 |
|-----|------|
| `channel.request_auth_agent_forwarding()` | ✅ 已暴露，可直接调用 |
| `session.set_auth_agent_callback()` | ❌ 未暴露，需要 FFI 桥接 |

### 核心问题

ssh2 crate 暴露了 `request_auth_agent_forwarding()` 方法，但没有暴露设置 `LIBSSH2_CALLBACK_AUTHAGENT` 回调的方法。这个回调是必须的——当远程服务器需要使用你的 agent 时，它会通过 SSH 隧道连回来，你需要处理这个连接。

## 实现方案

### 方案 A：FFI 直接调 libssh2（推荐）

利用 ssh2 crate 基于 libssh2-sys 的事实，直接用 FFI 调 libssh2 的 C 函数：

```rust
use libssh2_sys as raw;

// 1. 在 session 上设置 auth agent 回调
unsafe {
    raw::libssh2_session_callback_set2(
        session.raw(),
        raw::LIBSSH2_CALLBACK_AUTHAGENT,
        Some(auth_agent_callback),
    );
}

// 2. 在 channel 上请求 agent forwarding
channel.request_auth_agent_forwarding()?;

// 3. 启动 shell 或 exec
channel.shell()?;

// 4. 实现回调：处理远程服务器的 agent 请求
extern "C" fn auth_agent_callback(
    _session: *mut raw::LIBSSH2_SESSION,
    _channel: *mut raw::LIBSSH2_CHANNEL,
    _abstract: *mut *mut c_void,
) -> i32 {
    // 连接本地 ssh-agent（通过 SSH_AUTH_SOCK 环境变量）
    // 解析 SSH agent 协议请求
    // 通过 SSH 隧道转发到本地 agent
    // 返回响应
}
```

**优点**：libssh2 已提供基础设施，只需桥接回调
**工作量**：1-2 天，主要是写回调函数里的 agent 协议转发逻辑
**风险**：依赖 libssh2-sys 的 FFI 绑定，版本升级时需验证兼容性

### 方案 B：手动实现（备选）

不用 libssh2 的回调，完全自己实现：

1. 监听本地 Unix socket（`/tmp/ssh-agent-xxx` 或 Windows named pipe）
2. 远程服务器连过来时，解析 SSH agent 协议（ASSH_AGENTC_REQUEST / SSH_AGENT_SUCCESS 等）
3. 把请求通过 SSH 隧道转发到本地 ssh-agent
4. 把响应转发回去

**优点**：不依赖 libssh2 的回调机制，完全可控
**工作量**：3-5 天，需要自己实现 SSH agent 协议解析
**风险**：协议实现容易出错，需要处理各种边界情况

## 推荐方案

**方案 A**。理由：
1. libssh2 已经提供了 `libssh2_channel_request_auth_agent()` 和 `LIBSSH2_CALLBACK_AUTHAGENT`，不需要重复造轮子
2. ssh2 crate 的 `request_auth_agent_forwarding()` 已经可用，只需补上回调设置
3. 工作量小（1-2 天 vs 3-5 天）

## 注意事项

1. **SSH 服务器配置**：远程服务器的 `sshd_config` 需要设置 `AllowAgentForwarding yes`（默认是 yes）
2. **安全风险**：agent forwarding 意味着远程服务器可以使用你的密钥。如果远程服务器被入侵，攻击者可以利用你的密钥连接其他服务器。仅在信任的跳板机上使用
3. **Windows 兼容性**：Windows 上没有 Unix socket，需要用 named pipe 或 Windows OpenSSH agent 服务。这是额外的工作量
4. **回调线程安全**：`LIBSSH2_CALLBACK_AUTHAGENT` 回调在 libssh2 的 I/O 线程中调用，需要注意线程安全

## 竞品参考

| 产品 | SSH 实现 | Agent Forwarding |
|------|---------|-----------------|
| Electerm | node-ssh（ssh2.js） | ✅ 支持 |
| Tabby | node-ssh（ssh2.js） | ✅ 支持 |
| Termius | 自研 | ✅ 支持 |
| FinalShell | JavaFX + 自研 | ✅ 支持 |
| **Termlane** | **ssh2-rs（libssh2）** | **❌ 待实现** |

## 排期建议

- 优先级：中（大部分 SSH 连接是直连，不需要 agent forwarding；只有运维跳板机场景才需要）
- 建议在核心功能（SSH 连接、SFTP、批量命令）稳定后再实现
- Windows 兼容性可以先不做，Linux/macOS 先支持
