# Brainstorm: Local Fallback When No npoint.io Bin Provided

**Date:** 2026-03-13

## What We're Building

A local/offline mode for the FGC Scoreboard that uses `localStorage` instead of npoint.io when no `?bin=` URL parameter is provided. Both the overlay (`scoreboard.js`) and the controller (`controller.html`) will auto-detect which mode to use based on the presence of the `?bin=` parameter.

## Why This Approach

- **Offline dev/testing:** Preview and tweak the overlay without internet or an npoint.io account.
- **LAN tournament fallback:** At venues with unreliable internet, the scoreboard still works.
- **Zero config:** No extra parameters, servers, or setup. Just open the files without `?bin=` and it works locally.

## Key Decisions

1. **Auto-detect from URL param:** If `?bin=<id>` is present → npoint.io remote mode (existing behavior). If absent → localStorage mode. No explicit mode switch needed.

2. **localStorage as the data store:** The controller writes to a known localStorage key. The overlay reads from the same key. No local server or manual file editing required.

3. **`storage` event for real-time sync:** The overlay listens for the browser's `storage` event to get instant updates when the controller saves, rather than polling localStorage on a timer. This gives near-instant responsiveness without wasted reads.

4. **Same origin requirement:** Both the controller and overlay must be opened from the same origin (e.g., both as `file://` paths, or both served from `localhost`) for localStorage sharing and the `storage` event to work.

5. **Minimal scope (Approach A):** No seed files, no export/import buttons, no new UI elements. Just the dual-mode data layer. Can be extended later if needed.

## Affected Files

- **`_overlays/js/scoreboard.js`** — Add localStorage read path + `storage` event listener when no `?bin=` param.
- **`controller.html`** — Add localStorage write path when no `?bin=` param. Show indicator of which mode is active.

6. **Console log indicator:** In local mode, log `LOCAL MODE` to the browser console (both overlay and controller). No visible on-screen badge — the user knows based on whether they provided `?bin=`.

7. **localStorage key:** `fgc-scoreboard-data` — descriptive and collision-safe.

8. **Form restore on load:** In local mode, the controller reads `fgc-scoreboard-data` from localStorage on page load and pre-populates the form, mirroring the remote mode behavior of fetching from npoint.io.

## Resolved Questions

- **Visual indicator?** Console log only. No on-screen badge.
- **localStorage key?** `fgc-scoreboard-data`.
- **Form pre-population?** Yes, restore from localStorage on load (symmetric with remote mode).
