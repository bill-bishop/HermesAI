#!/usr/bin/env python3
import subprocess
import time
import requests

HEALTHCHECK_URL = "https://hermesai.dev/api/healthcheck"
CHECK_INTERVAL = 30  # seconds


def server_healthy():
    try:
        resp = requests.get(HEALTHCHECK_URL, timeout=5)
        return resp.status_code == 200
    except Exception:
        return False


def run(cmd):
    print(f"[KeepAlive] Running: {cmd}")
    subprocess.run(cmd, shell=True)


def restart_tunnel():
    print("[KeepAlive] Restarting Cloudflare tunnel...")
    try:
        # Find and kill existing cloudflared processes on Windows
        result = subprocess.run("tasklist | findstr cloudflared", shell=True, capture_output=True, text=True)
        if result.returncode == 0:
            for line in result.stdout.strip().splitlines():
                try:
                    pid = line.split()[1]
                    print(f"[KeepAlive] Killing cloudflared PID {pid}")
                    subprocess.run(f"taskkill /PID {pid} /F", shell=True)
                except Exception:
                    pass
    except Exception as e:
        print(f"[KeepAlive] Error while killing cloudflared: {e}")

    # Restart tunnel
    return subprocess.Popen(["cloudflared", "tunnel", "run", "hermesai"])


def restart_docker_containers():
    print("[KeepAlive] Restarting latest containers...")
    run("docker kill sandbox sandbox-router || true")
    time.sleep(3)
    run("docker run --rm -d --network sandbox-net --name sandbox -v %CD%:/sandbox --env-file ./apps/execution-sandbox/.env sandbox_server:latest")
    run("docker run --rm -d --network sandbox-net --name sandbox-router -p 80:80 terminal:latest")


def main():
    print("[KeepAlive] Initial startup check...")
    if not server_healthy():
        print("[KeepAlive] Initial healthcheck failed! Restarting services immediately...")
        restart_docker_containers()
        restart_tunnel()
        print("[KeepAlive] Startup complete. Sleeping 30 seconds before retry...")
        time.sleep(CHECK_INTERVAL)
    else:
        print("[KeepAlive] Initial healthcheck OK.")

    while True:
        if not server_healthy():
            print("[KeepAlive] Healthcheck failed! Restarting services...")
            restart_docker_containers()
            restart_tunnel()
            print("[KeepAlive] Sleeping 30 seconds before retry...")
            time.sleep(CHECK_INTERVAL)
        else:
            print("[KeepAlive] Healthcheck OK.")
            time.sleep(CHECK_INTERVAL)


if __name__ == "__main__":
    main()