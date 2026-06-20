# Backend rules — Actix + MongoDB

Applies to `backend/**/*.rs`.

**Canonical docs:** [docs/backend.md](../../docs/backend.md) · [docs/api.md](../../docs/api.md) · [docs/authentication.md](../../docs/authentication.md)

## Stack

- Actix Web 4, `awc` for HTTP client (not `actix_web::client`)
- MongoDB driver 3.x — **all** DB calls are `async` + `.await`
- JWT via `jsonwebtoken`; session validated in MongoDB (`loginSession` field)

## Dev vs prod frontend

- **Dev** (`forward-frontend` feature): backend proxies non-`/api` traffic to webpack (`forward_frontend.rs`)
- **Prod**: static files from `YEW_FULLSTACK_STATIC` (see `serve_frontend.rs`)

Build dev backend:

```bash
cargo build -p jheffmedia-site-backend --features forward-frontend
cargo watch -x "run --features forward-frontend"   # local, from backend/
```

## API surface

Routes under `/api/auth/`:

- `POST /api/auth/signup`
- `POST /api/auth/login`
- `POST /api/auth/logout` — expects `Authorization: bearer <token>`

## Patterns to follow

- Register app state with `app_data(web::Data::new(...))` (not deprecated `.data()`)
- CORS: `Cors::default()` + config (no `.finish()` in actix-cors 0.7)
- Collection access: `db.collection::<User>(USERS_COLLECTION)` (turbofish for local `User` type)
- `config_db()` is async; call `.await` in `main`

## Do not

- Reintroduce sync MongoDB driver APIs
- Change JWT key size without updating `user_token.rs` and README
- Add blocking DB calls inside async handlers without `block_in_place` (prefer keeping handlers async end-to-end)
