#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")"
MODE="${1:-build}"

command -v npm >/dev/null 2>&1 || { echo "Node.js/npm is required." >&2; exit 1; }
command -v cargo >/dev/null 2>&1 || { echo "Rust/Cargo is required." >&2; exit 1; }

if [ ! -d node_modules ]; then
  npm ci
fi

case "$MODE" in
  dev)
    npx tauri dev
    ;;
  build)
    npx tauri build --bundles deb
    ;;
  *)
    echo "Usage: ./build.sh [dev|build]" >&2
    exit 2
    ;;
esac
