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

Never commit the token value. For an **existing** service, IaC uses `preserve()` so
`railway config apply` keeps the token Railway already holds:

```typescript
FGC_AUTH_TOKEN: preserve(),
```

On **first-time** service create, omit `FGC_AUTH_TOKEN` from IaC (Railway rejects
`preserve()` when the variable does not exist yet). Apply infra, then set the token:

```bash
railway variables set FGC_AUTH_TOKEN="$(./server/target/release/fgc-server --generate-token)" -s fgc-scoreboard
```

After the token exists, add `preserve()` to `.railway/railway.ts` so future applies
do not delete it.

The IaC file uses `module.exports = project(...)` (not `export default`) because tsx
double-wraps ESM default exports and breaks `railway config plan`.

## Deploy code vs deploy infra

- **Git push** (GitHub integration) — deploys application code
- **`railway config apply`** — updates service settings, env vars, domains

Do **not** add `railway.toml` — it conflicts with IaC for the same service.

See [deploy/railway.md](../deploy/railway.md) for the full setup guide.
