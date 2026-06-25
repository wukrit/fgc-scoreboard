# Railway Infrastructure as Code

This project uses [Railway Infrastructure as Code](https://docs.railway.com/infrastructure-as-code) (`.railway/railway.ts`).

## Prerequisites

- [Railway CLI](https://docs.railway.com/develop/cli) v5.2.0+
- Node.js (for the `railway` SDK devDependency in the repo root)

```bash
npm install
railway login
railway link
```

## Workflow

```bash
# Preview changes (safe — read-only)
railway config plan

# Apply after review
railway config apply

# Import current dashboard state into railway.ts
railway config pull
```

## Secrets

Never commit `FGC_AUTH_TOKEN`. Set it once:

```bash
railway variables set FGC_AUTH_TOKEN="$(python3 server.py --generate-token)"
```

The IaC file uses `preserve()` so `railway config apply` does not overwrite an existing token.

## Deploy code vs deploy infra

- **Git push** (GitHub integration) — deploys application code
- **`railway config apply`** — updates service settings, env vars, domains

Do **not** add `railway.toml` — it conflicts with IaC for the same service.

See [deploy/railway.md](../deploy/railway.md) for the full setup guide.
