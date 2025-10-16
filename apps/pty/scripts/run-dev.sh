#!/usr/bin/env bash
set -euo pipefail
RUST_LOG=${RUST_LOG:-info} cargo run
