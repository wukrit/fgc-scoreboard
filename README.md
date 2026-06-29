# FGC Scoreboard

> Forked from [WASD-Gaming/fgc-scoreboard](https://github.com/WASD-Gaming/fgc-scoreboard) by [@tarikfayad](https://twitter.com/tarikfayad). Big thanks to Tarik for the original project — check out his work at [WASD Gaming](https://wasdgaming.gg).

FGC Scoreboard is an HTML and CSS scoreboard overlay for fighting game tournament streams. It uses no images for the scoreboard itself (only optional tournament logos) and no webm files for animations — everything is CSS keyframes and GreenSock (TweenMax).

The overlay uses a **neo-brutalist** look: P1 red (`#ff4444`), P2 blue (`#4488ff`), bold outlines, drop-shadows, and skewed player bars. The controller is a mobile-friendly operator form with a clean dark product UI.

## Quick Start

Pick the deployment that fits your event:

| Mode | Best for | Internet | Auth |
|------|----------|----------|------|
| **LAN** | In-person tournaments on the same network | Not required | Optional (`FGC_AUTH_TOKEN`) |
| **Hosted (Railway)** | Internet-accessible events with write protection | Required | Required (Bearer token) |
| **Tunnel** | Share your local server over the internet | Required | Optional |
| **Remote** | GitHub Pages + npoint.io, no server to run | Required | None (public bin) |
| **Local** | Quick testing in one browser | Not required | None |

**URLs (LAN / Hosted / Tunnel):**

- **Controller:** `https://<host>/`
- **Score overlay (OBS):** `https://<host>/overlay/scoreboard.html` — set browser source to **1920×1080**
- **Counters overlay (OBS, optional):** `https://<host>/overlay/counters.html` — same resolution; greyscale custom counters configured in the controller

> **Migration:** If you bookmarked the old overlay path `/_overlays/scoreboard.html`, update OBS to `/overlay/scoreboard.html`.

---

### Hosted Mode (Railway)

Best for internet-accessible deployments with write protection. Requires a [Railway](https://railway.com) account.

Infrastructure is defined in [`.railway/railway.ts`](.railway/railway.ts). See [deploy/railway.md](deploy/railway.md) for the full setup guide.

**Quick start:**

1. Deploy this repo from GitHub on Railway
2. Run `npm install && railway link && railway config apply` (creates the service)
3. Set `FGC_AUTH_TOKEN` in Railway Variables (generate with `fgc-server --generate-token` after building, or from a release binary)
4. Generate a Railway domain in **Settings → Networking**

**Operator workflow:**

| Share with operators | Keep secret |
|---------------------|-------------|
| Controller URL (`/`) | Bearer token |
| Score overlay URL (OBS) | — |
| Counters overlay URL (OBS, optional) | — |

Optional one-time link for operators: `/?token=YOUR_TOKEN` (token is stripped from the URL after unlock).

> **Note:** Scores reset when Railway redeploys. Overlay URLs are safe to share; keep the Bearer token secret.

---

### LAN Mode (No Internet Required)

Best for in-person tournaments. Download a [release archive](https://github.com/wukrit/fgc-scoreboard/releases) (includes `fgc-server` + `web/`) or build from source with Rust. See [Verifying downloads](#verifying-downloads) to check release integrity with GPG.

1. From the project directory (or unpacked release folder):
   ```
   ./scripts/start.sh
   ```
   Or after `cargo build --release --manifest-path server/Cargo.toml`:
   ```
   ./server/target/release/fgc-server --no-tunnel
   ```
   Or run `./fgc-server` from a release archive — if no internet is available, it falls back to LAN automatically after a brief retry.
2. The server auto-detects your machine's LAN IP and prints ready-to-use URLs:
   ```
   FGC Scoreboard Server
   Controller: http://<your-lan-ip>:8080/
   Overlay:    http://<your-lan-ip>:8080/overlay/scoreboard.html
   Counters:   http://<your-lan-ip>:8080/overlay/counters.html
   ```
3. Open the **Controller** URL on your phone or tablet.
4. In OBS, add a **Browser Source** pointing to the **Overlay** URL. Set the resolution to **1920×1080**. Optionally add a second browser source at `/overlay/counters.html` for custom counters.
5. Enter scores on the controller — score changes auto-save. Hit **Save** for name, round, game, swap, reset, or clear changes.

Custom port: `./scripts/start.sh --port 9090`

Optional auth (recommended for any internet-exposed LAN server):

```bash
export FGC_AUTH_TOKEN="$(./server/target/release/fgc-server --generate-token)"
FGC_AUTH_TOKEN="$FGC_AUTH_TOKEN" ./scripts/start.sh
```

Windows: unpack the release `.zip`, then run `.\fgc-server.exe` (or `.\fgc-server.exe --no-tunnel` to skip tunnel setup entirely).

---

### Cutting a release (maintainers)

Binary releases are published to [GitHub Releases](https://github.com/wukrit/fgc-scoreboard/releases) when a `v*` tag is pushed. [`.github/workflows/release.yml`](.github/workflows/release.yml) builds `fgc-server` for Linux (x64/ARM64), macOS (x64/ARM64), and Windows x64, packages each with `web/` and `data/`, and attaches SHA256 checksums (GPG-signed when repo secrets are configured — see [docs/signing.md](docs/signing.md)).

1. Ensure `main` is green (GitHub Pages + Server Build workflows).
2. Tag and push (version comes from the tag name):
   ```bash
   git tag -a v0.1.3 -m "v0.1.3"
   git push origin v0.1.3
   ```
3. Confirm the **Release** workflow completes and the GitHub Release lists all platform archives plus `SHA256SUMS.txt` and `SHA256SUMS.txt.asc`.

**First-time signing setup:** run `./scripts/setup-release-signing.sh` once (writes public key to `docs/release-signing.pub.asc`, private key to GitHub secrets only), commit the `.pub.asc` file, then tag.

Keep [server/Cargo.toml](server/Cargo.toml) `version` in sync with the tag manually when bumping versions.

---

### Tunnel Mode (Built-in localtunnel)

Best for sharing your local server over the internet with zero setup. The release binary starts a [localtunnel](https://github.com/localtunnel/localtunnel) public URL automatically — no extra installs.

> **Note:** The server has no authentication by default. Set `FGC_AUTH_TOKEN` to enable Bearer auth on writes. Anyone with your tunnel URL can read/overwrite the scoreboard if auth is disabled. Only share URLs with trusted operators.

**Running (release binary — tunnel on by default):**

```
./fgc-server
```

The server prints a public HTTPS URL on startup:

- **Controller:** `https://<subdomain>.loca.lt/`
- **Overlay:** `https://<subdomain>.loca.lt/overlay/scoreboard.html`
- **Counters (optional):** `https://<subdomain>.loca.lt/overlay/counters.html`

Custom port: `./fgc-server --port 9090`

Optional fixed subdomain (may not always be available):

```
./fgc-server --tunnel-subdomain my-event-name
```

LAN-only (disable tunnel): `./fgc-server --no-tunnel`

**Offline fallback:** If the tunnel server cannot be reached (no internet), the server automatically falls back to LAN-only mode after a few quick retries and prints local Controller/Overlay URLs.

**OBS browser reminder:** The public localtunnel service may show a one-time "Friendly Reminder" page in browsers (including OBS Browser Source). Visit the tunnel URL once in a normal browser on the streaming PC before adding the OBS source — it is suppressed for about 7 days per subdomain and IP. For events without that interstitial, self-host a [localtunnel server](https://github.com/localtunnel/server) and point `--tunnel-host` at it.

**From source (repo dev):** `./scripts/start.sh` disables tunnel by default. Run the binary directly for tunnel mode, or omit `--no-tunnel` when invoking `fgc-server` yourself.

Press Ctrl+C to stop the server and tunnel.

---

### Remote Mode (Internet Required)

Best for online tournaments or when the controller and streaming PC aren't on the same network. No server to run — host the static files on GitHub Pages (or any static host) and use [npoint.io](https://www.npoint.io/) for state.

> **Note:** npoint.io bins are public. Anyone who knows or guesses your bin ID can read or overwrite the scoreboard. For high-stakes tournaments, prefer LAN, Tunnel, or Hosted mode.

1. Create a free JSON bin at [npoint.io](https://www.npoint.io/) and copy the bin ID.
2. In OBS, add a **Browser Source** (1920×1080) pointing to:
   ```
   https://yourgithubpages.url/overlay/scoreboard.html?bin=YOUR_BIN_ID
   ```
   Optional counters overlay (same bin ID):
   ```
   https://yourgithubpages.url/overlay/counters.html?bin=YOUR_BIN_ID
   ```
3. Open the controller with the same bin ID:
   ```
   https://yourgithubpages.url/?bin=YOUR_BIN_ID
   ```
4. Enter scores on the controller — score changes auto-save. Hit **Save** for other field changes.

---

### Local Mode (Same Browser Only)

For quick testing without a server: open `web/index.html` and `web/overlay/scoreboard.html` as `file://` URLs in the **same browser**. The controller writes to `localStorage` and the overlay syncs via storage events. Add `web/overlay/counters.html` as a second tab for the optional counters overlay.

> **Limitation:** Does not work across devices (e.g. phone controller + OBS on another machine). Use LAN, Tunnel, Hosted, or Remote mode for that.

---

## The Controller

The controller is a mobile-friendly web form for updating:

- **Player names and teams**
- **Scores** — tap **+** / **−** (auto-saves) or type a score directly (0–999, saves on blur or Enter)
- **Round**
- **Game** — pick from 12 presets or toggle **Enter custom name** for any game
- **Additional Counters (optional)** — expand the collapsible section to add up to 8 custom label/value counters (0–999) for a separate OBS overlay

**Actions:**

| Button | Behavior |
|--------|----------|
| **Save** | Push all current fields to the scoreboard |
| **Swap** | Switch P1 and P2 names, teams, and scores (requires **Save**) |
| **Reset Scores** | Zero both scores (requires **Save**) |
| **Clear All** | Wipe all fields (requires **Save**) |

**Multi-controller sync:** In remote, LAN, and hosted modes, every open controller polls for updates every second. Changes from another operator appear automatically; fields you are actively editing are left alone.

**Auth (LAN / Hosted / Tunnel with `FGC_AUTH_TOKEN`):** The controller shows a token gate before writes. Enter the Bearer token once per session, or bootstrap via `?token=` in the URL. Use **Lock** to require the token again.

---

## Additional Counters Overlay

Use this when you need extra on-stream stats (stocks, timeouts, side bets, etc.) beyond the main P1/P2 scores.

1. In the controller, expand **Additional Counters (optional)** and click **Add Counter**.
2. Set a label (≤64 chars) and value (0–999). Stepper changes auto-save; label edits save on blur.
3. In OBS, add a second **Browser Source** at **1920×1080** pointing to `/overlay/counters.html` (same host and `?bin=` as the score overlay in remote mode).
4. Counters appear as greyscale cards along the bottom of the frame. Values animate on change.

**Notes:**

- Up to 8 counters; **Remove** deletes a row and auto-saves.
- **Clear All** resets player fields only — counters are preserved.
- Counter IDs are generated client-side; the server validates shape, label length, and numeric values 0–999.

---

## Supported Games

When you select a game and hit **Save**, the overlay adjusts its position so scores and logos don't cover important in-game gauges (HP bars, meters, etc.).

| Game | Layout |
|------|--------|
| BBTAG, SFVCE, TEKKEN7, UNICLR | Shifted down |
| BBCF, DBFZ, GGXRD, KOFXIV, MVCI, SF6, UMVC3 | Default |
| USF4 | Custom offset |

Custom game names work too — they use the default layout. BBTAG and UNICLR also get logo repositioning.

### Adding a New Game

1. Add the game to the `<select>` in `web/index.html`.
2. Add the game to the appropriate group in `GAME_GROUPS` at the top of `web/overlay/js/scoreboard.js`:
   ```javascript
   var GAME_GROUPS = {
       adjust1: ['BBTAG', 'SFVCE', 'TEKKEN7', 'UNICLR'],
       adjust2: ['BBCF', 'DBFZ', 'GGXRD', 'KOFXIV', 'MVCI', 'UMVC3'],
       adjust3: ['USF4'],
       logoAdjust: ['BBTAG', 'UNICLR']
   };
   ```
   Games not listed in any group default to `adjust2`.

---

## Customization

**Colors and styling:** Edit the SCSS variables at the top of `web/overlay/css/style.scss` and/or `web/overlay/css/counters.scss`, then compile:

```
sass web/overlay/css/style.scss web/overlay/css/style.css
sass web/overlay/css/counters.scss web/overlay/css/counters.css
```

Default accents: `$p1-accent: #ff4444`, `$p2-accent: #4488ff`. The score overlay auto-shrinks triple-digit scores (`fitScoreDisplay()` in `scoreboard.js`) so values up to 999 stay inside the score boxes.

**Animation timing:** Edit the inline `<script>` variables in `web/overlay/scoreboard.html` (timing, offsets, distances).

**Logos:** Add `<img>` tags with `class="logos"` inside the `#logoWrapper` div in `web/overlay/scoreboard.html`. Multiple logos rotate automatically.

```html
<div id="logoWrapper">
    <img id="logo1" class="logos" src="imgs/your-logo.png">
</div>
```

**OBS Browser Source:** Set the resolution to **1920×1080** with no custom CSS. The overlay background is transparent.

---

## How It Works

Three sync modes, auto-detected by priority:

1. **Remote** (`?bin=` parameter present) — Controller POSTs to npoint.io; controller and overlay poll every 1s.
2. **LAN / Hosted** (served over `http:` or `https:` without `?bin=`) — Controller POSTs to `/scoreboard.json` on the same origin; controller and overlay poll every 1s. When `FGC_AUTH_TOKEN` is set, POSTs require `Authorization: Bearer <token>`; reads stay public.
3. **Local** (`file://` protocol) — Controller writes to `localStorage`; overlay syncs via browser storage events.

Core fields are strings in JSON (`p1Name`, `p1Team`, `p1Score`, `p2Name`, `p2Team`, `p2Score`, `round`, `game`, `timestamp`). Scores and counter values must be numeric strings from `"0"` to `"999"` — the server rejects four-digit values and non-numeric input on POST. The controller sets `timestamp` on every save so multiple controllers can detect changes.

**Optional `counters` object** (max 8 entries): each key is a counter ID (≤32 chars); each value is `{ "label": string (≤64), "value": string "0"–"999" }`. Syncs through the same `/scoreboard.json` payload and displays on `/overlay/counters.html`. See [Additional Counters Overlay](#additional-counters-overlay).

**Hosted (Railway):** See [deploy/railway.md](deploy/railway.md). Infrastructure as code in `.railway/railway.ts`. POST rate limiting defaults to 60 requests/minute per IP (`FGC_RATE_LIMIT`).

---

## Verifying downloads

Releases include `SHA256SUMS.txt` covering every platform archive. When GPG signing is enabled, `SHA256SUMS.txt.asc` is also attached.

1. Import the maintainer **public** key (safe to store in this public repo):

   ```bash
   gpg --import docs/release-signing.pub.asc
   ```

2. Verify the checksum signature (optional if no `.asc` file yet):

   ```bash
   gpg --verify SHA256SUMS.txt.asc SHA256SUMS.txt
   ```

3. Confirm archive hashes from the directory where you downloaded the files:

   ```bash
   sha256sum -c SHA256SUMS.txt          # Linux
   shasum -a 256 -c SHA256SUMS.txt      # macOS
   ```

   On Windows, compare `Get-FileHash` output to the matching line in `SHA256SUMS.txt`.

GPG proves the checksum file is authentic. It does **not** remove OS warnings when running unsigned binaries:

- **macOS:** Right-click `fgc-server` → Open, or run `xattr -d com.apple.quarantine ./fgc-server` after verifying.
- **Windows:** Click "More info" → Run anyway after verifying.

Full maintainer setup: [docs/signing.md](docs/signing.md).

---

## Security

| Mode | Read access | Write access |
|------|-------------|--------------|
| LAN / Tunnel (default) | Anyone on the network/URL | Anyone on the network/URL |
| LAN / Tunnel + `FGC_AUTH_TOKEN` | Public | Bearer token required |
| Hosted (Railway) | Public overlay + scores | Bearer token required |
| Remote (npoint.io) | Public (anyone with bin ID) | Public (anyone with bin ID) |
| Local | Same browser only | Same browser only |

For high-stakes events: use Hosted mode with a strong token, or LAN/Tunnel with `FGC_AUTH_TOKEN`. Do not share bin IDs, tunnel URLs, or Bearer tokens publicly.

Generate a token: `fgc-server --generate-token` (build or download the binary first)

---

## Contact

If you found this useful or have suggestions, feel free to reach out! Find me on Twitch at [wukrit](https://www.twitch.tv/wukrit).

---

## Screenshots

<p align="center">
  <img src="screenshots/sf6.png" alt="FGC Scoreboard on SF6." width="75%">
</p>

---

## License

Usage is provided under the [MIT License](https://opensource.org/licenses/MIT). See LICENSE for the full details.
