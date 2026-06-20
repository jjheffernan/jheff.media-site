# Backend

Actix Web server crate: `jheffmedia-site-backend` in `backend/`.

See also: [architecture.md](./architecture.md), [authentication.md](./authentication.md), [api.md](./api.md), [development.md](./development.md).

## Modes of operation

Controlled by Cargo feature `forward-frontend`:

| Feature | Frontend handling | Build flag |
|---------|-------------------|------------|
| `forward-frontend` | Reverse proxy to Trunk dev server | `--features forward-frontend` |
| (default) | Serve files from `YEW_FULLSTACK_STATIC` | no feature |

Dev Docker and local watch use `forward-frontend`. Production binary serves baked static assets.

## Environment variables

| Variable | Default | Purpose |
|----------|---------|---------|
| `YEW_FULLSTACK_HOST` | `127.0.0.1` | Bind address |
| `YEW_FULLSTACK_PORT` | `3000` (Docker: `8080`) | Listen port |
| `YEW_FULLSTACK_STATIC` | `/usr/local/share/yew-fullstack/www` | Static root (prod) |
| `YEW_FULLSTACK_FORWARD_FRONTEND_URL` | `http://localhost:8080` | Upstream webpack (dev) |
| `YEW_FULLSTACK_DB_CONNSTR` | `mongodb://localhost:27017` | MongoDB connection string |
| `YEW_FULLSTACK_DB_NAME` | `yew-fullstack` | Database name |
| `YEW_FULLSTACK_FEED_PROVIDER` | `none` | `immich`, `static`, or `none` |
| `YEW_FULLSTACK_IMMICH_URL` | — | Immich base URL (no trailing slash) |
| `YEW_FULLSTACK_IMMICH_API_KEY` | — | Immich API key (server-side only) |
| `YEW_FULLSTACK_IMMICH_ALBUM_ID` | — | Album UUID for homepage feed |
| `YEW_FULLSTACK_IMMICH_SHARED_KEY` | — | Optional shared-link key for thumbnails |
| `YEW_FULLSTACK_FEED_STATIC_JSON` | — | JSON array of `FeedItem` for `static` provider |

Dev values are set in `scripts/docker-compose.dev.yml`.

`RUST_LOG` controls `env_logger` verbosity (e.g. `debug` in `backend/scripts/run-dev.sh`).

## Secrets

`backend/src/secret.key` — 32-byte JWT signing key, compile-time `include_bytes!`, **gitignored**.

```bash
openssl rand -out backend/src/secret.key 32
```

## Module layout

```
backend/src/
  main.rs                 # HttpServer, CORS, shared Client + DB
  config/
    app.rs                # route table
    db.rs                 # async MongoDB client
    server.rs             # static path config
  api/account_controller.rs
  api/feed_controller.rs
  models/feed.rs
  models/user.rs          # users collection, async CRUD
  models/user_token.rs    # JWT claims + signing
  models/response.rs      # { message, data } envelope
  services/
    feed_service.rs         # Immich / static photo feed
    forward_frontend.rs   # awc proxy (dev)
    serve_frontend.rs     # disk static (prod)
  utils/token.rs
```

## MongoDB

Driver **3.x** — all collection calls are `async` with `.await`.

Connection: `config_db().await` in `main`, database handle injected as `web::Data<Database>`.

Collection access uses typed documents: `db.collection::<User>("users")`.

## HTTP client

Outbound requests (dev proxy) use **`awc`**, not `actix_web::client` (removed in Actix 4).

## Build and run

```bash
# Dev (from repo root)
cargo build -p jheffmedia-site-backend --features forward-frontend
cargo test -p jheffmedia-site-backend

# Local watch (from backend/, MongoDB required)
cargo watch -x "run --features forward-frontend"
```

Docker dev container runs `backend/scripts/run-dev.sh` (`cargo watch` + `forward-frontend`).

## Errors

Service layer returns `ServiceError` with HTTP status + JSON body. Controllers map `Ok(err.response())` so errors still return structured JSON.

## Legacy README

Crate-level notes also in [backend/README.md](../backend/README.md) (kept short; this file is canonical).
