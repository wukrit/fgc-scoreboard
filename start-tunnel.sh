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

# Cleanup on exit
cleanup() {
  echo ""
  echo "Shutting down..."
  kill "$TUNNEL_PID" 2>/dev/null || true
  kill "$SERVER_PID" 2>/dev/null || true
  # Give processes 3 seconds to exit gracefully
  sleep 3
  kill -9 "$TUNNEL_PID" 2>/dev/null || true
  kill -9 "$SERVER_PID" 2>/dev/null || true
  wait 2>/dev/null || true
  echo "Done."
}
trap cleanup INT TERM EXIT

# Start server in background
python3 server.py --port "$PORT" &
SERVER_PID=$!

# Give server a moment to start
sleep 1

echo ""
echo "Starting Cloudflare Tunnel..."
echo ""

# Start tunnel in background
cloudflared tunnel &
TUNNEL_PID=$!

echo ""
echo "FGC Scoreboard is live!"
echo "  Check cloudflared output above for your tunnel URL."
echo "  Append /controller.html for the controller."
echo "  Append /_overlays/scoreboard.html for the overlay."
echo ""
echo "Press Ctrl+C to stop."
echo ""

# Wait for either process to exit
wait
