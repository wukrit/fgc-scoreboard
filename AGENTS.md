# AGENTS.md

This file provides guidance to AI coding agents when working with code in this repository.

## Project Overview

FGC Scoreboard is a pure HTML/CSS/JS scoreboard overlay for fighting game tournament streams. It uses no images for the scoreboard itself (only optional tournament logos) and no webm animations — all animations are CSS keyframes and GreenSock (TweenMax).

The overlay and controller received a **neo-brutalist visual reskin** (Mar 2026): P1 red (`#ff4444`), P2 blue (`#4488ff`), 3px outlines, drop-shadows, and skewed player bars.

For end-user setup (LAN, tunnel, remote, customization, security), see [README.md](README.md).

## Architecture

### Data Flow

Three sync modes, auto-detected by priority:

1. **npoint.io (remote)** — `?bin=<id>` URL parameter present → controller POSTs to npoint.io and polls it every 1s; overlay polls it every 1s
2. **LAN / hosted server** — Page served over `http:` or `https:` without `?bin=` → controller POSTs to own origin `/scoreboard.json` and polls it every 1s; overlay polls it every 1s. Hosted (Railway) uses the same client code path as LAN; optional Bearer auth when `FGC_AUTH_TOKEN` is set on the server
3. **localStorage** — `file://` protocol, no `?bin=` → controller writes localStorage and listens for `storage` events; overlay syncs via storage event

**Auth (server-side):** When `FGC_AUTH_TOKEN` env var is set (≥32 chars), `POST /scoreboard.json` and `GET /auth/check` require `Authorization: Bearer <token>`. `GET /scoreboard.json`, overlay assets, `/`, and `/health` stay public. Controller shows a token gate when the server sends `X-FGC-Auth-Required: 1`.

**JSON schema** (all values are strings):

```
p1Name, p1Team, p1Score, p2Name, p2Team, p2Score, round, game, timestamp
```

The controller sets `timestamp` on every save. The Rust server (`fgc-server`) validates allowed keys and string values (rejects unknown keys, non-strings, values >128 chars).

**Local mode limitation:** Only syncs across tabs in the **same browser** via `storage` events. Does **not** work for phone controller + OBS overlay — use LAN or remote for that.

**Public URLs (LAN / hosted / tunnel / GitHub Pages):**

| Page | URL |
|------|-----|
| Controller | `/` (`web/index.html`) |
| Overlay (OBS) | `/overlay/scoreboard.html` |
| Score API | `/scoreboard.json` |

**Deployment paths** (not separate sync modes):

| Path | How |
|------|-----|
| LAN | `./scripts/start.sh` or release `fgc-server` binary — no URL params; auth optional via `FGC_AUTH_TOKEN` |
| Hosted (Railway) | GitHub deploy + `.railway/railway.ts` + `railway config apply`; see [deploy/railway.md](deploy/railway.md). Scores ephemeral on redeploy |
| Remote | GitHub Pages (deploys `web/` via Actions) + `?bin=<id>` on controller and overlay |
| Tunnel | `./scripts/start-tunnel.sh [--port PORT]` — runs `fgc-server` + `cloudflared tunnel` |
| Local/file | Open `web/index.html` and `web/overlay/scoreboard.html` as `file://` in the same browser |

LAN and tunnel have **no authentication by default** (set `FGC_AUTH_TOKEN` to enable). Hosted Railway deployments should always set `FGC_AUTH_TOKEN`. npoint.io bins are public. Do not share bin IDs, tunnel URLs, or Bearer tokens publicly for high-stakes events.

### Key Files

- **`README.md`** — End-user setup (LAN, tunnel, remote, customization, security)
- **`web/index.html`** — Controller (neo-brutalist mobile form). Pico CSS + custom overrides. Token gate + `authFetch()` for LAN/hosted auth.
- **`web/css/pico.classless.min.css`** — Pico CSS v2.1.1 classless variant (controller only)
- **`web/overlay/scoreboard.html`** — Main OBS overlay (1920×1080). Animation config as inline `<script>` vars. `?bin=` required only in **remote** mode.
- **`web/overlay/js/scoreboard.js`** — Core overlay logic: mode setup, polling, game layout, TweenMax animations, logo rotation, `shrinkToFit()`
- **`web/overlay/css/style.scss`** — SCSS source; compile to `style.css`
- **`web/overlay/css/style.css`** — Compiled overlay CSS
- **`server/`** — Rust Axum HTTP server (`fgc-server` binary). API routes + static file serving from `web/`. Env: `PORT`, `FGC_BIND`, `FGC_AUTH_TOKEN`, `FGC_ASSET_ROOT` (default `web`), `FGC_DATA_DIR` (default `data`), `FGC_RATE_LIMIT`, `FGC_LOG_*`. Run: `cargo run --manifest-path server/Cargo.toml` or `./scripts/start.sh`
- **`scripts/start.sh`** — Start server (release binary or `cargo run`)
- **`scripts/start-tunnel.sh`** — Cloudflare Tunnel + server
- **`scripts/server-parity-test.sh`** — API/static smoke tests
- **`.railway/railway.ts`** — Railway IaC (service, healthcheck, env vars)
- **`package.json`** — `railway` SDK devDependency for IaC only
- **`deploy/railway.md`** — Railway deployment guide
- **`data/scoreboard.json`** — Runtime LAN/hosted state (gitignored; created on startup)
- **`docs/`** — Project documentation (see [docs/](#docs) below)

### Important Implementation Details

- **Scores are sent as strings** from the controller (`String(input.value)`), clamped 0–99. The overlay compares scores as text to detect changes and trigger animations. Sending numbers instead of strings will break change detection.
- **Remote mode requires `?bin=<npoint_id>`** on both controller (`/`) and overlay (`/overlay/scoreboard.html`).
- **LAN mode requires no URL parameters** — run `./scripts/start.sh` and open the printed URLs.
- **Bearer auth** — when enabled, controller stores token in `sessionStorage` (`fgc-auth-token`); attach to same-origin fetch only. Bootstrap via `?token=` once (stripped from URL). Overlay unchanged (public GET).
- **Score steppers auto-save** (`adjustScore()` calls `save()`); **Swap / Reset / Clear do not** — user must hit Save to push those changes.
- **Multi-controller sync** — remote/LAN controllers poll every 1s (same interval as overlay). Incoming updates merge field-by-field and skip focused inputs; `timestamp` detects changes. Local mode uses `storage` events for cross-tab sync only.
- **Overlay renders user data with jQuery `.text()`** (not `.html()`) to prevent XSS.
- **`GAME_GROUPS` in `scoreboard.js`** — data-driven game layout lookup; add new games to the appropriate array.
- **1920×1080** — overlay is fixed to this resolution (`body` in SCSS); OBS browser source should match.

### Supported Games

12 presets in the controller datalist: BBCF, BBTAG, DBFZ, GGXRD, KOFXIV, MVCI, SF6, SFVCE, TEKKEN7, UMVC3, UNICLR, USF4. The game field is free-text — custom games work and default to `adjust2`.

### Game-Specific Layout

Groups are defined in `GAME_GROUPS` at the top of `web/overlay/js/scoreboard.js`:

| Layout group | Games | Y behavior |
|--------------|-------|------------|
| **adjust1** (shift down +32px) | BBTAG, SFVCE, TEKKEN7, UNICLR | BG wrappers shift down; text wrappers +4px |
| **adjust2** (default, text up -28px) | BBCF, DBFZ, GGXRD, KOFXIV, MVCI, UMVC3 + **SF6** + any unlisted game | Text wrappers shift up |
| **adjust3** (custom +28px) | USF4 | Custom offset for wrappers and BG |
| **logoAdjust** (logo position/scale via `adjustLg`) | BBTAG, UNICLR | Logo repositioned/scaled |

**Adding a new game:** Add to the datalist in `web/index.html` and to the appropriate group in `GAME_GROUPS` in `scoreboard.js`.

Inline config in `web/overlay/scoreboard.html`: `adjust1`, `adjust2`, `adjust3`, `adjustLg` arrays.

### scoreboard.js Internals

- `shrinkToFit()` — auto-shrinks long player/round names (max 2px reduction)
- `createPoller()` — unified fetch poller with AbortController for remote + LAN
- `applyGameLayout()` / `playCSSAnimations()` — game change triggers fade-out, reposition, replay CSS keyframes, fade-in
- `currentGame` JS variable replaces removed `#gameHold` div

Animation config vars remain inline in `web/overlay/scoreboard.html`.

### Dependencies (vendored, no package manager)

- Pico CSS v2.1.1 (`web/css/pico.classless.min.css`) — controller only
- jQuery 3.3.1 (`web/overlay/js/jquery-3.3.1.min.js`)
- GreenSock/TweenMax — only `TweenMax.min.js` loaded at runtime
- Archivo Black font (`web/overlay/fonts/ArchivoBlack-Regular.ttf`)

**External runtime deps (not vendored):** Rust toolchain to build `fgc-server`, optional `cloudflared` (`scripts/start-tunnel.sh`), npoint.io (remote mode).

**IaC / deploy tooling:** Node.js + Railway CLI for `.railway/railway.ts` (`npm install`, `railway config apply`).

## Development

To compile SCSS → CSS:

```
sass web/overlay/css/style.scss web/overlay/css/style.css
```

Both `style.scss` and `style.css` are committed.

**Remote mode testing:** `web/overlay/scoreboard.html?bin=<id>` and `/?bin=<id>` on hosted Pages or local server.

**Local mode testing:** Open `web/index.html` and `web/overlay/scoreboard.html` as `file://` in the same browser.

**GitHub Pages:** `.github/workflows/pages.yml` deploys `web/`; `web/.nojekyll` disables Jekyll.

### LAN Mode (Tournament Use)

```bash
./scripts/start.sh
# or: cargo run --release --manifest-path server/Cargo.toml
```

Custom port: `./scripts/start.sh --port 9090`

**Auth testing:**

```bash
export FGC_AUTH_TOKEN="$(cargo run --quiet --manifest-path server/Cargo.toml -- --generate-token)"
FGC_AUTH_TOKEN="$FGC_AUTH_TOKEN" ./scripts/start.sh
```

### Hosted Mode (Railway)

See [deploy/railway.md](deploy/railway.md). Apply infra with `railway config apply` after editing `.railway/railway.ts`.

### Tunnel Mode

```bash
./scripts/start-tunnel.sh [--port PORT]
```

Requires prior `cloudflared` setup — see README.

## Customization

**Colors and styling:** Edit SCSS variables at the top of `web/overlay/css/style.scss`, then compile to `style.css`.

**Animation timing/positioning:** Edit inline `<script>` variables in `web/overlay/scoreboard.html`.

**Logos:** Add `<img class="logos">` tags inside `#logoWrapper` in `web/overlay/scoreboard.html`. Sample assets in `web/overlay/imgs/`.

**Controller styling:** Neo-brutalist overrides in `web/index.html` inline `<style>`.

## docs/

```
docs/
├── brainstorms/
├── plans/
└── solutions/
```
