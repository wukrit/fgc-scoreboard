#!/usr/bin/env bash
# Start FGC Scoreboard server from repo root.
# Usage: ./scripts/start.sh [--port PORT] [other fgc-server flags]

set -e
ROOT="$(cd "$(dirname "$0")/../" && pwd)"
cd "$ROOT"

BIN="$ROOT/server/target/release/fgc-server"

if [ -x "$BIN" ]; then
  exec "$BIN" "$@"
fi

if command -v cargo >/dev/null 2>&1; then
  exec cargo run --manifest-path "$ROOT/server/Cargo.toml" --release -- "$@"
fi

echo "fgc-server not found. Build with: cargo build --release --manifest-path server/Cargo.toml"
exit 1
