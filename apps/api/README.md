Absolutely — here’s your full **HermesAI Agent API (v0.1)** spec, written in the same concise, production-ready style you used for `/agent/sandbox`.

---

## ⚙️ **HermesAI Agent API (v0.1)**

Base URL: `https://hermesai.dev/agent`
All endpoints require:

```
Authorization: Bearer <token>
Content-Type: application/json
```

Each token is mapped to its own containerized sandbox.

---

### 🧠 **1. Terminal API**

#### **POST /agent/terminal**

Execute a non-interactive command inside the user’s sandbox.

**Body:**

```json
{ "cmd": "echo hello" }
```

**Response 200:**

```
STDOUT:
hello

STDERR:

EXIT CODE:
0
```

**Behavior:**

* Executes via the container’s PTY service.
* Streams output for up to ~10s before returning a summary.
* Truncates very long outputs (>1000 chars).

**Example:**

```bash
curl -X POST \
  -H "Authorization: Bearer <token>" \
  -H "content-type: application/json" \
  -d '{"cmd":"echo hello from terminal"}' \
  https://hermesai.dev/agent/terminal
```

---

#### **GET /agent/terminal**

Fetch the most recent terminal output (tail view).

**Response 200:**

```
last few lines of output
(... process still running ...)
```

**Example:**

```bash
curl -H "Authorization: Bearer <token>" \
  https://hermesai.dev/agent/terminal
```

---

### 🗂️ **2. Sandbox File API**

#### **GET /agent/sandbox/{*path}**

Read a file from the user’s sandbox (`/sandbox/{path}` in container).

**Response 200:** Raw file contents (UTF-8).
**Response 404:** `file not found`.

**Example:**

```bash
curl -H "Authorization: Bearer <token>" \
  https://hermesai.dev/agent/sandbox/src/main.rs
```

---

#### **POST /agent/sandbox/{*path}**

Create or overwrite a file inside the user’s sandbox.

**Body:**

```json
{ "content": "print('hello world')" }
```

**Response 200:**

```json
{ "ok": true, "path": "/sandbox/src/main.rs" }
```

**Example:**

```bash
curl -X POST \
  -H "Authorization: Bearer <token>" \
  -H "content-type: application/json" \
  -d '{"content":"print(\\\"hello world\\\")"}' \
  https://hermesai.dev/agent/sandbox/src/main.rs
```

---

### ✅ **Summary Table**

| Endpoint                 | Method | Description                |
| ------------------------ | ------ | -------------------------- |
| `/agent/terminal`        | `POST` | Execute command in sandbox |
| `/agent/terminal`        | `GET`  | Get last terminal output   |
| `/agent/sandbox/{*path}` | `GET`  | Read a sandbox file        |
| `/agent/sandbox/{*path}` | `POST` | Write a sandbox file       |

---

Everything now maps 1-to-1 to your per-user PTY containers and `/sandbox/` volumes — concise, self-contained, and ready for docs or API consumers.
