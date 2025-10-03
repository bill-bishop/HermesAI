#!/usr/bin/env python3
import subprocess
import os
import sys

ROOT_DIR = os.path.dirname(os.path.abspath(__file__))
EXEC_SANDBOX_DIR = os.path.join(ROOT_DIR, "apps", "execution-sandbox")

# Docker commands from execution-sandbox/README.md
DOCKER_COMMANDS = [
    "docker build -t sandbox_server:nightly sandbox_server && docker kill sandbox || true && sleep 3 && docker run --rm -d --network sandbox-net --name sandbox -v %CD%/../../:/sandbox --env-file .env sandbox_server:nightly",
    "cd ../dropcode-client && npm run build && cd ../../ && cp -R assets apps/execution-sandbox/terminal/html/ && cp apps/execution-sandbox/terminal/html/assets/favicon.ico apps/execution-sandbox/terminal/html/ && cd apps/execution-sandbox && docker build -t terminal:nightly terminal && docker kill sandbox-router || true && sleep 3 && docker run --rm -d --network sandbox-net --name sandbox-router -p 80:80 terminal:nightly"
]


def run(cmd, cwd=None):
    print(f"\n[CMD] {cmd}")
    result = subprocess.run(cmd, shell=True, cwd=cwd)
    return result.returncode == 0


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


def main():
    print("[Watcher] Checking for uncommitted changes...")
    if check_uncommitted_changes(ROOT_DIR):
        print("Uncommitted changes in monorepo root. Aborting.")
        sys.exit(1)
    if check_uncommitted_changes(EXEC_SANDBOX_DIR):
        print("Uncommitted changes in execution-sandbox. Aborting.")
        sys.exit(1)

    print("[Watcher] Checking for new commits...")
    root_new = sync_and_check_new_commits(ROOT_DIR)
    sandbox_new = sync_and_check_new_commits(EXEC_SANDBOX_DIR)

    if not (root_new or sandbox_new):
        print("No new commits. Nothing to do.")
        return

    print("New commits detected! Running nightly Docker build + run pipeline.")

    # Execute docker commands sequentially
    for cmd in DOCKER_COMMANDS:
        success = run(cmd, cwd=EXEC_SANDBOX_DIR)
        if not success:
            print("[Watcher] Command failed, aborting pipeline.")
            sys.exit(1)

    print("[Watcher] Nightly pipeline finished successfully.")


if __name__ == "__main__":
    main()