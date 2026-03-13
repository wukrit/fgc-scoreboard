---
title: "feat: Remote Scoreboard Controller"
type: feat
date: 2026-03-13
status: reviewed
---

# feat: Remote Scoreboard Controller

## Overview

Replace the local StreamControl.exe + local JSON file workflow with a remote setup: a mobile-friendly web form hosted on GitHub Pages writes scoreboard data to a remote JSON store, and the OBS overlay polls that remote URL instead of a local file. This lets a score operator on a phone at the venue update scores while OBS runs on a separate machine.

## Problem Statement / Motivation

Currently, the score operator must use StreamControl.exe on the same machine running OBS (or on a machine sharing the local filesystem). This is limiting:
- StreamControl is Windows-only
- The operator must be physically at the OBS machine
- No way to update scores from a phone while walking around a tournament venue

## Proposed Solution

### JSON Store: npoint.io

**Why npoint.io over JSONBin.io:** JSONBin.io's free tier allows only 10,000 requests/day. At 1 poll/second, the overlay alone consumes 28,800 reads in an 8-hour tournament — exceeding the limit in ~3 hours. npoint.io has **no rate limits on reads**, is free, and has a dead-simple API. The tradeoff is no API key auth — anyone with the endpoint URL can write. This is acceptable because:
- The endpoint URL is a random ID (e.g., `https://api.npoint.io/a1b2c3d4e5f6`)
- Only the operator and OBS machine need to know it
- The data is non-sensitive (player names and scores)

**API — always use POST (full replace), not PATCH:**
- **Read:** `GET https://api.npoint.io/<id>` → returns JSON
- **Write:** `POST https://api.npoint.io/<id>` with full JSON body → replaces entire document
- CORS headers included by default

Using POST exclusively (never PATCH) keeps the controller as single source of truth and avoids ambiguity about merge semantics.

### Components

#### 1. Controller Web Form (`controller.html`)

A single-page mobile-friendly HTML form that replaces StreamControl. Hosted on GitHub Pages.

**Location:** Place in repo root (not `_overlays/`) because GitHub Pages with Jekyll ignores directories starting with `_`. Add a `.nojekyll` file to the repo root to disable Jekyll processing entirely. Note: this means all root files (README.md, LICENSE, etc.) will be publicly accessible via GitHub Pages — acceptable since nothing is sensitive.

**Fields (matching existing JSON schema):**
- Player 1: name (text), team (text), score (number 0-99)
- Player 2: name (text), team (text), score (number 0-99)
- Round: text input
- Game: `<input>` with `<datalist>` for autocomplete suggestions (presets: BBCF, BBTAG, DBFZ, GGXRD, KOFXIV, MVCI, SF6, SFVCE, TEKKEN7, UMVC3, UNICLR, USF4). This is native HTML, zero JavaScript, and handles both preset and custom games in a single element.
- Buttons: Save, Swap (P1↔P2 name+team+score), Reset (scores→0), Clear (all fields)

**Commentary/misc fields omitted.** The overlay doesn't render `cTitle1`, `cTitle2`, `mText1-4`. Adding form fields for data nobody sees is YAGNI. They can be added later if overlay support is built.

**Configuration:** The npoint endpoint ID is passed as a URL parameter:
`https://<user>.github.io/fgc-scoreboard/controller.html?bin=a1b2c3d4e5f6`

**Save behavior:** Explicit **Save button** (not auto-save). Reasons:
- Avoids partial data appearing on stream while the operator is mid-edit
- No debounce complexity
- Operator controls exactly when the overlay updates
- Mobile-friendly — one clear action

**On page load:** Fetch current state from the bin and populate the form, so a page refresh preserves state.

**Timestamp handling:** On every save, set `timestamp` to `Math.floor(Date.now() / 1000).toString()` to match the existing schema format.

**Error handling:**
- Show a status indicator: green dot = last save succeeded, red dot = last save failed
- On save failure, show a clear error message and keep form data intact so the operator can retry

#### 2. Overlay Update (`_overlays/js/scoreboard.js`)

Minimal changes to the existing polling logic:

**a. Remote URL configuration:**
The bin endpoint URL is configured via a URL parameter on `scoreboard.html`. The OBS browser source URL becomes:
`file:///path/to/_overlays/scoreboard.html?bin=a1b2c3d4e5f6`

In `scoreboard.js`, read the bin ID from the URL (must be outside `init()` or at the top of `init()` — do NOT add jQuery dependencies outside `init()` since jQuery loads after `scoreboard.js`):
```javascript
var urlParams = new URLSearchParams(window.location.search);
var binId = urlParams.get('bin');
var streamJSON = binId
  ? 'https://api.npoint.io/' + binId
  : '../sc/streamcontrol.json'; // fallback to local for backwards compat
```

**b. Polling interval:** Change `setInterval` from 500ms to 1000ms unconditionally (both local and remote). The difference is imperceptible for scoreboard updates and simplifies the code to one path.

**c. Error handling:** Wrap `JSON.parse` in try-catch. On any fetch or parse error, **silently skip the update cycle** and retry on the next poll. Never render error states on the overlay — this is a stream overlay visible to viewers.

**d. Cache busting:** Keep `?v=cBust` unconditionally for both local and remote. It's harmless on npoint URLs (ignored by the server) and avoids branching logic.

**e. Backwards compatibility:** If no `?bin=` parameter is provided, the overlay falls back to polling the local `../sc/streamcontrol.json` file — existing StreamControl users are unaffected.

#### 3. GitHub Pages Setup

- Add `.nojekyll` file to repo root (empty file, disables Jekyll)
- Place `controller.html` in repo root
- Enable GitHub Pages in repo settings (source: main branch, root directory)
- The controller will be available at `https://<user>.github.io/fgc-scoreboard/controller.html?bin=<id>`

## Technical Considerations

**Scores as strings:** The existing JSON schema stores scores as strings (`"0"` not `0`). The controller must explicitly use `String(value)` when constructing the JSON payload. The overlay's comparison logic (`$('#p1Score').text() != p1Score` at line 189) does string comparison — a numeric `0` would trigger unnecessary animation flicker on every poll.

**Swap atomicity:** The swap button modifies 6 fields locally, then saves the full JSON object in a single POST request. The overlay will detect all 6 changes on the next poll and trigger simultaneous fade-out/fade-in animations for both player wrappers and scores — this matches the current StreamControl swap behavior.

**Custom games:** When a custom game name is entered (not in the presets), the overlay's game-positioning logic falls through to the `else` default, which uses `adjust2` placement. This is the same behavior as the existing StreamControl `editable="1"` comboBox.

**Long sessions:** The overlay may run for 8-12 hours during a tournament. Remote polling via npoint has no rate limits. The `cBust` counter is a simple integer increment that won't overflow in any practical session.

**Update latency:** With 1-second polling, the effective update latency after the operator hits Save is `poll_interval + network_round_trip`, roughly 1-2 seconds. This is acceptable for tournament use.

## Known Limitations

- **No concurrent operator protection.** If two people open the controller with the same bin ID, the last save wins. A stale form could overwrite fresh data. Mitigation: only share the bin URL with one operator. This can be improved later with timestamp-based conflict detection if needed.
- **npoint.io has no SLA.** If it goes down, the overlay falls back gracefully (keeps showing last known data), but the controller cannot save. Mitigated by the local fallback mode and the fact that any JSON API endpoint can be substituted.

## Acceptance Criteria

- [x] `controller.html` — mobile-friendly form with player/score/round/game fields, save/swap/reset/clear buttons
- [x] Game field uses `<input>` + `<datalist>` with 12 presets (including SF6)
- [x] Controller reads `?bin=` URL param and uses it for npoint API calls
- [x] Controller loads current state from npoint on page load
- [x] Controller sets `timestamp` on every save
- [x] Controller shows save success/failure status indicator
- [x] `scoreboard.js` reads `?bin=` URL param and polls remote npoint endpoint
- [x] `scoreboard.js` falls back to local JSON file when no `?bin=` param is present
- [x] `scoreboard.js` has try-catch around JSON.parse (silent skip on error)
- [x] Polling interval is 1000ms (unconditional)
- [x] `.nojekyll` file added to repo root
- [ ] All existing overlay animations work unchanged when reading from remote source
- [ ] Score swapping works atomically (single POST, all fields)

## Dependencies & Risks

**Dependencies:**
- npoint.io service availability (free tier, no SLA)
- GitHub Pages for hosting the controller

**Risks:**
- **npoint.io goes down or shuts down:** Mitigated by backwards-compatible local fallback. Could swap to any JSON API endpoint without changing the overlay code.
- **Operator shares the bin URL publicly:** Anyone could write to it. Low impact (non-sensitive data), mitigable by creating a new bin.
- **Network latency causes stale data:** 1-2 second worst case. Acceptable for tournament use.

## Implementation Order

1. **Create npoint bin** — Set up a test bin with the scoreboard JSON schema
2. **Update `scoreboard.js`** — Add `?bin=` param reading, remote URL support, try-catch error handling, 1000ms interval
3. **Build `controller.html`** — Mobile-friendly form with save/swap/reset/clear, npoint POST integration
4. **Add `.nojekyll`** — Enable GitHub Pages serving
5. **Test end-to-end** — Controller on phone → npoint → overlay in browser

## References & Research

- Brainstorm: `docs/brainstorms/2026-03-13-remote-scoreboard-brainstorm.md`
- Current polling logic: `_overlays/js/scoreboard.js:18-26`
- JSON schema: `sc/streamcontrol.json`
- Game-specific positioning: `_overlays/js/scoreboard.js:44-67`
- npoint.io API: `https://www.npoint.io/docs`
- Review notes: DHH, Kieran, and Simplicity reviewers (2026-03-13)
