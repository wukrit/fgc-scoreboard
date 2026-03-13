# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

FGC Scoreboard is a pure HTML/CSS/JS scoreboard overlay for fighting game tournament streams. It uses no images for the scoreboard itself (only optional tournament logos) and no webm animations — all animations are CSS keyframes and GreenSock (TweenMax).

## Architecture

### Data Flow

Remote controller (`controller.html`) → writes JSON to npoint.io → `_overlays/js/scoreboard.js` polls this JSON every 1s via AJAX → updates the HTML overlay with animated transitions.

### Key Files

- **`_overlays/scoreboard.html`** — Main overlay loaded as a browser source in OBS/streaming software. Contains animation config variables (timing, offsets, distances) as inline `<script>` vars. Requires `?bin=<npoint_id>` URL parameter.
- **`_overlays/js/scoreboard.js`** — Core logic: polls npoint.io JSON endpoint, parses data, handles game-specific layout adjustments, animates score/name/round changes with TweenMax, manages logo rotation.
- **`_overlays/css/style.scss`** — SCSS source with customizable variables at the top (`$main-color`, `$accent-color`, `$font-color`, `$team-color`). Must be compiled to `style.css`.
- **`_overlays/css/style.css`** — Compiled CSS (what the overlay actually loads).
- **`controller.html`** — Mobile-friendly web form for score entry. Hosted on GitHub Pages. Reads/writes to npoint.io JSON bin.

### Game-Specific Layout

The scoreboard repositions itself vertically based on the selected game (to avoid covering HP bars). Three position groups exist in `scoreboard.js`:
1. **Shift down** (`adjust1`): BBTAG, SFVCE, TEKKEN7, UNICLR
2. **Shift text up** (`adjust2`, default): BBCF, DBFZ, GGXRD, KOFXIV, MVCI, UMVC3
3. **Custom offset** (`adjust3`): USF4

### Dependencies (vendored, no package manager)

- jQuery 3.3.1 (`_overlays/js/jquery-3.3.1.min.js`)
- GreenSock/TweenMax (`_overlays/js/greensock-js/`)
- Valorant font (`_overlays/fonts/ValorantFont.ttf`)

## Development

No build system or package manager. To compile SCSS → CSS, use any Sass compiler:

```
sass _overlays/css/style.scss _overlays/css/style.css
```

To test, open `_overlays/scoreboard.html?bin=<npoint_id>` in a browser. Use the controller or edit the npoint.io bin directly to simulate input.

## Customization

All visual customization is done through SCSS variables at the top of `_overlays/css/style.scss` — colors, font, opacity. Animation timing/positioning is configured via inline JS variables in `scoreboard.html`.
