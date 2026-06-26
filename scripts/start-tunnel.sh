#!/usr/bin/env bash
# Start FGC Scoreboard server + Cloudflare Tunnel
# Usage: ./scripts/start-tunnel.sh [--port PORT]

set -e

ROOT="$(cd "$(dirname "$0")/../" && pwd)"
cd "$ROOT"

PORT="${1:-8080}"
if [ "$1" = "--port" ]; then
  PORT="$2"
fi

command -v cloudflared >/dev/null 2>&1 || { echo "cloudflared required — brew install cloudflared"; exit 1; }

if lsof -i :"$PORT" >/dev/null 2>&1; then
  echo "Port $PORT already in use, killing existing process..."
  lsof -i :"$PORT" | grep -v "^COMMAND" | awk '{print $2}' | xargs kill -9 2>/dev/null || true
  sleep 1
fi

cleanup() {
  echo ""
  echo "Shutting down..."
  kill "$TUNNEL_PID" 2>/dev/null || true
  kill "$SERVER_PID" 2>/dev/null || true
  sleep 3
  kill -9 "$TUNNEL_PID" 2>/dev/null || true
  kill -9 "$SERVER_PID" 2>/dev/null || true
  wait 2>/dev/null || true
  echo "Done."
}
trap cleanup INT TERM EXIT

"$ROOT/scripts/start.sh" --port "$PORT" &
SERVER_PID=$!

sleep 1

echo ""
echo "Starting Cloudflare Tunnel..."
echo ""

cloudflared tunnel &
TUNNEL_PID=$!

echo ""
echo "FGC Scoreboard is live!"
echo "  Check cloudflared output above for your tunnel URL."
echo "  Controller: /"
echo "  Overlay:    /overlay/scoreboard.html"
echo ""
echo "Press Ctrl+C to stop."
echo ""

wait
