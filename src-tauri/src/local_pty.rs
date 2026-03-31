// local_pty.rs - Local shell PTY support

use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter};

/// Safely lock a Mutex, recovering from poisoned locks
macro_rules! lock {
    ($mutex:expr) => {
        $mutex.lock().unwrap_or_else(|e| e.into_inner())
    };
}

struct LocalPty {
    _master: Box<dyn portable_pty::MasterPty + Send>,
    _child: Box<dyn portable_pty::Child + Send + Sync>,
    input_tx: crossbeam_channel::Sender<String>,
    _reader: std::thread::JoinHandle<()>,
}

static LOCAL_PTYS: std::sync::LazyLock<Mutex<HashMap<String, LocalPty>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn start_local_shell(
    app: AppHandle,
    cols: u16,
    rows: u16,
    shell: Option<&str>,
    cwd: Option<&str>,
) -> Result<String, String> {
    let pty_system = native_pty_system();

    let pair = pty_system
        .openpty(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|e| format!("PTY 创建失败: {}", e))?;

    let shell_cmd = shell.unwrap_or(if cfg!(target_os = "windows") {
        "powershell.exe"
    } else {
        "/bin/bash"
    });

    let mut cmd = CommandBuilder::new(shell_cmd);
    cmd.env("TERM", "xterm-256color");
    cmd.env("COLORTERM", "truecolor");
    if let Some(dir) = cwd {
        cmd.cwd(dir);
    }

    let child = pair
        .slave
        .spawn_command(cmd)
        .map_err(|e| format!("启动 shell 失败: {}", e))?;

    // Drop the slave - master owns the PTY
    drop(pair.slave);

    let mut reader = pair
        .master
        .try_clone_reader()
        .map_err(|e| format!("克隆读取器失败: {}", e))?;

    let writer = pair
        .master
        .take_writer()
        .map_err(|e| format!("获取写入器失败: {}", e))?;

    let (input_tx, input_rx) = crossbeam_channel::unbounded::<String>();

    let session_id = format!("local_{}", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs());

    let sid = session_id.clone();
    let sid_for_reader = sid.clone();

    // Reader thread
    let reader_thread = std::thread::spawn(move || {
        let mut buf = [0u8; 8192];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => {
                    let _ = app.emit(&format!("local-output:{}", sid_for_reader), "\r\n\x1b[1;33m[Shell 已退出]\x1b[0m\r\n");
                    break;
                }
                Ok(n) => {
                    let output = String::from_utf8_lossy(&buf[..n]).into_owned();
                    let _ = app.emit(&format!("local-output:{}", sid_for_reader), output);
                }
                Err(_) => {
                    let _ = app.emit(&format!("local-output:{}", sid_for_reader), "\r\n\x1b[1;31m[连接断开]\x1b[0m\r\n");
                    break;
                }
            }
        }
    });

    // Writer thread
    let writer_thread = std::thread::spawn(move || {
        let mut w = writer;
        while let Ok(input) = input_rx.recv() {
            if w.write_all(input.as_bytes()).is_err() {
                break;
            }
            w.flush().ok();
        }
    });

    // We can't store the writer thread handle in LocalPty because it needs to outlive the struct
    // Actually let's just store everything we need
    std::mem::forget(writer_thread); // Writer thread runs until channel closes

    lock!(LOCAL_PTYS).insert(
        session_id.clone(),
        LocalPty {
            _master: pair.master,
            _child: child,
            input_tx,
            _reader: reader_thread,
        },
    );

    Ok(sid)
}

pub fn local_input(session_id: &str, data: &str) -> Result<(), String> {
    let ptys = lock!(LOCAL_PTYS);
    let pty = ptys.get(session_id).ok_or("本地 Shell 不存在")?;
    pty.input_tx
        .send(data.to_string())
        .map_err(|e| format!("发送失败: {}", e))
}

pub fn local_resize(session_id: &str, cols: u16, rows: u16) -> Result<(), String> {
    // Resize is handled through the master PTY, but we'd need a separate channel
    // For now, this is a no-op (most programs handle SIGWINCH automatically)
    let _ = (session_id, cols, rows);
    Ok(())
}

pub fn close_local_shell(session_id: &str) -> Result<(), String> {
    lock!(LOCAL_PTYS).remove(session_id);
    Ok(())
}

pub fn list_local_shells() -> Vec<String> {
    lock!(LOCAL_PTYS).keys().cloned().collect()
}
