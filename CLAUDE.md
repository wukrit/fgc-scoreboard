# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

FGC Scoreboard is a pure HTML/CSS/JS scoreboard overlay for fighting game tournament streams. It uses no images for the scoreboard itself (only optional tournament logos) and no webm animations — all animations are CSS keyframes and GreenSock (TweenMax).

## Architecture

### Data Flow

Three sync modes, auto-detected by priority:

1. **npoint.io (remote)** — `?bin=<id>` URL parameter present → controller POSTs to npoint.io, overlay polls it every 1s
2. **LAN server** — Page served over `http:` without `?bin=` → controller POSTs to own origin, overlay polls own origin every 1s
3. **localStorage** — `file://` protocol, no `?bin=` → controller writes localStorage, overlay syncs via storage event

### Key Files

- **`_overlays/scoreboard.html`** — Main overlay loaded as a browser source in OBS/streaming software. Contains animation config variables (timing, offsets, distances) as inline `<script>` vars. Requires `?bin=<npoint_id>` URL parameter.
- **`_overlays/js/scoreboard.js`** — Core logic: polls npoint.io JSON endpoint, parses data, handles game-specific layout adjustments, animates score/name/round changes with TweenMax, manages logo rotation.
- **`_overlays/css/style.scss`** — SCSS source with customizable variables at the top (`$main-color`, `$accent-color`, `$font-color`, `$team-color`). Must be compiled to `style.css`.
- **`_overlays/css/style.css`** — Compiled CSS (what the overlay actually loads).
- **`controller.html`** — Mobile-friendly web form for score entry. Hosted on GitHub Pages (remote mode) or served by `server.py` (LAN mode).
- **`server.py`** — Zero-dependency Python 3 HTTP server for LAN tournaments. Serves static files + JSON endpoint. Run with `python3 server.py [--port PORT]`.
- **`_overlays/scoreboard.xml`** — OBS/StreamControl-compatible source config file.
- **`docs/`** — Project documentation: brainstorms, implementation plans, and solution docs (e.g., the StreamControl-to-npoint.io migration).

### Important Implementation Details

- **Scores are sent as strings** from the controller (`String(input.value)`). The overlay compares scores as text to detect changes and trigger animations. Sending numbers instead of strings will break change detection.
- **Remote mode requires a `?bin=<npoint_id>` URL parameter** on both `controller.html` and `scoreboard.html`. Create a free bin at https://www.npoint.io/.
- **LAN mode requires no URL parameters** — just serve via `python3 server.py` and open the printed URLs.

### Supported Games

BBCF, BBTAG, DBFZ, GGXRD, KOFXIV, MVCI, SF6, SFVCE, TEKKEN7, UMVC3, UNICLR, USF4. Games not listed in an adjustment group default to `adjust2`.

### Game-Specific Layout

The scoreboard repositions itself vertically based on the selected game (to avoid covering HP bars). Three position groups exist in `scoreboard.js`:
1. **Shift down** (`adjust1`): BBTAG, SFVCE, TEKKEN7, UNICLR
2. **Shift text up** (`adjust2`, default): BBCF, DBFZ, GGXRD, KOFXIV, MVCI, UMVC3
3. **Custom offset** (`adjust3`): USF4

### Dependencies (vendored, no package manager)

- Pico CSS v2.1.1 classless variant (`css/pico.classless.min.css`) — used by controller.html only
- jQuery 3.3.1 (`_overlays/js/jquery-3.3.1.min.js`)
- GreenSock/TweenMax (`_overlays/js/greensock-js/`)
- Valorant font (`_overlays/fonts/ValorantFont.ttf`)

## Development

No build system or package manager. To compile SCSS → CSS, use any Sass compiler:

```
sass _overlays/css/style.scss _overlays/css/style.css
```

To test remote mode, open `_overlays/scoreboard.html?bin=<npoint_id>` in a browser. Use the controller or edit the npoint.io bin directly to simulate input.

### LAN Mode (Tournament Use)

For LAN tournaments without internet:

```
python3 server.py
```

This prints controller and overlay URLs with the LAN IP. Open the controller on a phone and point OBS to the overlay URL. No `?bin=` parameter needed — mode is auto-detected.

## Customization

All visual customization is done through SCSS variables at the top of `_overlays/css/style.scss` — colors, font, opacity. Animation timing/positioning is configured via inline JS variables in `scoreboard.html`.
