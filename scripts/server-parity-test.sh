#!/usr/bin/env bash
# Smoke test for fgc-server API and static routes.
# Usage: ./scripts/server-parity-test.sh [BASE_URL]

set -euo pipefail

BASE="${1:-http://127.0.0.1:8080}"

fail() { echo "FAIL: $1"; exit 1; }
pass() { echo "OK: $1"; }

echo "Testing $BASE"

# Health
body=$(curl -sf "$BASE/health")
[ "$body" = '{"ok":true}' ] || fail "health body=$body"
pass "GET /health"

# Static
curl -sf "$BASE/" | head -c 20 | grep -qi '<!DOCTYPE\|<html' || fail "GET /"
pass "GET /"

curl -sf "$BASE/overlay/scoreboard.html" | head -c 20 | grep -qi '<html' || fail "GET overlay"
pass "GET /overlay/scoreboard.html"

curl -sf "$BASE/css/pico.classless.min.css" | head -c 5 | grep -q '.' || fail "GET css"
pass "GET /css/pico.classless.min.css"

# Scoreboard GET
curl -sf "$BASE/scoreboard.json" | grep -q p1Name || fail "GET scoreboard.json"
pass "GET /scoreboard.json"

# Schema rejection
code=$(curl -s -o /dev/null -w '%{http_code}' -X POST "$BASE/scoreboard.json" \
  -H 'Content-Type: application/json' -d '{"bad":1}')
[ "$code" = "400" ] || fail "invalid schema expected 400 got $code"
pass "POST invalid schema -> 400"

# Unknown path
code=$(curl -s -o /dev/null -w '%{http_code}' "$BASE/not-found-path")
[ "$code" = "404" ] || fail "unknown path expected 404 got $code"
pass "unknown path -> 404"

echo "All checks passed."
