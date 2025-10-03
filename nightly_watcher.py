#!/usr/bin/env python3
import subprocess
import os
import sys
import time
from datetime import datetime
import requests

ROOT_DIR = os.path.dirname(os.path.abspath(__file__))
EXEC_SANDBOX_DIR = os.path.join(ROOT_DIR, "apps", "execution-sandbox")

# Split Docker pipeline into steps (build -> kill -> pause -> run)
DOCKER_PIPELINES = [
    {
        "name": "sandbox_server",
        "build": "docker build -t sandbox_server:nightly sandbox_server",
        "kill": "docker kill sandbox || true",
        "pause": "sleep 3",
        "run": "docker run --rm -d --network sandbox-net --name sandbox -v %CD%/../../:/sandbox --env-file .env sandbox_server:nightly",
    },
    {
        "name": "terminal",
        "build": "cd ../dropcode-client && npm run build && cd ../../ && cp -R assets apps/execution-sandbox/terminal/html/ && cp apps/execution-sandbox/terminal/html/assets/favicon.ico apps/execution-sandbox/terminal/html/ && cd apps/execution-sandbox && docker build -t terminal:nightly terminal",
        "kill": "docker kill sandbox-router || true",
        "pause": "sleep 3",
        "run": "docker run --rm -d --network sandbox-net --name sandbox-router -p 80:80 terminal:nightly",
    },
]


def run(cmd, cwd=None, abort_on_fail=False):
    print(f"\n[CMD] {cmd}")
    result = subprocess.run(cmd, shell=True, cwd=cwd)
    if result.returncode != 0:
        print(f"[Watcher] Command failed: {cmd}")
        if abort_on_fail:
            return False
    return True


def check_uncommitted_changes(repo_dir):
    status = subprocess.check_output(["git", "status", "--porcelain"], cwd=repo_dir, text=True)
    return bool(status.strip())


def sync_and_check_new_commits(repo_dir):
    subprocess.run(["git", "fetch"], cwd=repo_dir, check=True)
    status = subprocess.check_output(["git", "status", "-sb"], cwd=repo_dir, text=True)

    if "[ahead" in status:
        print(f"[Watcher] {repo_dir} has local commits ahead. Pushing...")
        subprocess.run(["git", "push"], cwd=repo_dir, check=True)
        return True  # treat as new commits

    if "[behind" in status:
        print(f"[Watcher] {repo_dir} is behind remote. Please pull manually to avoid overwriting.")
        return False

    local = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=repo_dir, text=True).strip()
    remote = subprocess.check_output(["git", "rev-parse", "@{u}"], cwd=repo_dir, text=True).strip()
    return local != remote


def server_healthy():
    try:
        resp = requests.get("https://dropcode.org/api/healthcheck", timeout=5)
        return resp.status_code == 200
    except Exception:
        return False


def run_pipeline():
    for pipe in DOCKER_PIPELINES:
        print(f"[Watcher] Running pipeline: {pipe['name']}")
        # Build must succeed, otherwise abort this pipeline
        if not run(pipe["build"], cwd=EXEC_SANDBOX_DIR, abort_on_fail=True):
            print(f"[Watcher] Build failed for {pipe['name']}, skipping remaining steps.")
            continue
        # Kill, pause, and run can fail without aborting CI
        run(pipe["kill"], cwd=EXEC_SANDBOX_DIR)
        run(pipe["pause"], cwd=EXEC_SANDBOX_DIR)
        run(pipe["run"], cwd=EXEC_SANDBOX_DIR)


def main():
    while True:
        ts = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
        print(f"\n[Watcher] === Cycle start @ {ts} ===")

        # If server is unhealthy, force pipeline run
        if not server_healthy():
            print("[Watcher] Healthcheck failed! Stashing pending changes and running CI.")
            # Stash pending changes if any
            if check_uncommitted_changes(ROOT_DIR):
                subprocess.run(["git", "stash", "push", "-m", "watcher-autostash"], cwd=ROOT_DIR)
            if check_uncommitted_changes(EXEC_SANDBOX_DIR):
                subprocess.run(["git", "stash", "push", "-m", "watcher-autostash"], cwd=EXEC_SANDBOX_DIR)
            run_pipeline()
            print("[Watcher] Sleeping for 60 seconds...")
            time.sleep(60)
            continue

        # Normal flow: check for commits
        if check_uncommitted_changes(ROOT_DIR):
            print("Uncommitted changes in monorepo root. Aborting this cycle.")
            time.sleep(60)
            continue
        if check_uncommitted_changes(EXEC_SANDBOX_DIR):
            print("Uncommitted changes in execution-sandbox. Aborting this cycle.")
            time.sleep(60)
            continue

        print("[Watcher] Checking for new commits...")
        root_new = sync_and_check_new_commits(ROOT_DIR)
        sandbox_new = sync_and_check_new_commits(EXEC_SANDBOX_DIR)

        if root_new or sandbox_new:
            print("New commits detected! Running nightly Docker pipelines.")
            run_pipeline()
        else:
            print("No new commits detected.")

        # Sleep before checking again
        print("[Watcher] Sleeping for 60 seconds...")
        time.sleep(60)


if __name__ == "__main__":
    main()