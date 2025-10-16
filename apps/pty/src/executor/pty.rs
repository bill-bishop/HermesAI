use crate::models::StreamFrame;
use crate::state::SessionHandle;
use nix::pty::{openpty, Winsize};
use nix::unistd::{fork, ForkResult, setsid, dup2, close, execvp, dup, read as nix_read, write as nix_write};
use std::ffi::CString;
use std::os::fd::{AsRawFd, IntoRawFd, FromRawFd, BorrowedFd};
use parking_lot::Mutex;
use std::sync::Arc;
use tokio::io::unix::AsyncFd;

pub fn spawn_pty_shell(shell:&str,cols:u16,rows:u16)->anyhow::Result<SessionHandle>{
    let ws=Winsize{ws_row:rows,ws_col:cols,ws_xpixel:0,ws_ypixel:0};
    let pty=openpty(Some(&ws),None)?;
    match unsafe{fork()}?{
        ForkResult::Child=>{
            setsid()?;
            unsafe{libc::ioctl(pty.slave.as_raw_fd(),libc::TIOCSCTTY,0)};
            dup2(pty.slave.as_raw_fd(),0)?;dup2(pty.slave.as_raw_fd(),1)?;dup2(pty.slave.as_raw_fd(),2)?;
            let _=close(pty.master.as_raw_fd());let _=close(pty.slave.as_raw_fd());
            // minimal env for real shells
            std::env::set_var("TERM", "xterm-256color");
            if std::env::var_os("PATH").is_none() {
                std::env::set_var("PATH", "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin");
            }
            // prefer interactive login shell args: "-li"
            // execvp(argv[0], argv) — pass program + args
            let prog = CString::new(shell).unwrap();
            let arg0 = prog.clone();
            let li   = CString::new("-li").unwrap();
            let args = &[arg0, li];
            execvp(&prog, args).expect("exec failed");
            unsafe{libc::_exit(127);}
        }
        ForkResult::Parent{child}=>{
            let _=close(pty.slave.as_raw_fd());
            let master_fd = pty.master.as_raw_fd();
            let rd = unsafe{ std::fs::File::from_raw_fd(dup(master_fd)?) };
            let wr = unsafe{ std::fs::File::from_raw_fd(pty.master.into_raw_fd()) };
            let reader = Arc::new(AsyncFd::new(rd)?);
            let writer = Arc::new(AsyncFd::new(wr)?);
            let (tx,_)=tokio::sync::broadcast::channel::<StreamFrame>(1024);
            let latest_seq=Arc::new(Mutex::new(0u64));
            let exit_code=Arc::new(Mutex::new(None));

            let txr=tx.clone();let seqr=latest_seq.clone();let reader_clone=reader.clone();
            tokio::spawn(async move{
                let mut buf=[0u8;4096];
                loop{
                    match reader_clone.readable().await{
                        Ok(mut guard)=>{
                            let res = guard.try_io(|inner|{
                                // nix_read expects RawFd (i32), not AsFd
                                let fd = inner.get_ref().as_raw_fd();
                                match nix_read(fd, &mut buf){
                                    Ok(n) => Ok(n),
                                    Err(e) => Err(std::io::Error::from_raw_os_error(e as i32)),
                                }
                            });
                            match res{
                                Ok(Ok(0))=>break,
                                Ok(Ok(n)) => {
                                    let s = String::from_utf8_lossy(&buf[..n]).to_string();
                                    let mut seq = seqr.lock();
                                    *seq += 1;
                                    let _ = txr.send(StreamFrame { t:"stdout".into(), seq:*seq, d:s });
                                }
                                Ok(Err(_))=>break,
                                Err(_)=>continue,
                            }
                        }
                        Err(_)=>break,
                    }
                }
            });

            Ok(SessionHandle{latest_seq,tx,exit_code,reader,writer,pid:child.as_raw()})
        }
    }
}

pub async fn write_pty(h:&SessionHandle,data:&str)->anyhow::Result<()>{
    let writer=h.writer.clone();
    let bytes=data.as_bytes();
    let mut off=0usize;
    while off<bytes.len(){
        let mut guard = writer.writable().await?;
        let res = guard.try_io(|inner|{
            // use BorrowedFd to satisfy AsFd bound on nix_write
            let raw = inner.get_ref().as_raw_fd();
            let fd  = unsafe { BorrowedFd::borrow_raw(raw) };
            match nix_write(fd, &bytes[off..]){
                Ok(n)=>Ok(n),
                Err(e)=>Err(std::io::Error::from_raw_os_error(e as i32)),
            }
        });
        match res{
            Ok(Ok(0)) => break,                 // wrote nothing -> done
            Ok(Ok(n)) => { off += n; }          // progress
            Ok(Err(e)) => return Err(e.into()), // real I/O error
            Err(_would_block) => continue,      // spurious readiness -> retry
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
