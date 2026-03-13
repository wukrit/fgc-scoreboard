---
title: Migrate from StreamControl.exe to Remote Web Controller with npoint.io
date: 2026-03-13
category: integration-issues
tags:
  - remote-workflow
  - github-pages
  - json-storage
  - mobile-control
  - npoint
  - streamcontrol
module: scoreboard
symptoms:
  - Operator must be physically at OBS machine
  - Windows-only dependency (StreamControl.exe)
  - No mobile or cross-platform score entry
severity: medium
resolution_time_estimate: 4-6 hours
---

# Migrate from StreamControl.exe to Remote Web Controller

## Problem

StreamControl.exe imposed three constraints:
1. **Platform lock-in** — Windows-only application
2. **Physical proximity** — operator must be at the OBS machine (or on shared local filesystem)
3. **No venue mobility** — no way to update scores from a phone while walking a tournament floor

## Solution

Replace StreamControl with a three-tier remote system:
1. **Web form** (`controller.html`) hosted on GitHub Pages for data entry from any device
2. **Remote JSON store** (npoint.io) as the shared state layer
3. **Overlay polling** (`scoreboard.js`) reads from npoint.io via `?bin=` URL parameter

## Key Decisions

### Why npoint.io over JSONBin.io

JSONBin.io's free tier allows 10,000 requests/day. At 1 poll/second, the overlay alone consumes 28,800 reads in an 8-hour tournament — exhausting the limit in ~3 hours. npoint.io has **no rate limits on reads**.

**Always calculate:** `polls_per_second x seconds_in_tournament = min_daily_reads`

Tradeoff: npoint.io has no write authentication. Mitigated by random endpoint URLs and non-sensitive data.

### Explicit Save vs Auto-save

Chose explicit Save button because:
- Prevents partial data appearing on stream while operator is mid-edit
- No debounce complexity
- Operator controls exact update timing
- Mobile-friendly single action

### POST not PATCH for writes

Always send the full JSON object via POST (never PATCH). Keeps the controller as single source of truth with no merge semantics ambiguity.

## Gotchas

### Scores must be strings

The overlay compares text node content via `$('#p1Score').text() != p1Score`. Numeric `0` vs string `"0"` causes false inequality, triggering animation flicker every poll cycle. Controller must explicitly call `String()` on score values.

### GitHub Pages ignores `_` directories

Jekyll ignores directories starting with `_` (like `_overlays/`). The controller must live in the repo root. Add a `.nojekyll` file to disable Jekyll processing entirely.

### Cache busting on remote URLs

Keep the `?v=cBust` query parameter even for npoint URLs. It's ignored by the server and avoids branching logic between local/remote paths.

### Silent error handling on overlay

Never render error states on the overlay — it's visible to stream viewers. On fetch/parse failure, silently skip the update cycle and retry on next poll.

### Swap button atomicity

The swap modifies 6 fields locally, then sends a single POST. The overlay detects all changes on the next poll and triggers simultaneous animations for both players.

## Known Limitations

- **No concurrent operator protection** — last save wins. Only share bin URL with one operator.
- **npoint.io has no SLA** — if it goes down, overlay keeps showing last known data. Any JSON API endpoint can be substituted.
- **Update latency** — poll interval + network round-trip = ~1-2 seconds. Acceptable for tournament use.

## Prevention / Future Improvements

- Timestamp-based conflict detection for multi-operator scenarios
- Service health check in controller with fallback endpoint support

## References

- Brainstorm: `docs/brainstorms/2026-03-13-remote-scoreboard-brainstorm.md`
- Plan: `docs/plans/2026-03-13-feat-remote-scoreboard-controller-plan.md`
- Implementation commit: `c021413`
- Removal of StreamControl: `b8bc770`
