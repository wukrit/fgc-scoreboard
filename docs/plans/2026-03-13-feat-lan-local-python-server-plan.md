---
title: "feat: Add LAN-mode local Python HTTP server for cross-browser score sync"
type: feat
date: 2026-03-13
---

# feat: Add LAN-mode local Python HTTP server for cross-browser score sync

## Overview

Add a zero-dependency Python HTTP server (`server.py`) that enables cross-device score sync at LAN tournaments without internet. The controller (phone) POSTs scores to the server, which writes them to a JSON file. The overlay (OBS) polls that file via HTTP GET — reusing the existing polling logic. One command starts everything.

## Problem Statement / Motivation

The current localStorage fallback only works across tabs in the **same browser process**. At LAN tournaments, the controller runs on a phone and the overlay runs in OBS (Chromium Embedded Framework) on a separate PC. localStorage cannot bridge this gap. The npoint.io remote mode works but requires internet, which is unreliable at many tournament venues.

## Proposed Solution

A three-tier priority system (consistent in both overlay and controller):

1. **npoint.io** — If `?bin=` URL parameter exists, use npoint.io (existing behavior, always wins)
2. **LAN server** — If page served over `http:` (not `file://`), poll own origin
3. **localStorage** — Fallback for `file://` same-browser dev/testing (existing behavior)

### Tournament Workflow

1. Run `python3 server.py` on the tournament PC
2. Server prints controller + overlay URLs with LAN IP to terminal
3. Score operator opens controller URL on phone
4. OBS browser source points to overlay URL
5. Done — no internet, no npoint.io, no same-browser requirement

## Technical Approach

### New File: `server.py`

Python 3.6+ stdlib-only HTTP server (~40 lines). Subclass `SimpleHTTPRequestHandler`, override only what's needed:

| Method | Path | Behavior |
|---|---|---|
| `GET /scoreboard.json` | Read scores | Returns JSON file with `Content-Type: application/json` (200), or default empty state if file missing |
| `POST /scoreboard.json` | Write scores | Validates JSON, writes to disk atomically, returns 200 |
| `GET /*` | Static files | Delegates to `super().do_GET()` (serves repo root) |

**Atomic writes:** Write to `scoreboard.json.tmp`, then `os.replace()` to `scoreboard.json`. Prevents corrupted reads during mid-write polls.

**JSON validation on POST:** Parse the body with `json.loads()` before writing. Return 400 on invalid JSON. This is three lines and prevents corrupted data from reaching the overlay.

**Initial state:** On startup, if `scoreboard.json` doesn't exist, create it with the default empty state:

```json
{
  "p1Name": "", "p1Team": "", "p1Score": "0",
  "p2Name": "", "p2Team": "", "p2Score": "0",
  "round": "", "game": "",
  "cTitle1": "", "cTitle2": "",
  "mText1": "", "mText2": "", "mText3": "", "mText4": "",
  "timestamp": ""
}
```

**LAN IP detection:** Use the UDP socket trick (`socket.socket(AF_INET, SOCK_DGRAM)` connect to `8.8.8.8:80`, read `getsockname()[0]`). Catch `OSError` and fall back to `127.0.0.1` if no default route exists.

**Startup output:** Print controller and overlay URLs to stdout:
```
FGC Scoreboard Server
Controller: http://192.168.1.5:8080/controller.html
Overlay:    http://192.168.1.5:8080/_overlays/scoreboard.html
```

**CLI:** `python3 server.py [--port PORT]`. Default port 8080.

### Modified File: `_overlays/js/scoreboard.js`

**Mode detection (one-liner):**

```javascript
var lanMode = !binId && window.location.protocol === 'http:';
```

This works because:
- GitHub Pages uses `https:` → won't trigger
- `file://` URLs → won't trigger
- Local Python server serves over `http:` → triggers correctly

**Priority order (consistent with controller):**
```
if (binId) → npoint.io polling mode (existing code)
else if (lanMode) → LAN server polling mode (new)
else → localStorage fallback mode (existing code)
```

**Polling implementation:** Same pattern as npoint.io branch, but URL is `window.location.origin + '/scoreboard.json'`. Keep `?v=cBust` cache busting. Keep the 1-second interval.

**Error handling:** Silent. On fetch failure, skip update and retry next cycle.

### Modified File: `controller.html`

**Same detection and priority:**
```javascript
var lanMode = !binId && window.location.protocol === 'http:';
```

```
if (binId) → remote mode (npoint.io) — existing
else if (lanMode) → LAN mode (POST to own origin) — new
else → localStorage fallback — existing
```

**LAN mode save:** `fetch(window.location.origin + '/scoreboard.json', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(formData) })`.

**LAN mode load:** On page open, `fetch(window.location.origin + '/scoreboard.json')` → `populateForm(data)`.

**UI indicator:** Show "LAN mode — connected to [hostname:port]" in the status area.

**Error feedback:** On POST failure, show "Save failed — check server connection".

### New: `.gitignore` entry

Add `scoreboard.json` to `.gitignore` so the server's data file doesn't dirty the working tree.

### Modified File: `CLAUDE.md`

Add a "LAN Mode" section documenting the three-tier priority, the `server.py` usage, and the tournament workflow.

## Acceptance Criteria

### Functional

- [x] `python3 server.py` starts a server on port 8080, prints controller + overlay URLs
- [x] `python3 server.py --port 9090` overrides the port
- [x] `POST /scoreboard.json` validates JSON, writes to disk atomically, returns 200
- [x] `GET /scoreboard.json` returns current JSON with `Content-Type: application/json`
- [x] Controller served from the server auto-detects LAN mode, POSTs to own origin on Save
- [x] Controller loads current state from server on page open in LAN mode
- [x] Overlay served from the server auto-detects LAN mode, polls own origin every 1s
- [x] Score changes trigger the same animated transitions as npoint.io mode
- [x] All three modes still work: npoint.io (`?bin=`), LAN server, localStorage
- [x] `?bin=` always takes priority over LAN mode in both overlay and controller
- [x] Scores remain strings throughout the round-trip (no type coercion)
- [x] `scoreboard.json` is in `.gitignore`

### Edge Cases

- [x] Server creates default `scoreboard.json` on first startup
- [x] Overlay handles missing/invalid JSON gracefully (no error on stream)
- [x] Controller shows clear error feedback when server is unreachable
- [x] GitHub Pages deployment still works (`https:` protocol, LAN mode not triggered)
- [x] OBS browser source works when pointed to `http://<LAN_IP>:8080/_overlays/scoreboard.html`
- [x] Server returns 400 on invalid JSON POST body

## Implementation Phases

### Phase 1: Python Server (`server.py`) + `.gitignore`

Create the server:
- Subclass `SimpleHTTPRequestHandler`
- Override `do_POST` for `/scoreboard.json` (validate JSON, atomic write)
- Override `do_GET` to intercept `/scoreboard.json` (read + Content-Type header), delegate rest to parent
- Print controller/overlay URLs on startup with LAN IP
- `--port` CLI argument
- Create default `scoreboard.json` if missing
- Add `scoreboard.json` to `.gitignore`

**Files:** `server.py` (new), `.gitignore` (new or modified)

### Phase 2: Overlay LAN Mode (`scoreboard.js`)

Add one-line LAN detection and polling branch. Priority: `?bin=` > LAN > localStorage.

**Files:** `_overlays/js/scoreboard.js`

### Phase 3: Controller LAN Mode (`controller.html`)

Add LAN mode detection, POST-to-origin save, GET-on-load, and UI status indicator.

**Files:** `controller.html`

### Phase 4: Documentation

Update CLAUDE.md with the new LAN mode section and tournament workflow.

**Files:** `CLAUDE.md`

## Dependencies & Risks

| Risk | Mitigation |
|---|---|
| Python 3 not installed on tournament PC | Document requirement. Server prints clear error if run with Python 2. |
| Port 8080 already in use | `--port` flag. Server prints clear error on bind failure. |
| Multiple network interfaces (wrong IP printed) | UDP socket trick finds primary outbound IP. Catch OSError, fall back to 127.0.0.1. |
| Concurrent POSTs from multiple operators | Last write wins (atomic replace). Document single-operator assumption. |
| LAN security (anyone on WiFi can POST) | Accepted risk for simplicity. Same risk model as npoint.io (no auth). |

## Review Feedback Applied

Changes from plan review (DHH, Kieran, Simplicity reviewers):

1. **Fixed priority order** — `?bin=` always wins in both overlay and controller (was inconsistent)
2. **Simplified mode detection** — `!binId && protocol === 'http:'` one-liner replaces `isServedByLocalServer()` function with hostname checks
3. **Removed CORS/OPTIONS** — Same-origin requests when served by the Python server; not needed
4. **Removed status page** — Print URLs to terminal instead (used once per tournament)
5. **Added JSON validation on POST** — Parse before writing, return 400 on invalid
6. **Added Content-Type: application/json** on GET responses
7. **Added .gitignore entry** for `scoreboard.json`
8. **Removed over-specified criteria** — 64KB limit, directory listing prevention, graceful SIGINT (all either YAGNI or free from stdlib)
9. **Added OSError catch** for LAN IP detection on machines with no default route

## References

- **Brainstorm:** `docs/brainstorms/2026-03-13-lan-local-server-brainstorm.md`
- **npoint.io solution doc:** `docs/solutions/integration-issues/streamcontrol-to-npoint-remote-controller.md`
- **localStorage fallback plan:** `docs/plans/2026-03-13-feat-local-storage-fallback-plan.md`
- **Current overlay init:** `_overlays/js/scoreboard.js:1-76`
- **Current controller modes:** `controller.html:235-325`
- **JSON format:** `controller.html:281-299` (`getFormData()`)
