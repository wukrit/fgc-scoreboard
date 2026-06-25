# AGENTS.md

This file provides guidance to AI coding agents when working with code in this repository.

## Project Overview

FGC Scoreboard is a pure HTML/CSS/JS scoreboard overlay for fighting game tournament streams. It uses no images for the scoreboard itself (only optional tournament logos) and no webm animations — all animations are CSS keyframes and GreenSock (TweenMax).

The overlay and controller received a **neo-brutalist visual reskin** (Mar 2026): P1 red (`#ff4444`), P2 blue (`#4488ff`), 3px outlines, drop-shadows, and skewed player bars.

For end-user setup (LAN, tunnel, remote, customization, security), see [README.md](README.md).

## Architecture

### Data Flow

Three sync modes, auto-detected by priority:

1. **npoint.io (remote)** — `?bin=<id>` URL parameter present → controller POSTs to npoint.io, overlay polls it every 1s
2. **LAN server** — Page served over `http:` or `https:` without `?bin=` → controller POSTs to own origin `/scoreboard.json`, overlay polls it every 1s
3. **localStorage** — `file://` protocol, no `?bin=` → controller writes localStorage, overlay syncs via storage event

**JSON schema** (all values are strings):

```
p1Name, p1Team, p1Score, p2Name, p2Team, p2Score, round, game, timestamp
```

The controller sets `timestamp` on every save. The LAN server (`server.py`) validates allowed keys and string values (rejects unknown keys, non-strings, values >128 chars).

**Local mode limitation:** Only syncs across tabs in the **same browser** via `storage` events. Does **not** work for phone controller + OBS overlay — use LAN or remote for that.

**Deployment paths** (not separate sync modes):

| Path | How |
|------|-----|
| LAN | `python3 server.py` — no URL params needed |
| Remote | GitHub Pages + `?bin=<id>` on controller and overlay |
| Tunnel | `./start-tunnel.sh [--port PORT]` — runs `server.py` + `cloudflared tunnel` for internet-accessible LAN (uses LAN sync mode over `https:`) |
| Local/file | Open both pages as `file://` in the same browser |

Tunnel and LAN endpoints have **no authentication**. npoint.io bins are also public. Do not share bin IDs or tunnel URLs publicly for high-stakes events.

### Key Files

- **`README.md`** — End-user setup (LAN, tunnel, remote, customization, security)
- **`controller.html`** — Mobile-friendly neo-brutalist web form for score entry. Pico CSS + custom overrides. Hosted on GitHub Pages (remote mode) or served by `server.py` (LAN mode).
- **`server.py`** — Zero-dependency Python 3 HTTP server for LAN tournaments. Serves static files + GET/POST `/scoreboard.json`. Run with `python3 server.py [--port PORT]`. Creates `scoreboard.json` on startup if missing.
- **`start-tunnel.sh`** — Cloudflare Tunnel launcher (runs `server.py` + `cloudflared tunnel`)
- **`scoreboard.json`** — Runtime LAN state file (gitignored; created on server startup if missing)
- **`_overlays/scoreboard.html`** — Main overlay loaded as a browser source in OBS/streaming software (1920×1080). Contains animation config variables (timing, offsets, distances) as inline `<script>` vars. `?bin=` required only in **remote** mode; LAN/local work without it.
- **`_overlays/js/scoreboard.js`** — Core logic: mode setup, polling, game-specific layout, TweenMax animations, logo rotation, `shrinkToFit()` for long names.
- **`_overlays/css/style.scss`** — SCSS source with customizable variables at the top (`$main-color`, `$p1-accent`, `$p2-accent`, `$font-color`, `$team-color`). Must be compiled to `style.css`.
- **`_overlays/css/style.css`** — Compiled CSS (what the overlay actually loads).
- **`css/pico.classless.min.css`** — Pico CSS v2.1.1 classless variant — controller only (root `css/`, not `_overlays/css/`)
- **`_overlays/scoreboard.xml`** — Legacy StreamControl plugin stub (minimal metadata; does not configure bin URL or paths)
- **`_overlays/imgs/`** — Sample tournament logos (not wired by default; `#logoWrapper` in scoreboard.html is empty)
- **`docs/`** — Project documentation (see [docs/](#docs) below)

### Important Implementation Details

- **Scores are sent as strings** from the controller (`String(input.value)`), clamped 0–99. The overlay compares scores as text to detect changes and trigger animations. Sending numbers instead of strings will break change detection.
- **Remote mode requires a `?bin=<npoint_id>` URL parameter** on both `controller.html` and `scoreboard.html`. Create a free bin at https://www.npoint.io/.
- **LAN mode requires no URL parameters** — just serve via `python3 server.py` and open the printed URLs.
- **Score steppers auto-save** (`adjustScore()` calls `save()`); **Swap / Reset / Clear do not** — user must hit Save to push those changes.
- **Overlay renders user data with jQuery `.text()`** (not `.html()`) to prevent XSS.
- **`GAME_GROUPS` in `scoreboard.js`** — data-driven game layout lookup; add new games to the appropriate array.
- **1920×1080** — overlay is fixed to this resolution (`body` in SCSS); OBS browser source should match.

### Supported Games

12 presets in the controller datalist: BBCF, BBTAG, DBFZ, GGXRD, KOFXIV, MVCI, SF6, SFVCE, TEKKEN7, UMVC3, UNICLR, USF4. The game field is free-text — custom games work and default to `adjust2`.

### Game-Specific Layout

The scoreboard repositions itself vertically based on the selected game (to avoid covering HP bars). Groups are defined in `GAME_GROUPS` at the top of `scoreboard.js`:

| Layout group | Games | Y behavior |
|--------------|-------|------------|
| **adjust1** (shift down +32px) | BBTAG, SFVCE, TEKKEN7, UNICLR | BG wrappers shift down; text wrappers +4px |
| **adjust2** (default, text up -28px) | BBCF, DBFZ, GGXRD, KOFXIV, MVCI, UMVC3 + **SF6** + any unlisted game | Text wrappers shift up |
| **adjust3** (custom +28px) | USF4 | Custom offset for wrappers and BG |
| **logoAdjust** (logo position/scale via `adjustLg`) | BBTAG, UNICLR | Logo repositioned/scaled |

SF6 is in the controller datalist but not explicitly listed in `GAME_GROUPS.adjust2` — it falls through to the default in `getGameGroup()`.

**Adding a new game:** Add to the datalist in `controller.html` and to the appropriate group in `GAME_GROUPS` in `scoreboard.js`. Games not listed in any group default to `adjust2`.

Inline config in `scoreboard.html`: `adjust1`, `adjust2`, `adjust3`, `adjustLg` arrays.

### scoreboard.js Internals

Key helpers (recent refactors):

- `shrinkToFit()` — auto-shrinks long player/round names (max 2px reduction)
- `createPoller()` — unified fetch poller with AbortController for remote + LAN
- `applyGameLayout()` / `playCSSAnimations()` — game change triggers fade-out, reposition, replay CSS keyframes, fade-in
- `currentGame` JS variable replaces removed `#gameHold` div (dead CSS for `#gameHold` may remain in style.scss)

Animation config vars remain inline in `scoreboard.html` (`nameSize`, `adjust1/2/3`, `adjustLg`, timing vars).

### Dependencies (vendored, no package manager)

- Pico CSS v2.1.1 classless variant (`css/pico.classless.min.css`) — used by controller.html only
- jQuery 3.3.1 (`_overlays/js/jquery-3.3.1.min.js`)
- GreenSock/TweenMax — only `TweenMax.min.js` is loaded at runtime (`_overlays/js/greensock-js/src/minified/TweenMax.min.js`); full source tree is vendored but unused
- Archivo Black font (`_overlays/fonts/ArchivoBlack-Regular.ttf`) — used by overlay
- Valorant font (`_overlays/fonts/ValorantFont.ttf`) — legacy, unused

**External runtime deps (not vendored):** Python 3 (`server.py`), optional `cloudflared` (`start-tunnel.sh`), npoint.io (remote mode).

## Development

No build system or package manager. To compile SCSS → CSS, use any Sass compiler:

```
sass _overlays/css/style.scss _overlays/css/style.css
```

Both `style.scss` and `style.css` are committed.

**Remote mode testing:** Open `_overlays/scoreboard.html?bin=<npoint_id>` and `controller.html?bin=<npoint_id>`. Use the controller or edit the npoint.io bin directly to simulate input.

**Local mode testing:** Open both pages as `file://` URLs in the same browser.

**GitHub Pages:** `.nojekyll` at repo root disables Jekyll; remote URLs use `?bin=` on both controller and overlay.

### LAN Mode (Tournament Use)

For LAN tournaments without internet:

```
python3 server.py
```

This prints controller and overlay URLs with the LAN IP. Open the controller on a phone and point OBS to the overlay URL. No `?bin=` parameter needed — mode is auto-detected.

Custom port: `python3 server.py --port 9090`

### Tunnel Mode

```
./start-tunnel.sh [--port PORT]
```

Requires prior `cloudflared` setup — see README. No authentication on the tunnel endpoint.

## Customization

**Colors and styling:** Edit SCSS variables at the top of `_overlays/css/style.scss`:

```scss
$main-color: rgba(0, 0, 0, 0.85);
$p1-accent: #ff4444;
$p2-accent: #4488ff;
$font-color: white;
$team-color: #e5e5e5;
```

Then compile to `style.css`.

**Animation timing/positioning:** Edit inline `<script>` variables in `_overlays/scoreboard.html`.

**Logos:** Add `<img class="logos">` tags inside `#logoWrapper` in `scoreboard.html`. Multiple logos rotate automatically. Sample assets in `_overlays/imgs/`. Note: `.logos` has `display: none` until JS fade-in runs.

**Controller styling:** Neo-brutalist overrides on top of Pico CSS in `controller.html` inline `<style>`.

## docs/

```
docs/
├── brainstorms/   # Design/requirements explorations (Mar 2026) — 6 files
├── plans/         # Implementation plans — 6 files
└── solutions/     # Post-implementation writeups — 2 files
    ├── integration-issues/streamcontrol-to-npoint-remote-controller.md
    └── ui-redesign/controller-neo-brutalist-pico-css-redesign.md
```
