use crate::models::StreamFrame;
use crate::state::SessionHandle;
use nix::pty::{openpty, Winsize};
use nix::sys::signal::{killpg, Signal};
use nix::unistd::{fork, ForkResult, setsid, dup2, close, execvp};
use nix::libc::{TIOCSCTTY, ioctl};
use nix::unistd::{Pid};
use std::ffi::CString;
use parking_lot::Mutex;
use std::os::fd::{FromRawFd, AsRawFd};
use std::sync::Arc;
use tokio::io::unix::AsyncFd;
use tokio::io::AsyncWriteExt;

pub fn spawn_pty_shell(shell: &str, cols: u16, rows: u16, cwd: Option<String>, env: Option<std::collections::HashMap<String,String>>) -> anyhow::Result<SessionHandle> {
    // open PTY
    let mut ws = Winsize { ws_row: rows, ws_col: cols, ws_xpixel: 0, ws_ypixel: 0 };
    let pty = openpty(Some(&ws), None)?;
    // Fork
    match unsafe { fork() }? {
        ForkResult::Child => {
            // Child: new session, make slave controlling tty
            setsid()?;
            unsafe { ioctl(pty.slave, TIOCSCTTY as _, 0) };
            // Duplicate stdio
            dup2(pty.slave, 0)?;
            dup2(pty.slave, 1)?;
            dup2(pty.slave, 2)?;
            // Close fds
            let _ = close(pty.master);
            let _ = close(pty.slave);
            if let Some(dir) = cwd {
                let _ = std::env::set_current_dir(dir);
            }
            if let Some(map) = env {
                for (k,v) in map {
                    std::env::set_var(k, v);
                }
            }
            let sh = CString::new(shell).unwrap();
            let args = &[sh.clone()];
            execvp(&sh, args).expect("execvp failed");
        }
        ForkResult::Parent { child } => {
            // Parent: async read from master
            // Close slave in parent
            let _ = close(pty.slave);
            let master_file = unsafe { std::fs::File::from_raw_fd(pty.master) };
            let async_master = AsyncFd::new(master_file)?;
            let (tx, _rx) = tokio::sync::broadcast::channel::<StreamFrame>(1024);
            let latest_seq = Arc::new(Mutex::new(0u64));
            let exit_code = Arc::new(Mutex::new(None));

            // Reader task
            let txr = tx.clone();
            let seqr = latest_seq.clone();
            tokio::spawn(async move {
                let mut buf = [0u8; 4096];
                loop {
                    match async_master.readable().await {
                        Ok(mut guard) => {
                            match guard.get_inner().read(&mut buf) {
                                Ok(0) => break,
                                Ok(n) => {
                                    let s = String::from_utf8_lossy(&buf[..n]).to_string();
                                    let mut seq = seqr.lock();
                                    *seq += 1;
                                    let _ = txr.send(StreamFrame{ t:"stdout".into(), seq:*seq, d:s });
                                    guard.clear_ready();
                                }
                                Err(_) => { break; }
                            }
                        }
                        Err(_) => break,
                    }
                }
            });

            let writer = Arc::new(Mutex::new(Some(async_master)));
            let pid = child.as_raw();

            Ok(SessionHandle {
                latest_seq,
                tx,
                exit_code,
                writer,
                pid,
            })
        }
    }
}

pub async fn write_pty(h: &SessionHandle, data: &str) -> anyhow::Result<()> {
    if let Some(w) = &mut *h.writer.lock() {
        let mut ready = w.writable().await?;
        ready.get_inner_mut().write_all(data.as_bytes())?;
        ready.clear_ready();
    }
    Ok(())
}

pub async fn resize_pty(_h: &SessionHandle, _cols: u16, _rows: u16) -> anyhow::Result<()> {
    // Left as exercise (platform-specific ioctl TIOCSWINSZ)
    Ok(())
}

pub async fn close_pty(h: &SessionHandle) -> anyhow::Result<()> {
    // send SIGTERM to process group
    let _ = killpg(Pid::from_raw(h.pid), Signal::SIGTERM);
    Ok(())
}
