# Backend

Actix Web crate (`jheffmedia-site-backend`).

**Full reference:** [docs/backend.md](../docs/backend.md) · API: [docs/api.md](../docs/api.md) · auth: [docs/authentication.md](../docs/authentication.md)

## Modes

- **Dev:** `--features forward-frontend` — proxies UI to webpack
- **Prod:** default — serves static files from `YEW_FULLSTACK_STATIC`

## Quick commands

```bash
openssl rand -out src/secret.key 32   # first-time, from backend/

cargo build --features forward-frontend
cargo watch -x "run --features forward-frontend"
```

## Environment variables

| Variable | Default |
|----------|---------|
| `YEW_FULLSTACK_HOST` | `127.0.0.1` |
| `YEW_FULLSTACK_PORT` | `3000` (8080 in Docker) |
| `YEW_FULLSTACK_STATIC` | `/usr/local/share/yew-fullstack/www` |
| `YEW_FULLSTACK_FORWARD_FRONTEND_URL` | `http://localhost:8080` (dev proxy target) |
| `YEW_FULLSTACK_DB_CONNSTR` | `mongodb://localhost:27017` |
| `YEW_FULLSTACK_DB_NAME` | `yew-fullstack` |

See [docs/backend.md](../docs/backend.md) for module layout and MongoDB notes.
