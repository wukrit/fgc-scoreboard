# FGC Scoreboard

> Forked from [WASD-Gaming/fgc-scoreboard](https://github.com/WASD-Gaming/fgc-scoreboard) by [@tarikfayad](https://twitter.com/tarikfayad). Big thanks to Tarik for the original project — check out his work at [WASD Gaming](https://wasdgaming.gg).

FGC Scoreboard is an HTML and CSS scoreboard overlay for fighting game tournament streams. It uses no images for the scoreboard itself (only optional tournament logos) and no webm files for animations — everything is CSS keyframes and GreenSock (TweenMax).

The overlay and controller use a **neo-brutalist** look: P1 red (`#ff4444`), P2 blue (`#4488ff`), bold outlines, drop-shadows, and skewed player bars. The controller is a mobile-friendly web form built on [Pico CSS](https://picocss.com/).

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

- **Controller:** `https://<host>/` (also works at `/controller.html`)
- **Overlay (OBS):** `https://<host>/_overlays/scoreboard.html` — set browser source to **1920×1080**

---

### Hosted Mode (Railway)

Best for internet-accessible deployments with write protection. Requires a [Railway](https://railway.com) account.

Infrastructure is defined in [`.railway/railway.ts`](.railway/railway.ts). See [deploy/railway.md](deploy/railway.md) for the full setup guide.

**Quick start:**

1. Deploy this repo from GitHub on Railway
2. Run `npm install && railway link && railway config apply` (creates the service)
3. Set `FGC_AUTH_TOKEN` in Railway Variables (generate with `python3 server.py --generate-token`)
4. Generate a Railway domain in **Settings → Networking**

**Operator workflow:**

| Share with operators | Keep secret |
|---------------------|-------------|
| Controller URL (`/`) | Bearer token |
| Overlay URL (OBS) | — |

Optional one-time link for operators: `/?token=YOUR_TOKEN` (token is stripped from the URL after unlock).

> **Note:** Scores reset when Railway redeploys. The overlay URL is safe to share; keep the Bearer token secret.

---

### LAN Mode (No Internet Required)

Best for in-person tournaments. Requires Python 3.

1. Run the server from the project directory:
   ```
   python3 server.py
   ```
2. The server auto-detects your machine's LAN IP and prints ready-to-use URLs:
   ```
   FGC Scoreboard Server
   Controller: http://<your-lan-ip>:8080/
   Overlay:    http://<your-lan-ip>:8080/_overlays/scoreboard.html
   ```
3. Open the **Controller** URL on your phone or tablet.
4. In OBS, add a **Browser Source** pointing to the **Overlay** URL. Set the resolution to **1920×1080**.
5. Enter scores on the controller — score changes auto-save. Hit **Save** for name, round, game, swap, reset, or clear changes.

Custom port: `python3 server.py --port 9090`

Optional auth (recommended for any internet-exposed LAN server):

```bash
export FGC_AUTH_TOKEN="$(python3 server.py --generate-token)"
FGC_AUTH_TOKEN="$FGC_AUTH_TOKEN" python3 server.py
```

---

### Tunnel Mode (Cloudflare Tunnel)

Best for sharing your local server over the internet with a stable URL. Requires [cloudflared](https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/get-started/).

> **Note:** The LAN server has no authentication by default. Set `FGC_AUTH_TOKEN` to enable Bearer auth on writes. Anyone with your tunnel URL can read/overwrite the scoreboard if auth is disabled. Only share URLs with trusted operators.

**One-time setup:**

1. Install cloudflared:
   ```
   brew install cloudflared
   ```
2. Login to your Cloudflare account:
   ```
   cloudflared tunnel login
   ```
3. Create a named tunnel:
   ```
   cloudflared tunnel create fgc-scoreboard
   ```
4. Create a config file at `~/.cloudflared/config.yml`:
   ```yaml
   tunnel: fgc-scoreboard
   credentials-file: /path/to/.cloudflared/<TUNNEL_ID>.json

   ingress:
     - hostname: fgc.yourdomain.com
       service: http://localhost:8080
     - service: http_status:404
   ```
   Replace `<TUNNEL_ID>` with the ID printed from step 3, and `fgc.yourdomain.com` with your subdomain.
5. Create the DNS record:
   ```
   cloudflared tunnel route dns fgc-scoreboard fgc.yourdomain.com
   ```

**Running:**

```
./start-tunnel.sh
```

This starts both `server.py` and the Cloudflare Tunnel. Your URLs will be:

- **Controller:** `https://fgc.yourdomain.com/`
- **Overlay:** `https://fgc.yourdomain.com/_overlays/scoreboard.html`

Custom port: `./start-tunnel.sh --port 9090`

Press Ctrl+C to stop both.

---

### Remote Mode (Internet Required)

Best for online tournaments or when the controller and streaming PC aren't on the same network. No server to run — host the static files on GitHub Pages (or any static host) and use [npoint.io](https://www.npoint.io/) for state.

> **Note:** npoint.io bins are public. Anyone who knows or guesses your bin ID can read or overwrite the scoreboard. For high-stakes tournaments, prefer LAN, Tunnel, or Hosted mode.

1. Create a free JSON bin at [npoint.io](https://www.npoint.io/) and copy the bin ID.
2. In OBS, add a **Browser Source** (1920×1080) pointing to:
   ```
   https://yourgithubpages.url/_overlays/scoreboard.html?bin=YOUR_BIN_ID
   ```
3. Open the controller with the same bin ID:
   ```
   https://yourgithubpages.url/controller.html?bin=YOUR_BIN_ID
   ```
4. Enter scores on the controller — score changes auto-save. Hit **Save** for other field changes.

---

### Local Mode (Same Browser Only)

For quick testing without a server: open both `controller.html` and `_overlays/scoreboard.html` as `file://` URLs in the **same browser**. The controller writes to `localStorage` and the overlay syncs via storage events.

> **Limitation:** Does not work across devices (e.g. phone controller + OBS on another machine). Use LAN, Tunnel, Hosted, or Remote mode for that.

---

## The Controller

The controller is a mobile-friendly web form for updating:

- **Player names and teams**
- **Scores** — tap **+** / **−** (auto-saves) or type a score directly (0–99, saves on blur or Enter)
- **Round**
- **Game** — pick from 12 presets or toggle **Enter custom name** for any game

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

## Supported Games

When you select a game and hit **Save**, the overlay adjusts its position so scores and logos don't cover important in-game gauges (HP bars, meters, etc.).

| Game | Layout |
|------|--------|
| BBTAG, SFVCE, TEKKEN7, UNICLR | Shifted down |
| BBCF, DBFZ, GGXRD, KOFXIV, MVCI, SF6, UMVC3 | Default |
| USF4 | Custom offset |

Custom game names work too — they use the default layout. BBTAG and UNICLR also get logo repositioning.

### Adding a New Game

1. Add the game to the `<select>` in `controller.html`.
2. Add the game to the appropriate group in `GAME_GROUPS` at the top of `_overlays/js/scoreboard.js`:
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

**Colors and styling:** Edit the SCSS variables at the top of `_overlays/css/style.scss`, then compile:

```
sass _overlays/css/style.scss _overlays/css/style.css
```

Default accents: `$p1-accent: #ff4444`, `$p2-accent: #4488ff`.

**Animation timing:** Edit the inline `<script>` variables in `_overlays/scoreboard.html` (timing, offsets, distances).

**Logos:** Add `<img>` tags with `class="logos"` inside the `#logoWrapper` div in `_overlays/scoreboard.html`. Multiple logos rotate automatically.

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

All scoreboard fields are strings in JSON (`p1Name`, `p1Team`, `p1Score`, `p2Name`, `p2Team`, `p2Score`, `round`, `game`, `timestamp`). The controller sets `timestamp` on every save so multiple controllers can detect changes.

**Hosted (Railway):** See [deploy/railway.md](deploy/railway.md). Infrastructure as code in `.railway/railway.ts`. POST rate limiting defaults to 60 requests/minute per IP (`FGC_RATE_LIMIT`).

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

Generate a token: `python3 server.py --generate-token`

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
