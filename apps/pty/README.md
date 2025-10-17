oh that’s a *beautifully clear* readme already — just a few quick tweaks to match the new API (since we replaced the `shell` field with `{ "mode": "interactive", "profile": "bash" }`) and clarify the proper CR/LF and test order.

here’s the **updated and ready-to-paste section** for your README:

---

# 🧪 Testing `isolated-exec`

## 1️⃣ Build & Run

```bash
docker build -t pty:latest .
docker run -p 8080:8080 pty:latest
```

---

## 2️⃣ Start an Interactive Session (PTY)

### **Terminal A** — create & stream the PTY

```bash
SID=$(curl -s -X POST localhost:8080/sessions \
  -H 'content-type: application/json' \
  -d '{"mode":"interactive","profile":"cat","cols":100,"rows":28}' \
  | jq -r .session_id)

echo "Session ID: $SID"

curl -N "http://localhost:8080/sessions/$SID/stream?from=0"
```

### **Terminal B** — send input to that PTY

```bash
SID=<PASTE_SAME_SID_HERE>

curl -s -X POST "http://localhost:8080/sessions/$SID/write" \
  -H 'content-type: application/json' \
  -d '{"data":"hello world\r\n"}'
```

✅ **Expected behavior**

* Server logs show:

  ```
  write_session: 13 bytes to s_...
  PTY wrote 13 bytes (total 13)
  PTY read 13 bytes
  ```
* Terminal A prints `hello world` in its stream.

---

## 3️⃣ Bash Sanity Test

```bash
SID=$(curl -s -X POST localhost:8080/sessions \
  -H 'content-type: application/json' \
  -d '{"mode":"interactive","profile":"bash","cols":100,"rows":28}' \
  | jq -r .session_id)

# Start stream in background
curl -Ns "http://localhost:8080/sessions/$SID/stream?from=0"   | jq -r 'select(.t=="stdout" or .t=="stderr") | .d'

# Send a simple echo command with CR (important!)
curl -s -X POST "http://localhost:8080/sessions/$SID/write" \
  -H 'content-type: application/json' \
  -d '{"data":"echo hello from bash\r"}'
```

✅ **Expected behavior**

```
{"t":"stdout","seq":...,"d":"hello from bash\r\n"}
{"t":"event","seq":...,"d":"exit:None"}
```

---

## 4️⃣ Non-PTY (Batch) Execution Example

```bash
J=$(curl -s -X POST http://localhost:8080/exec \
  -H 'content-type: application/json' \
  -d '{"cmd":["echo start; for i in 1 2 3; do echo tick:$i; sleep 1; done; echo done"]}' \
  | jq -r .job_id)

curl -Ns "http://localhost:8080/stream/$J?from=0" | jq -r 'select(.t=="stdout" or .t=="stderr") | .d'
```

✅ **Expected behavior**

```
{"t":"stdout","d":"tick:1\n"}
{"t":"stdout","d":"tick:2\n"}
{"t":"stdout","d":"tick:3\n"}
{"t":"event","d":"exit:Some(0)"}
```

---

## 5️⃣ Resize the PTY (window change)

> Works for interactive shells that honor TIOCSWINSZ (bash, sh, zsh, busybox).

```bash
# assuming $SID is set
curl -s -X POST "http://localhost:8080/sessions/$SID/resize" \
  -H 'content-type: application/json' \
  -d '{"cols":140,"rows":40}'
```

**Optional sanity check (bash):**

```bash
# Terminal B: ask bash to report the new rows/cols
curl -s -X POST "http://localhost:8080/sessions/$SID/write" \
  -H 'content-type: application/json' \
  -d '{"data":"stty size\r"}'
```

You should see a line like `40 140` appear in the stream (Terminal A).

---

## 6️⃣ Close the PTY (graceful EOF)

```bash
# assuming $SID is set
curl -s -X POST "http://localhost:8080/sessions/$SID/close"
```

**Expected in stream (Terminal A):**

```
{"t":"event","seq":...,"d":"exit:None"}
```

> Note: `close` writes an EOT (^D). The shell should exit; the reader task emits the `exit:*` event and closes the stream.

---

## 7️⃣ Check Job/Session Status & Resume Stream

### Job status (non-PTY runs)

```bash
# assuming $J is a job_id from /exec response
curl -s "http://localhost:8080/status/$J" | jq
# -> { "state":"running"|"exited", "exit_code":..., "seq_latest": N }
```

### Resume a stream from a known sequence

If you’ve already received frames up to `seq = N`, you can resume from there:

```bash
curl -N "http://localhost:8080/sessions/$SID/stream?from=N"
# or for non-PTY jobs:
curl -N "http://localhost:8080/stream/$J?from=N"
```

This replays any backlog frames with `seq > N` and then continues live.

