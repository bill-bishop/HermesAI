# isolated-exec 



build & run the docker file and test it with `curl` and `jq`:

very hacky testing method for now:

CAT sanity (must echo):

# terminal A (attach first)
```bash
SID=$(curl -s -X POST localhost:8080/sessions \
-H 'content-type: application/json' \
-d '{"shell":"/bin/cat","cols":100,"rows":28}' | jq -r .session_id)
```

Copy the SID:
```bash
echo $SID
```

```bash
curl -N "http://localhost:8080/sessions/$SID/stream?from=0"
```

# terminal B

Paste the SID into terminal B:
```bash
SID=<PastedSID>
```

```bash
curl -s -X POST "http://localhost:8080/sessions/$SID/write" \
-H 'content-type: application/json' -d '{"data":"hello world\r\n"}'
```


You should see in server logs:

```bash
write_session: 13 bytes to s_...
PTY wrote 13 bytes (total 13)
PTY read 13 bytes
```


…and the stream prints hello world in terminal A.

BASH next (CR is key):

```bash
SID=$(curl -s -X POST localhost:8080/sessions \
-H 'content-type: application/json' \
-d '{"shell":"/bin/bash","cols":100,"rows":28}' | jq -r .session_id)
curl -N "http://localhost:8080/sessions/$SID/stream?from=0" &
curl -s -X POST "http://localhost:8080/sessions/$SID/write" \
-H 'content-type: application/json' -d '{"data":"echo hello from bash\r"}'
```
