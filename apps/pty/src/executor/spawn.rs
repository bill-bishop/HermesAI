use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use crate::models::StreamFrame;
use crate::state::JobHandle;
use parking_lot::Mutex;
use std::sync::Arc;

pub async fn spawn_noninteractive(cmd: Vec<String>, cwd: Option<String>) -> JobHandle {
    let mut c = Command::new(&cmd[0]);
    if cmd.len() > 1 {
        c.args(&cmd[1..]);
    }
    if let Some(dir) = cwd {
        c.current_dir(dir);
    }
    c.stdout(std::process::Stdio::piped());
    c.stderr(std::process::Stdio::piped());

    let mut child = c.spawn().expect("spawn failed");

    let (tx, _rx) = tokio::sync::broadcast::channel::<StreamFrame>(1024);
    let latest_seq = Arc::new(Mutex::new(0u64));
    let exit_code = Arc::new(Mutex::new(None));

    // stdout task
    if let Some(out) = child.stdout.take() {
        let txc = tx.clone();
        let seqc = latest_seq.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(out).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                let mut s = seqc.lock();
                *s += 1;
                let _ = txc.send(StreamFrame{ t: "stdout".into(), seq: *s, d: format!("{}\n", line)});
            }
        });
    }

    // stderr task
    if let Some(err) = child.stderr.take() {
        let txc = tx.clone();
        let seqc = latest_seq.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(err).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                let mut s = seqc.lock();
                *s += 1;
                let _ = txc.send(StreamFrame{ t: "stderr".into(), seq: *s, d: format!("{}\n", line)});
            }
        });
    }

    // waiter
    let txe = tx.clone();
    let seqe = latest_seq.clone();
    let exite = exit_code.clone();
    tokio::spawn(async move {
        let status = child.wait().await.ok();
        let code = status.and_then(|s| s.code());
        *exite.lock() = code;
        let mut s = seqe.lock();
        *s += 1;
        let _ = txe.send(StreamFrame{ t: "event".into(), seq: *s, d: format!("exit:{:?}", code)});
    });

    JobHandle {
        latest_seq,
        tx,
        exit_code,
        kill: Arc::new(Mutex::new(None)), // minimal for now
    }
}
