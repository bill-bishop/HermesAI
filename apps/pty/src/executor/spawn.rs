use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use parking_lot::Mutex;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use crate::models::StreamFrame;
use crate::state::JobHandle;

const BACKLOG_CAP: usize = 1024;

pub async fn spawn_noninteractive(cmd: Vec<String>, cwd: Option<String>) -> JobHandle {
    // Build command
    let joined = if cmd.len() == 1 { cmd[0].clone() } else { cmd.join(" ") };

    let mut c = Command::new("/bin/bash");
    c.arg("-lc").arg(joined.clone());
    if let Some(dir) = cwd { c.current_dir(dir); }
    c.stdout(std::process::Stdio::piped());
    c.stderr(std::process::Stdio::piped());
    c.env("TERM", "xterm");

    let child = Arc::new(tokio::sync::Mutex::new(c.spawn().expect("spawn failed")));
    let child_watcher = Arc::clone(&child);

    // Shared state
    let (tx, _rx) = tokio::sync::broadcast::channel::<StreamFrame>(BACKLOG_CAP);
    let latest_seq = Arc::new(AtomicU64::new(0));
    let exit_code  = Arc::new(Mutex::new(None::<i32>));
    let backlog    = Arc::new(Mutex::new(VecDeque::with_capacity(BACKLOG_CAP)));

    // Helper for broadcasting frames
    let push = {
        let latest_seq = Arc::clone(&latest_seq);
        let backlog    = Arc::clone(&backlog);
        let tx         = tx.clone();
        move |t: &str, data: String| {
            let seq = latest_seq.fetch_add(1, Ordering::Relaxed) + 1;
            let frame = StreamFrame { t: t.into(), seq, d: data };
            {
                let mut b = backlog.lock();
                if b.len() == b.capacity() { b.pop_front(); }
                b.push_back(frame.clone());
            }
            let _ = tx.send(frame);
        }
    };

    push("event", "stream-start".into());

    // Spawn readers and keep their JoinHandles
    let stdout_task = if let Some(out) = child_watcher.lock().await.stdout.take() {
        let push = push.clone();
        Some(tokio::spawn(async move {
            let mut reader = BufReader::new(out);
            let mut buf = Vec::with_capacity(256);
            loop {
                buf.clear();
                match reader.read_until(b'\n', &mut buf).await {
                    Ok(0) => {
                        if !buf.is_empty() {
                            push("stdout", String::from_utf8_lossy(&buf).into());
                        }
                        break;
                    }
                    Ok(_) => push("stdout", String::from_utf8_lossy(&buf).into()),
                    Err(e) => {
                        push("event", format!("stdout-reader-error:{e}"));
                        break;
                    }
                }
            }
            push("event", "stdout-reader-done".into());
        }))
    } else {
        push("event", "stdout-none".into());
        None
    };

    let stderr_task = if let Some(err) = child_watcher.lock().await.stderr.take() {
        let push = push.clone();
        Some(tokio::spawn(async move {
            let mut reader = BufReader::new(err);
            let mut buf = Vec::with_capacity(256);
            loop {
                buf.clear();
                match reader.read_until(b'\n', &mut buf).await {
                    Ok(0) => {
                        if !buf.is_empty() {
                            push("stderr", String::from_utf8_lossy(&buf).into());
                        }
                        break;
                    }
                    Ok(_) => push("stderr", String::from_utf8_lossy(&buf).into()),
                    Err(e) => {
                        push("event", format!("stderr-reader-error:{e}"));
                        break;
                    }
                }
            }
            push("event", "stderr-reader-done".into());
        }))
    } else {
        push("event", "stderr-none".into());
        None
    };

    // Wait for process, then readers, then exit
    {
        let exit_code = Arc::clone(&exit_code);
        let push = push.clone();
        tokio::spawn(async move {
            let code = match child_watcher.lock().await.wait().await {
                Ok(status) => status.code(),
                Err(e) => {
                    push("event", format!("wait-error:{e}"));
                    None
                }
            };

            // Ensure both readers finish before exit event
            if let Some(t) = stdout_task { let _ = t.await; }
            if let Some(t) = stderr_task { let _ = t.await; }

            *exit_code.lock() = code;
            push("event", format!("exit:{code:?}"));
        });
    }

    tokio::task::yield_now().await;
    JobHandle { latest_seq, tx, exit_code, backlog, child }
}
