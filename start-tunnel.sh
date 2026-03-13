#!/usr/bin/env bash
# Start FGC Scoreboard server + Cloudflare Tunnel
# Usage: ./start-tunnel.sh [--port PORT]

set -e

PORT="${1:-8080}"
if [ "$1" = "--port" ]; then
  PORT="$2"
fi

# Check dependencies
command -v python3 >/dev/null 2>&1 || { echo "python3 required"; exit 1; }
command -v cloudflared >/dev/null 2>&1 || { echo "cloudflared required — brew install cloudflared"; exit 1; }

# Start server in background
python3 server.py --port "$PORT" &
SERVER_PID=$!

# Give server a moment to start
sleep 1

echo ""
echo "Starting Cloudflare Tunnel..."
echo ""

# Start tunnel (foreground so Ctrl+C stops everything)
cloudflared tunnel run fgc-scoreboard &
TUNNEL_PID=$!

echo ""
echo "FGC Scoreboard is live!"
echo "  Controller: https://fgc.sukritwalia.com/controller.html"
echo "  Overlay:    https://fgc.sukritwalia.com/_overlays/scoreboard.html"
echo ""
echo "Press Ctrl+C to stop."
echo ""

# Cleanup on exit
cleanup() {
  echo ""
  echo "Shutting down..."
  kill "$TUNNEL_PID" 2>/dev/null
  kill "$SERVER_PID" 2>/dev/null
  wait "$TUNNEL_PID" 2>/dev/null
  wait "$SERVER_PID" 2>/dev/null
  echo "Done."
}
trap cleanup INT TERM

# Wait for either process to exit
wait
