#[macro_export]
macro_rules! lock {
    ($mutex:expr) => {
        $mutex.lock().unwrap_or_else(|e| e.into_inner())
    };
}

// src-tauri/src/utils.rs — 共享工具函数与常量

/// 连接超时（TCP 握手）
pub const CONNECT_TIMEOUT_SECS: u64 = 10;
/// 读取超时（普通命令）
pub const READ_TIMEOUT_SECS: u64 = 30;
/// 读取超时（PTY shell，24小时）
pub const PTY_READ_TIMEOUT_SECS: u64 = 86400;
/// PTY 读取缓冲区大小
pub const PTY_BUF_SIZE: usize = 8192;
/// PTY 空闲轮询间隔（正常）
pub const PTY_POLL_FAST_MS: u64 = 5;
/// PTY 空闲轮询间隔（连续空闲后降频）
pub const PTY_POLL_SLOW_MS: u64 = 50;
/// PTY 连续空闲多少次后降频
pub const PTY_IDLE_THRESHOLD: u32 = 10;
/// SFTP 上传分块大小
pub const SFTP_CHUNK_SIZE: usize = 4000;
/// TCP ping 超时
pub const TCP_PING_TIMEOUT_SECS: u64 = 5;
/// 一天的秒数
pub const SECS_PER_DAY: u64 = 86400;
/// 一小时的秒数
pub const SECS_PER_HOUR: u64 = 3600;

/// 获取 Unix 时间戳（秒）
pub fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
