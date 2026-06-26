# Deploying on Railway

Hosted mode runs `server.py` on [Railway](https://railway.com) with HTTPS, Bearer token auth, and infrastructure defined in `.railway/railway.ts`.

## Architecture

- **Overlay** (`/_overlays/scoreboard.html`) — public read via `GET /scoreboard.json`
- **Controller** (`/controller.html`) — token gate; writes require `Authorization: Bearer <token>`
- **Scores** — stored in `scoreboard.json` on the container filesystem; **reset on redeploy**

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

Run this **before** setting variables — `railway variables set` fails with
"Project has no services" until a service exists.

```bash
railway config plan
railway config apply
```

This creates/updates the service with:

- GitHub source: `wukrit/fgc-scoreboard` (main branch)
- Start command: `python3 server.py`
- Healthcheck: `GET /health`
- `FGC_RATE_LIMIT=60`

`FGC_AUTH_TOKEN` is **not** in IaC — set it on the service after apply (step 4).
Using `preserve()` in IaC breaks `railway config apply` on first create.

### 4. Set the auth token (secret)

Generate a token locally:

```bash
python3 server.py --generate-token
```

Set it on the service (do not commit):

```bash
railway variables set FGC_AUTH_TOKEN="YOUR_TOKEN_HERE" -s fgc-scoreboard
```

Or use the dashboard **Variables** tab on the `fgc-scoreboard` service. Use a
[sealed variable](https://docs.railway.com/variables) for production.

### 5. Connect GitHub (if not already)

In the Railway dashboard, confirm the `fgc-scoreboard` service is linked to the
GitHub repo. Pushes to `main` deploy application code; `railway config apply`
manages service settings.

### 6. Generate a public domain

In the Railway dashboard: **Settings → Networking → Generate Domain**.

Optionally add a custom domain (requires CNAME + TXT records). To track domains in code, add to `.railway/railway.ts`:

```typescript
domains: ["scoreboard.example.com"],
```

Then run `railway config apply`.

### 7. Verify

```bash
DOMAIN=https://your-app.up.railway.app

curl "$DOMAIN/health"
curl "$DOMAIN/scoreboard.json"

curl -X POST "$DOMAIN/scoreboard.json" \
  -H "Authorization: Bearer $FGC_AUTH_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"p1Name":"Test","p1Team":"","p1Score":"1","p2Name":"","p2Team":"","p2Score":"0","round":"","game":"SF6","timestamp":"1234567890"}'
```

Open `$DOMAIN/controller.html`, enter the token, and save scores. Point OBS at `$DOMAIN/_overlays/scoreboard.html`.

## Operator workflow

| Share with operators | Keep secret |
|---------------------|-------------|
| Controller URL | Bearer token |
| Overlay URL (OBS) | — |

Optional one-time QR/link: `controller.html?token=...` (token is stripped from the URL after load).

Rotate the token between events: update `FGC_AUTH_TOKEN` in Railway Variables and redeploy.

## Logging

The server emits structured JSON logs on Railway (`RAILWAY_ENVIRONMENT` is set automatically). POST saves log at `info`; poll traffic (`GET /scoreboard.json`, `GET /health`) is `debug` by default to keep logs quiet.

| Variable | Default | Purpose |
|----------|---------|---------|
| `FGC_LOG_LEVEL` | `INFO` | Root log level (`DEBUG`, `INFO`, …) |
| `FGC_LOG_POLL` | off | Set to `1` to log poll GETs at INFO (sync debugging) |
| `FGC_LOG_JSON` | auto | `1` or `0` to force JSON or plain text |

In the Railway log explorer, `@level:error` reflects real failures (5xx, startup errors), not routine POST 200s.

## Local auth testing

```bash
export FGC_AUTH_TOKEN="$(python3 server.py --generate-token)"
echo "Token: $FGC_AUTH_TOKEN"
FGC_AUTH_TOKEN="$FGC_AUTH_TOKEN" python3 server.py
```

Open `http://localhost:8080/controller.html` — the token gate appears when auth is enabled.

## CI drift check (optional)

```bash
railway config plan --detailed-exit-code
```

Exit `0` = no pending infra changes; exit `2` = drift detected.

## Troubleshooting

| Issue | Fix |
|-------|-----|
| Healthcheck fails | Ensure `GET /health` returns 200; check deploy logs |
| 401 on save | Token mismatch; re-enter on controller or update Railway Variable |
| POST 200s show as errors | Fixed in current `server.py` — logs go to stdout with structured JSON on Railway; redeploy if on an older build |
| Debug overlay/controller sync | Set `FGC_LOG_POLL=1` on the service to log `GET /scoreboard.json` and `GET /health` at INFO; remove when done |
| IaC plan blocked | Remove `railway.toml` if present — cannot mix with `.railway/railway.ts` |
| Scores gone after deploy | Expected — scores are ephemeral on Railway without a Volume |
