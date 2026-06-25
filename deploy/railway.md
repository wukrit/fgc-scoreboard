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

### 3. Set the auth token (secret)

Generate a token locally:

```bash
python3 server.py --generate-token
```

Set it in Railway (do not commit):

```bash
railway variables set FGC_AUTH_TOKEN="YOUR_TOKEN_HERE"
```

Or use the dashboard **Variables** tab. Use a [sealed variable](https://docs.railway.com/variables) for production.

### 4. Apply infrastructure

```bash
railway config plan
railway config apply
```

This creates/updates the service with:

- Start command: `python3 server.py`
- Healthcheck: `GET /health`
- `FGC_RATE_LIMIT=60`
- `FGC_AUTH_TOKEN` preserved from step 3

### 5. Generate a public domain

In the Railway dashboard: **Settings → Networking → Generate Domain**.

Optionally add a custom domain (requires CNAME + TXT records). To track domains in code, add to `.railway/railway.ts`:

```typescript
domains: ["scoreboard.example.com"],
```

Then run `railway config apply`.

### 6. Verify

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
| IaC plan blocked | Remove `railway.toml` if present — cannot mix with `.railway/railway.ts` |
| Scores gone after deploy | Expected — scores are ephemeral on Railway without a Volume |
