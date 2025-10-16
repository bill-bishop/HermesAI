use crate::models::StreamFrame;
use crate::state::SessionHandle;
use crate::config::resolve_profile;

use parking_lot::Mutex;
use std::collections::VecDeque;
use std::sync::Arc;
use tracing::debug;

use nix::fcntl::{fcntl, FcntlArg, OFlag};
use nix::pty::{forkpty, ForkptyResult, Winsize};
use nix::unistd::{dup, execvp, read as nix_read, write as nix_write};
use std::ffi::CString;
use std::os::fd::{AsRawFd, FromRawFd, IntoRawFd, BorrowedFd};
use tokio::io::unix::AsyncFd;

pub fn spawn_pty_shell(profile: Option<String>, cols: u16, rows: u16) -> anyhow::Result<SessionHandle> {
    let prof = resolve_profile(profile.as_deref());
    let prog = CString::new(prof.program.clone()).expect("prog CString");
    let mut argv: Vec<CString> = Vec::with_capacity(1 + prof.args.len());
    argv.push(prog.clone());
    for a in &prof.args {
        argv.push(CString::new(a.as_str()).expect("arg CString"));
    }

    let ws = Winsize { ws_row: rows, ws_col: cols, ws_xpixel: 0, ws_ypixel: 0 };

    let fp = unsafe { forkpty(Some(&ws), None)? };

    match fp {
        ForkptyResult::Child => {
            unsafe {
                let _ = execvp(&prog, &argv);
                libc::_exit(127);
            }
        }
        ForkptyResult::Parent { child, master } => {
            let mfd = master.as_raw_fd();
            let cur = OFlag::from_bits_truncate(fcntl(mfd, FcntlArg::F_GETFL)?);
            fcntl(mfd, FcntlArg::F_SETFL(cur | OFlag::O_NONBLOCK))?;

            let rd_raw = dup(mfd)?;
            let rd_file = unsafe { std::fs::File::from_raw_fd(rd_raw) };
            let wr_file = unsafe { std::fs::File::from_raw_fd(master.into_raw_fd()) };

            let reader = Arc::new(AsyncFd::new(rd_file)?);
            let writer = Arc::new(AsyncFd::new(wr_file)?);

            let (tx, _rx) = tokio::sync::broadcast::channel::<StreamFrame>(1024);
            let latest_seq = Arc::new(Mutex::new(0u64));
            let exit_code  = Arc::new(Mutex::new(None));
            let backlog    = Arc::new(Mutex::new(VecDeque::with_capacity(1024)));

            let txr = tx.clone();
            let seqr = latest_seq.clone();
            let backlog_c = backlog.clone();
            let reader_c = reader.clone();

            tokio::spawn(async move {
                let mut buf = [0u8; 4096];
                loop {
                    match reader_c.readable().await {
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
                                    let mut s = seqr.lock(); *s += 1;
                                    let ev = StreamFrame { t: "event".into(), seq: *s, d: "exit:None".into() };
                                    let mut b = backlog_c.lock();
                                    if b.len() == b.capacity() { b.pop_front(); }
                                    b.push_back(ev.clone());
                                    let _ = txr.send(ev);
                                    break;
                                }
                                Ok(Ok(n)) => {
                                    debug!("PTY read {} bytes", n);
                                    let sdata = String::from_utf8_lossy(&buf[..n]).to_string();
                                    let mut s = seqr.lock(); *s += 1;
                                    let frame = StreamFrame { t: "stdout".into(), seq: *s, d: sdata };
                                    let mut b = backlog_c.lock();
                                    if b.len() == b.capacity() { b.pop_front(); }
                                    b.push_back(frame.clone());
                                    let _ = txr.send(frame);
                                }
                                Ok(Err(e)) => {
                                    if e.kind() == std::io::ErrorKind::WouldBlock { continue; }
                                    debug!("PTY read error: {}", e);
                                    let mut s = seqr.lock(); *s += 1;
                                    let ev = StreamFrame { t: "event".into(), seq: *s, d: "exit:None".into() };
                                    let mut b = backlog_c.lock();
                                    if b.len() == b.capacity() { b.pop_front(); }
                                    b.push_back(ev.clone());
                                    let _ = txr.send(ev);
                                    break;
                                }
                                Err(_would_block) => continue,
                            }
                        }
                        Err(_e) => break,
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
                backlog,
            });
        }
    }
}

pub async fn write_pty(h: &SessionHandle, data: &str) -> anyhow::Result<()> {
    use std::os::fd::AsFd;
    let writer = h.writer.clone();
    let bytes = data.as_bytes();
    let mut off = 0usize;
    while off < bytes.len() {
        let mut guard = writer.writable().await?;
        let res = guard.try_io(|inner| {
            let raw = inner.get_ref().as_raw_fd();
            let fd = unsafe { BorrowedFd::borrow_raw(raw) };
            match nix_write(fd, &bytes[off..]) {
                Ok(n) => Ok(n),
                Err(e) => Err(std::io::Error::from_raw_os_error(e as i32)),
            }
        });
        match res {
            Ok(Ok(0)) => break,
            Ok(Ok(n)) => { off += n; debug!("PTY wrote {} bytes (total {})", n, off); }
            Ok(Err(e)) => {
                if e.kind() == std::io::ErrorKind::WouldBlock { continue; }
                return Err(e.into());
            }
            Err(_would_block) => continue,
        }
    }
    Ok(())
}

pub async fn resize_pty(h:&SessionHandle, cols:u16, rows:u16) -> anyhow::Result<()> {
    let fd = h.reader.as_raw_fd();
    let ws = libc::winsize { ws_row: rows, ws_col: cols, ws_xpixel: 0, ws_ypixel: 0 };
    let rc = unsafe { libc::ioctl(fd, libc::TIOCSWINSZ, &ws) };
    if rc != 0 {
        return Err(anyhow::anyhow!("ioctl TIOCSWINSZ failed: {}", std::io::Error::last_os_error()));
    }
    Ok(())
}

pub async fn close_pty(h:&SessionHandle) -> anyhow::Result<()> {
    let _ = write_pty(h, "\x04").await;
    Ok(())
}
