#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
HTML_PATH="$ROOT_DIR/gui/index.html"

if [ ! -f "$HTML_PATH" ]; then
  echo "Cannot find GUI entrypoint: $HTML_PATH"
  exit 1
fi

if command -v xdg-open >/dev/null 2>&1; then
  xdg-open "$HTML_PATH"
elif command -v gio >/dev/null 2>&1; then
  gio open "$HTML_PATH"
else
  echo "Please open the file in a browser: $HTML_PATH"
  exit 1
fi
