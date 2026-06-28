# Deploying on Railway

Hosted mode runs `fgc-server` (Rust) on [Railway](https://railway.com) with HTTPS, Bearer token auth, and infrastructure defined in `.railway/railway.ts`.

## Architecture

- **Score overlay** (`/overlay/scoreboard.html`) — public read via `GET /scoreboard.json`
- **Counters overlay** (`/overlay/counters.html`, optional) — same JSON payload; greyscale custom counters from the controller
- **Controller** (`/`) — token gate; writes require `Authorization: Bearer <token>`
- **Scores** — stored in `data/scoreboard.json` on the container filesystem; **reset on redeploy**

POST validation: core fields are strings; `p1Score`, `p2Score`, and counter values must be numeric strings `"0"`–`"999"`.

## First-time setup

### 1. Create the Railway project

1. [Railway dashboard](https://railway.com) → **New Project** → **Deploy from GitHub repo**
2. Select this repository

### 2. Link locally and install IaC tooling

```bash
npm install
railway login
railway link
```

### 3. Apply infrastructure (creates the service)

```bash
railway config plan
railway config apply
```

This creates/updates the service with:

- GitHub source: `wukrit/fgc-scoreboard` (main branch)
- Builder: Dockerfile (multi-stage Rust build)
- Start command: `/app/fgc-server --no-tunnel` (binary path inside the container; tunnel disabled for hosted deploys)
- Healthcheck: `GET /health`
- `FGC_RATE_LIMIT=60`

`FGC_AUTH_TOKEN` is **not** stored in IaC — it uses `preserve()` so apply keeps the
existing Railway value. Set the token on the service after first apply (step 4).

### 4. Set the auth token (secret)

Generate a token locally (after building `fgc-server`):

```bash
cargo build --release --manifest-path server/Cargo.toml
./server/target/release/fgc-server --generate-token
```

Set it on the service (do not commit):

```bash
railway variables set FGC_AUTH_TOKEN="YOUR_TOKEN_HERE" -s fgc-scoreboard
```

### 5. Connect GitHub (if not already)

Confirm the `fgc-scoreboard` service is linked to the GitHub repo.

### 6. Generate a public domain

In the Railway dashboard: **Settings → Networking → Generate Domain**.

### 7. Verify

```bash
DOMAIN=https://your-app.up.railway.app

curl "$DOMAIN/health"
curl "$DOMAIN/scoreboard.json"

curl -sf "$DOMAIN/overlay/counters.html" | head -c 20 | grep -qi '<html' && echo "counters overlay OK"

curl -X POST "$DOMAIN/scoreboard.json" \
  -H "Authorization: Bearer $FGC_AUTH_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"p1Name":"Test","p1Team":"","p1Score":"999","p2Name":"","p2Team":"","p2Score":"0","round":"","game":"SF6","timestamp":"1234567890","counters":{"c1":{"label":"Stock","value":"100"}}}'
```

Open `$DOMAIN/`, enter the token, and save scores. Point OBS at `$DOMAIN/overlay/scoreboard.html`. Optional counters: `$DOMAIN/overlay/counters.html`.

## Operator workflow

| Share with operators | Keep secret |
|---------------------|-------------|
| Controller URL (`/`) | Bearer token |
| Score overlay URL (OBS) | — |
| Counters overlay URL (OBS, optional) | — |

Optional one-time QR/link: `/?token=...` (token is stripped from the URL after load).

## Logging

Structured JSON logs on Railway when `RAILWAY_ENVIRONMENT` is set. POST saves at `info`; poll traffic is `debug` by default.

| Variable | Default | Purpose |
|----------|---------|---------|
| `FGC_LOG_LEVEL` | `INFO` | Root log level |
| `FGC_LOG_POLL` | off | Set to `1` to log poll GETs at INFO |
| `FGC_LOG_JSON` | auto | `1` or `0` to force format |

## Local auth testing

```bash
export FGC_AUTH_TOKEN="$(./server/target/release/fgc-server --generate-token)"
FGC_AUTH_TOKEN="$FGC_AUTH_TOKEN" ./scripts/start.sh
```

Open `http://localhost:8080/` — the token gate appears when auth is enabled.

## Troubleshooting

| Issue | Fix |
|-------|-----|
| Deploy fails after Python → Rust migration | Ensure start command is `/app/fgc-server --no-tunnel` (not `python3 server.py` or a host path like `./server/target/release/fgc-server`). Run `railway config plan` — IaC must match Dockerfile. |
| Healthcheck fails | Ensure `GET /health` returns 200; check deploy logs |
| 401 on save | Token mismatch; re-enter on controller or update Railway Variable |
| Debug overlay/controller sync | Set `FGC_LOG_POLL=1` on the service |
| Scores gone after deploy | Expected — scores are ephemeral without a Volume |
