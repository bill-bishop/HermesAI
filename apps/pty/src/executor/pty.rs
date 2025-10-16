use crate::models::StreamFrame;
use crate::state::SessionHandle;
use nix::pty::{openpty, Winsize};
use nix::unistd::{fork, ForkResult, setsid, dup2, close, execvp, read as nix_read};
use std::ffi::CString;
use std::os::fd::{AsRawFd, IntoRawFd, FromRawFd};
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
            let sh=CString::new(shell).unwrap();let args=&[sh.clone()];execvp(&sh,args).expect("exec failed");
            unsafe{libc::_exit(127);}
        }
        ForkResult::Parent{child}=>{
            let _=close(pty.slave.as_raw_fd());
            let master_file=unsafe{std::fs::File::from_raw_fd(pty.master.into_raw_fd())};
            let async_master=AsyncFd::new(master_file)?;
            let (tx,_)=tokio::sync::broadcast::channel::<StreamFrame>(1024);
            let latest_seq=Arc::new(parking_lot::Mutex::new(0u64));
            let exit_code=Arc::new(parking_lot::Mutex::new(None));
            let txr=tx.clone();let seqr=latest_seq.clone();

            tokio::spawn(async move{
                let mut buf=[0u8;4096];
                loop{
                    match async_master.readable().await {
                        Ok(mut guard) => {
                            // Use fd-based read so we don't need &mut File
                            let res = guard.try_io(|inner| {
                                let fd = inner.get_ref().as_raw_fd();
                                match nix_read(fd, &mut buf) {
                                    Ok(n) => Ok(n), // io::Result<usize>
                                    Err(e) => Err(std::io::Error::from_raw_os_error(e as i32)),
                                }
                            });

                            match res {
                                Ok(Ok(0)) => break,
                                Ok(Ok(n)) => {
                                    let s = String::from_utf8_lossy(&buf[..n]).to_string();
                                    let mut seq = seqr.lock();
                                    *seq += 1;
                                    let _ = txr.send(StreamFrame { t: "stdout".into(), seq: *seq, d: s });
                                }
                                Ok(Err(_e)) => break,
                                Err(_would_block) => continue,
                            }
                        }
                        Err(_) => break,
                    }

                }
            });

            Ok(SessionHandle{latest_seq,tx,exit_code,pid:child.as_raw()})
        }
    }
}

pub async fn write_pty(_h:&SessionHandle,_d:&str)->anyhow::Result<()> {Ok(())}
pub async fn resize_pty(_h:&SessionHandle,_c:u16,_r:u16)->anyhow::Result<()> {Ok(())}
pub async fn close_pty(_h:&SessionHandle)->anyhow::Result<()> {Ok(())}
