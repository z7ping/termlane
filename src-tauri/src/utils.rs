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
/// 远程文件上传/写入时的 Base64 shell 分块大小
pub const SFTP_CHUNK_SIZE: usize = 4000;
/// TCP ping 超时
pub const TCP_PING_TIMEOUT_SECS: u64 = 5;
/// 一天的秒数
pub const SECS_PER_DAY: u64 = 86400;
/// 一小时的秒数
pub const SECS_PER_HOUR: u64 = 3600;

const BASE64_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// RFC 4648 Base64 编码。
///
/// 当前用于远程文件 shell 传输，以及把 SSH SHA-256 原始摘要转换为
/// 与 OpenSSH 可直接核对的可打印指纹格式。
pub fn base64_encode(input: &[u8]) -> String {
    let mut result = String::with_capacity((input.len() + 2) / 3 * 4);
    for chunk in input.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        result.push(BASE64_CHARS[((triple >> 18) & 0x3f) as usize] as char);
        result.push(BASE64_CHARS[((triple >> 12) & 0x3f) as usize] as char);
        if chunk.len() > 1 {
            result.push(BASE64_CHARS[((triple >> 6) & 0x3f) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(BASE64_CHARS[(triple & 0x3f) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}

pub fn base64_decode(input: &str) -> Result<Vec<u8>, String> {
    let mut table = [255u8; 256];
    for (index, &character) in BASE64_CHARS.iter().enumerate() {
        table[character as usize] = index as u8;
    }
    table[b'=' as usize] = 0;

    let clean: Vec<u8> = input
        .bytes()
        .filter(|byte| !byte.is_ascii_whitespace())
        .collect();
    if clean.len() % 4 != 0 {
        return Err("无效的 base64 编码".to_string());
    }

    let mut result = Vec::with_capacity(clean.len() / 4 * 3);
    for chunk in clean.chunks(4) {
        let a = table[chunk[0] as usize];
        let b = table[chunk[1] as usize];
        let c = table[chunk[2] as usize];
        let d = table[chunk[3] as usize];
        if a == 255 || b == 255 || c == 255 || d == 255 {
            return Err("无效的 base64 字符".to_string());
        }
        result.push((a << 2) | (b >> 4));
        if chunk[2] != b'=' {
            result.push(((b & 0x0f) << 4) | (c >> 2));
        }
        if chunk[3] != b'=' {
            result.push(((c & 0x03) << 6) | d);
        }
    }
    Ok(result)
}

/// 获取 Unix 时间戳（秒）
pub fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_matches_rfc4648_vectors() {
        for (raw, encoded) in [
            (b"".as_slice(), ""),
            (b"f".as_slice(), "Zg=="),
            (b"fo".as_slice(), "Zm8="),
            (b"foo".as_slice(), "Zm9v"),
            (b"foobar".as_slice(), "Zm9vYmFy"),
        ] {
            assert_eq!(base64_encode(raw), encoded);
            assert_eq!(base64_decode(encoded).expect("valid base64"), raw);
        }
    }
}
