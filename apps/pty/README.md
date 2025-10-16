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
curl -N "http://localhost:8080/sessions/$SID/stream?from=0" &

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
  -d '{"cmd":["/bin/bash","-lc","echo start; for i in 1 2 3; do echo tick:$i; sleep 1; done; echo done"]}' \
  | jq -r .job_id)

curl -N "http://localhost:8080/stream/$J?from=0"
```

✅ **Expected behavior**

```
{"t":"stdout","d":"tick:1\n"}
{"t":"stdout","d":"tick:2\n"}
{"t":"stdout","d":"tick:3\n"}
{"t":"event","d":"exit:Some(0)"}
```

---

Would you like me to add a section that also shows how to gracefully close or resize the PTY (using `/close` and `/resize`)? Those endpoints are wired up and make a nice completeness touch.
