# Rust Server Rewrite (implemented)

Replaced `server.py` with `fgc-server` (Axum + tower-http). See branch `feat/rust-server`.

## Layout

- `server/` — Rust crate, binary `fgc-server`
- `web/` — static assets (controller at `index.html`, overlay under `overlay/`)
- `data/` — runtime `scoreboard.json`
- `scripts/` — `start.sh`, `start-tunnel.sh`, `server-parity-test.sh`

## Public URLs

| Page | Path |
|------|------|
| Controller | `/` |
| Overlay | `/overlay/scoreboard.html` |
| API | `/scoreboard.json`, `/health`, `/auth/check` |

## API parity

Matches Python server for all API routes, auth, rate limiting, schema validation, and atomic writes.

## Build

```bash
cargo build --release --manifest-path server/Cargo.toml
./server/target/release/fgc-server --generate-token
./scripts/start.sh
```
