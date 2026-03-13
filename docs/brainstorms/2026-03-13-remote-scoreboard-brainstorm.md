# Remote Scoreboard Data

**Date:** 2026-03-13
**Status:** Ready for planning

## What We're Building

Replace the local StreamControl.exe + local JSON file workflow with a remote setup:

1. **A mobile-friendly web form** hosted on GitHub Pages that replaces StreamControl as the data entry interface. Supports all current fields: player names, teams, scores, round, game, commentary, and misc text.
2. **A remote JSON store** (using a free JSON bin service like JSONBin.io or npoint.io) that holds the scoreboard state.
3. **Update the overlay** (`scoreboard.js`) to poll the remote JSON URL instead of the local `../sc/streamcontrol.json` path at 1-second intervals.

### Use Case

A score operator on a separate machine (or phone at a venue) enters match data via the web form. The OBS machine running the overlay reads the remote JSON and displays animated score updates — same as today, just over the internet instead of the local filesystem.

## Why This Approach

- **JSON Bin service** was chosen over GitHub API (propagation delay), self-hosted server (hosting overhead), and Dropbox (auth complexity).
- Minimal changes to the existing overlay code — only the XHR URL and polling interval change.
- Free tier services are sufficient for 1-second polling.
- No backend to build, deploy, or maintain.
- The web form replaces StreamControl entirely, removing the Windows-only dependency.

## Key Decisions

1. **Storage:** Free JSON bin service (e.g., JSONBin.io, npoint.io) — publicly readable, API-key-protected writes.
2. **Input:** Mobile-friendly web form hosted on **GitHub Pages** — replaces StreamControl.exe.
3. **Game dropdown:** Editable combo — includes the current 11 games as presets but allows free-text entry for any game.
4. **Autocomplete:** Not needed — plain text inputs for player names and rounds.
5. **API key handling:** Passed as a **URL parameter** (e.g., `?key=abc123`). Operator bookmarks the URL with key included.
6. **Polling interval:** Increased from 500ms to **1 second** for remote polling.
7. **Overlay changes:** Minimal — swap XHR URL to remote endpoint, adjust interval to 1s.

## Components to Build

### Web Form (`_overlays/controller.html` or similar)
- Player 1: name, team, score (0-99)
- Player 2: name, team, score (0-99)
- Round: text input
- Game: dropdown with presets + free text
- Swap button (swap P1/P2 names, teams, scores)
- Reset button (reset scores to 0)
- Clear button (clear all fields)
- Commentary tab: cTitle1, cTitle2
- Misc tabs: mText1-4
- Auto-saves to JSON bin on every change (or on a "Save" button)
- Reads API key from URL parameter

### Overlay Update (`_overlays/js/scoreboard.js`)
- Change `streamJSON` from `'../sc/streamcontrol.json'` to remote bin URL
- Change `setInterval` from 500ms to 1000ms
- Handle CORS (JSON bin services typically support this)
