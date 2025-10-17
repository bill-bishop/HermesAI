use std::collections::VecDeque;
use std::sync::Arc;

use parking_lot::Mutex;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use crate::models::StreamFrame;
use crate::state::JobHandle;

pub async fn spawn_noninteractive(cmd: Vec<String>, cwd: Option<String>) -> JobHandle {
    // Always execute via bash -lc so shell features work.
    // Accept either a single string or an array of tokens; join arrays with spaces.
    let joined = if cmd.len() == 1 {
        cmd[0].clone()
    } else {
        cmd.join(" ")
    };

    let mut c = Command::new("/bin/bash");
    c.arg("-lc").arg(joined);

    if let Some(dir) = cwd { c.current_dir(dir); }
    c.stdout(std::process::Stdio::piped());
    c.stderr(std::process::Stdio::piped());

    let mut child = c.spawn().expect("spawn failed");
    let (tx, _rx) = tokio::sync::broadcast::channel::<StreamFrame>(1024);
    let latest_seq = Arc::new(Mutex::new(0u64));
    let exit_code = Arc::new(Mutex::new(None));
    let backlog = Arc::new(Mutex::new(VecDeque::with_capacity(1024)));

    let push = |frame: StreamFrame, tx: &tokio::sync::broadcast::Sender<StreamFrame>, backlog: &Arc<Mutex<VecDeque<StreamFrame>>>| {
        let mut b = backlog.lock();
        if b.len() == b.capacity() { b.pop_front(); }
        b.push_back(frame.clone());
        let _ = tx.send(frame);
    };

    if let Some(out) = child.stdout.take() {
        let txc = tx.clone(); let seqc = latest_seq.clone(); let backc = backlog.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(out).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                let mut s = seqc.lock(); *s += 1;
                let frame = StreamFrame{ t:"stdout".into(), seq:*s, d: format!("{line}\n") };
                push(frame, &txc, &backc);
            }
        });
    }
    if let Some(err) = child.stderr.take() {
        let txc = tx.clone(); let seqc = latest_seq.clone(); let backc = backlog.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(err).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                let mut s = seqc.lock(); *s += 1;
                let frame = StreamFrame{ t:"stderr".into(), seq:*s, d: format!("{line}\n") };
                push(frame, &txc, &backc);
            }
        });
    }
    {
        let txe = tx.clone(); let seqe = latest_seq.clone(); let exite = exit_code.clone(); let backe = backlog.clone();
        tokio::spawn(async move {
            let status = child.wait().await.ok();
            let code = status.and_then(|s| s.code());
            *exite.lock() = code;
            let mut s = seqe.lock(); *s += 1;
            let frame = StreamFrame{ t:"event".into(), seq:*s, d: format!("exit:{:?}", code) };
            let mut b = backe.lock();
            if b.len() == b.capacity() { b.pop_front(); }
            b.push_back(frame.clone());
            let _ = txe.send(frame);
        });
    }

    JobHandle { latest_seq, tx, exit_code, backlog }
}
