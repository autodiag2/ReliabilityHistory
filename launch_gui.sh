#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN_PATH="$ROOT_DIR/target/release/reliability_history"

if [ ! -x "$BIN_PATH" ]; then
  BIN_PATH="$ROOT_DIR/target/debug/reliability_history"
fi

if [ ! -x "$BIN_PATH" ]; then
  echo "GUI binary not found or not built at $ROOT_DIR/target/release/reliability_history or $ROOT_DIR/target/debug/reliability_history"
  echo "Build with: (cd $ROOT_DIR && cargo build --release) or cargo build"
  exit 1
fi

echo "Launching GUI..."
"$BIN_PATH" &
PID=$!
echo "Started reliability_history (pid $PID)"
