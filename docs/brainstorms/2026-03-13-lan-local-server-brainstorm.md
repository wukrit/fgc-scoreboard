# LAN Local Server for Cross-Browser Score Sync

**Date:** 2026-03-13
**Status:** Ready for planning

## What We're Building

A lightweight Python HTTP server (~30 lines, stdlib only) that replaces the localStorage fallback for LAN tournament use. The controller POSTs score JSON to the server, which writes it to a `scoreboard.json` file. The overlay polls that file via HTTP GET — reusing the exact same XHR polling logic it already uses for npoint.io.

This solves the core problem: localStorage only works within the same browser process, but at LAN tournaments the controller (phone/tablet) and overlay (OBS on a PC) are on different devices.

## Why This Approach

- **Reuses existing code:** The overlay's 1-second XHR polling loop works as-is — just pointed at `localhost:8080/scoreboard.json` instead of `api.npoint.io/<id>`.
- **Minimal new code:** The server is ~30 lines of Python using only `http.server` (no pip dependencies). The controller needs a small change to POST to a local URL instead of npoint.io.
- **No internet required:** Works fully offline at venues with no/bad WiFi.
- **Python is ubiquitous:** Pre-installed on macOS/Linux; common on tournament PCs.

### Approaches Considered and Rejected

1. **BroadcastChannel API** — Same-browser-only, doesn't solve cross-device.
2. **WebRTC P2P** — Complex signaling, overkill for LAN.
3. **WebSocket server** — Works but requires new client-side code. HTTP polling reuses what exists.
4. **File System Access API** — Chrome-only, doesn't work in OBS/CEF.

## Key Decisions

1. **Python stdlib HTTP server** — No dependencies, handles POST (write JSON) and GET (serve JSON).
2. **Full static file server** — Serves overlay HTML/CSS/JS files too. One command starts everything — no need for GitHub Pages or `file://` at tournaments.
3. **Auto-detect localhost** — Overlay tries `localhost:8080` first. If unreachable, falls back to `?bin=` (npoint.io) or localStorage. Zero config when running on the same machine.
4. **Same JSON format** — Identical `scoreboard.json` structure. Scores remain strings. No schema changes.
5. **CORS headers** — Server must send `Access-Control-Allow-Origin: *` since controller and overlay may be on different origins.
6. **Priority order:** local server > npoint.io (`?bin=`) > localStorage (same browser fallback).
7. **Port 8080 default with CLI override** — `python3 server.py` uses 8080, `python3 server.py --port 9090` overrides. Overlay auto-detect always tries 8080.
8. **Status page at root** — `http://localhost:8080/` shows LAN IP, links to controller and overlay URLs, and current scoreboard data. Helpful for sharing the URL with score operators at a tournament.

## Tournament Workflow

1. Run `python3 server.py` on the tournament PC.
2. Server prints: `Serving at http://192.168.1.5:8080/`
3. Open `http://192.168.1.5:8080/` on any device — status page shows links.
4. Score operator opens controller link on their phone.
5. OBS browser source points to the overlay link.
6. Done — no internet, no npoint.io, no same-browser requirement.
