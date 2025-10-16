use crate::models::StreamFrame;
use crate::state::SessionHandle;

use parking_lot::Mutex;
use std::sync::Arc;
use tracing::debug;

use nix::fcntl::{fcntl, FcntlArg, OFlag};
use nix::pty::{forkpty, ForkptyResult, Winsize};
use nix::unistd::{dup, execvp, read as nix_read, write as nix_write};
use std::ffi::CString;
use std::os::fd::{AsRawFd, FromRawFd, IntoRawFd, BorrowedFd};
use tokio::io::unix::AsyncFd;

pub fn spawn_pty_shell(shell: &str, cols: u16, rows: u16) -> anyhow::Result<SessionHandle> {
    // Prepare C strings BEFORE fork (avoid allocations in child)
    let prog = CString::new(shell).expect("shell CString");
    let arg0 = prog.clone();
    let li   = CString::new("-li").expect("CString -li");
    let args = [arg0, li];

    let ws = Winsize { ws_row: rows, ws_col: cols, ws_xpixel: 0, ws_ypixel: 0 };

    // SAFETY: forkpty crosses the FFI boundary and forks the process.
    // In the child branch below we only call execvp then _exit.
    let fp = unsafe { forkpty(Some(&ws), None)? };

    match fp {
        ForkptyResult::Child => {
            // CHILD: only async-signal-safe ops; do not allocate or log.
            unsafe {
                // Attempt to exec; on any error, immediately _exit(127).
                let _ = execvp(&prog, &args);
                libc::_exit(127); // never returns
            }
        }
        ForkptyResult::Parent { child, master } => {
            // PARENT: mark master PTY nonblocking so AsyncFd readiness works
            let mfd = master.as_raw_fd();
            let cur = OFlag::from_bits_truncate(fcntl(mfd, FcntlArg::F_GETFL)?);
            fcntl(mfd, FcntlArg::F_SETFL(cur | OFlag::O_NONBLOCK))?;

            // Dedicated handles: dup once for reader; original becomes writer
            let rd_raw  = dup(mfd)?;
            let rd_file = unsafe { std::fs::File::from_raw_fd(rd_raw) };
            let wr_file = unsafe { std::fs::File::from_raw_fd(master.into_raw_fd()) };

            let reader = Arc::new(AsyncFd::new(rd_file)?);
            let writer = Arc::new(AsyncFd::new(wr_file)?);

            let (tx, _rx) = tokio::sync::broadcast::channel::<StreamFrame>(1024);
            let latest_seq = Arc::new(Mutex::new(0u64));
            let exit_code  = Arc::new(Mutex::new(None));

            // Reader task
            let txr = tx.clone();
            let seqr = latest_seq.clone();
            let reader_clone = reader.clone();
            tokio::spawn(async move {
                let mut buf = [0u8; 4096];
                loop {
                    match reader_clone.readable().await {
                        Ok(mut guard) => {
                            let res = guard.try_io(|inner| {
                                let fd = inner.get_ref().as_raw_fd();
                                match nix_read(fd, &mut buf) {
                                    Ok(n) => Ok(n),
                                    Err(e) => Err(std::io::Error::from_raw_os_error(e as i32)),
                                }
                            });
                            match res {
                                Ok(Ok(0)) => {
                                    debug!("PTY EOF");
                                    break;
                                }
                                Ok(Ok(n)) => {
                                    debug!("PTY read {} bytes", n);
                                    let s = String::from_utf8_lossy(&buf[..n]).to_string();
                                    let mut seq = seqr.lock();
                                    *seq += 1;
                                    let _ = txr.send(StreamFrame { t: "stdout".into(), seq: *seq, d: s });
                                }
                                Ok(Err(e)) => {
                                    if e.kind() == std::io::ErrorKind::WouldBlock {
                                        // spurious readability; try again
                                        continue;
                                    } else {
                                        debug!("PTY read error: {}", e);
                                        break;
                                    }
                                }
                                Err(_would_block) => continue, // readiness false positive; try again
                            }

                        }
                        Err(_) => break,
                    }
                }
            });

            return Ok(SessionHandle {
                latest_seq,
                tx,
                exit_code,
                reader,
                writer,
                pid: child.as_raw(),
            });
        }
    }
}


pub async fn write_pty(h: &SessionHandle, data: &str) -> anyhow::Result<()> {
    let writer = h.writer.clone();
    let bytes = data.as_bytes();
    let mut off = 0usize;
    while off < bytes.len() {
        let mut guard = writer.writable().await?;
        let res = guard.try_io(|inner| {
            let raw = inner.get_ref().as_raw_fd();
            let fd  = unsafe { BorrowedFd::borrow_raw(raw) };
            match nix_write(fd, &bytes[off..]) {
                Ok(n) => Ok(n),
                Err(e) => Err(std::io::Error::from_raw_os_error(e as i32)),
            }
        });
        match res {
            Ok(Ok(0)) => {
                debug!("PTY write returned 0 (done)");
                break;
            }
            Ok(Ok(n)) => {
                off += n;
                debug!("PTY wrote {} bytes (total {})", n, off);
            }
            Ok(Err(e)) => {
                if e.kind() == std::io::ErrorKind::WouldBlock {
                    // try again after next readiness
                    continue;
                } else {
                    debug!("PTY write error: {}", e);
                    return Err(e.into());
                }
            }
            Err(_would_block) => continue,
        }

    }
    Ok(())
}


pub async fn resize_pty(h:&SessionHandle,cols:u16,rows:u16)->anyhow::Result<()>{
    let fd=h.reader.as_raw_fd();
    let ws=libc::winsize{ws_row:rows,ws_col:cols,ws_xpixel:0,ws_ypixel:0};
    let rc=unsafe{libc::ioctl(fd, libc::TIOCSWINSZ, &ws)};
    if rc!=0{ return Err(anyhow::anyhow!("ioctl TIOCSWINSZ failed: {}", std::io::Error::last_os_error()));}
    Ok(())
}

pub async fn close_pty(h:&SessionHandle)->anyhow::Result<()>{
    let _=write_pty(h, "\x04").await;
    Ok(())
}
