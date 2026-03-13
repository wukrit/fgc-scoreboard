# FGC Scoreboard
FGC Scoreboard is an HTML and CSS scoreboard overlay for fighting game tournament streams. It uses no images for the scoreboard itself (only optional tournament logos) and no webm files for animations — everything is CSS keyframes and GreenSock (TweenMax).

## Quick Start

There are two ways to use FGC Scoreboard: **LAN mode** (recommended for tournaments) and **Remote mode** (for online setups).

### LAN Mode (No Internet Required)

Best for in-person tournaments. Requires Python 3.

1. Run the server from the project directory:
   ```
   python3 server.py
   ```
2. The server prints your LAN URLs:
   ```
   FGC Scoreboard Server
   Controller: http://192.168.1.5:8080/controller.html
   Overlay:    http://192.168.1.5:8080/_overlays/scoreboard.html
   ```
3. Open the **Controller** URL on your phone or tablet.
4. In OBS, add a **Browser Source** pointing to the **Overlay** URL.
5. Enter scores on the controller, hit **Save**, and the overlay updates live.

To use a different port: `python3 server.py --port 9090`

### Tunnel Mode (Cloudflare Tunnel)

Best for sharing your local server over the internet with a stable URL. Requires [cloudflared](https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/get-started/).

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
- **Controller:** `https://fgc.yourdomain.com/controller.html`
- **Overlay:** `https://fgc.yourdomain.com/_overlays/scoreboard.html`

Press Ctrl+C to stop both.

### Remote Mode (Internet Required)

Best for online tournaments or when the controller and streaming PC aren't on the same network.

1. Create a free JSON bin at [npoint.io](https://www.npoint.io/) and copy the bin ID.
2. In OBS, add a **Browser Source** pointing to:
   ```
   https://yourgithubpages.url/_overlays/scoreboard.html?bin=YOUR_BIN_ID
   ```
3. Open the controller with the same bin ID:
   ```
   https://yourgithubpages.url/controller.html?bin=YOUR_BIN_ID
   ```
4. Enter scores on the controller, hit **Save**, and the overlay polls for updates every second.

## The Controller

The controller is a mobile-friendly web form for updating:
- Player Names
- Team Names
- Scores
- Round
- Game

It also has **Swap** (switch player sides), **Reset** (zero out scores), and **Clear** (wipe all fields) buttons.

## Supported Games

When you select a game and hit save, the overlay adjusts its position so scores and logos don't cover important in-game gauges (HP bars, meters, etc.).

| Game | Layout |
|------|--------|
| BBTAG, SFVCE, TEKKEN7, UNICLR | Shifted down |
| BBCF, DBFZ, GGXRD, KOFXIV, MVCI, SF6, UMVC3 | Default |
| USF4 | Custom offset |

## Customization

**Colors and styling:** Edit the SCSS variables at the top of `_overlays/css/style.scss`, then compile:
```
sass _overlays/css/style.scss _overlays/css/style.css
```

**Animation timing:** Edit the inline `<script>` variables in `_overlays/scoreboard.html` (timing, offsets, distances).

**Logos:** Drop tournament/company logo images into the logos section of `scoreboard.html`. Multiple logos rotate automatically.

## How It Works

The scoreboard has three sync modes, auto-detected by priority:

1. **Remote** (`?bin=` parameter present) — Controller POSTs to npoint.io, overlay polls it every 1s.
2. **LAN** (served over `http:` without `?bin=`) — Controller POSTs to the local server, overlay polls it every 1s.
3. **Local** (`file://` protocol) — Controller writes to localStorage, overlay syncs via browser storage events. Only works within the same browser.

## Drop Me a Line
If you found this at all useful, or have some suggestions, please let me know! You can drop me a line on twitter ([@tarikfayad](https://twitter.com/tarikfayad)), find me on Twitch ([ImpurestClub](https://www.twitch.tv/impurestclub/)), or ping me on my Discord server ([Link](https://discord.gg/ykj8tsN)).

Also, feel free to check out a much bigger project I've been working on called **WASD**. It's a search engine for teammates/sparring partners along with a pretty comprehensive tournament calendar. You can find it here: https://wasdgaming.gg

If you've really found this useful, feel free to <br>
<a href="https://www.buymeacoffee.com/tarik" target="_blank"><img src="https://www.buymeacoffee.com/assets/img/custom_images/orange_img.png" alt="Buy Me A Coffee" style="height: 41px !important;width: 174px !important;box-shadow: 0px 3px 2px 0px rgba(190, 190, 190, 0.5) !important;-webkit-box-shadow: 0px 3px 2px 0px rgba(190, 190, 190, 0.5) !important;" ></a>

## Thank Yous
Massive thank you to TheSassageKing for making the initial [4 hour YouTube tutorial this is based on](https://www.youtube.com/watch?v=qqyFknxaVWo). The general look of the board and a good chunk of the code's skeleton comes from following that video.

Also, thank you to [u/Brylark](https://www.reddit.com/r/VALORANT/comments/g0747t/valorant_font/) over on Reddit for making the VALORANT font that I've included in the repo and plan to use myself for the time being.

## Screenshots
<p align="center">
  <img src="screenshots/dbfz.png" alt="FGC Scoreboard on DBFZ." width="75%">
  <img src="screenshots/uniclr.png" alt="FGC Scoreboard on UNICLR." width="75%">
</p>

## License
Usage is provided under the [MIT License](http://http//opensource.org/licenses/mit-license.php). See LICENSE for the full details.
