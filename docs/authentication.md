# Authentication

JWT-based auth with a server-side login session. The token alone is not enough—the backend also checks that the session id in the JWT matches MongoDB.

See also: [api.md](./api.md), [backend.md](./backend.md), [frontend.md](./frontend.md).

## Overview

| Layer | Mechanism |
|-------|-----------|
| Password storage | bcrypt hash in `users` collection |
| Token | JWT (HS256). Production reads `YEW_FULLSTACK_JWT_SECRET_FILE` or hex `YEW_FULLSTACK_JWT_SECRET`. Dev falls back to `backend/src/secret.key` at startup |
| Session | Random `loginSession` field on user document, embedded in JWT |
| Client storage | Browser session storage via Yew `StorageService` |

## JWT payload (`UserToken`)

| Field | Meaning |
|-------|---------|
| `iat` | Issued-at (seconds) |
| `exp` | Expiry (issued + 7 days) |
| `user` | Username string |
| `login_session` | Server session id (must match DB) |

Signing key: `backend/src/config/jwt.rs` loads the key once at startup. `user_token.rs` signs with that key.

## Login flow

1. Client `POST /api/auth/login` with `emailOrUsername` + `password`, and `totpCode` when 2FA is already enabled (camelCase JSON).
2. The attempt is counted first: 8 login attempts per IP per 10 minutes, separate from signup. Over the limit returns `429`.
3. Backend finds user by email or username.
4. bcrypt verifies password.
5. If `totp_enabled` and the code is missing, the response is `401` `TOTP_REQUIRED` and no session is written. A wrong code is `400` `TOTP_INVALID`, also with no session.
6. New `loginSession` generated (`Uuid`) and stored on the user document.
7. JWT minted containing username + `login_session`.
8. Response: `LOGIN_SUCCESS` with `token`, `token_type` (`bearer`), and public user (`email`, `username`).

Frontend (`login.rs`) parses the response, then `AuthEventBus` stores `{ jwt, user }` in session storage under key `Auth`.

## Logout flow

1. Client `POST /api/auth/logout` with header `Authorization: bearer <jwt>`.
2. Backend decodes JWT, verifies session still valid in DB.
3. `loginSession` removed from user document (`$unset`).
4. Response: `LOGOUT_SUCCESS`.

Frontend (`controls.rs`) calls logout API, then `AuthEventBus` clears session storage.

**Important:** Logout must use the `Authorization` header (not `Authentication`).

## Session validation

`User::is_valid_login_session` counts users where `username` and `loginSession` match the JWT claims. Used during logout token processing; extend here for protected API routes.

Invalid or expired JWT → decode fails. Valid JWT but wrong/missing session → validation fails.

## Signup flow

1. Client `POST /api/auth/signup` with user JSON (`email`, `username`, `password` camelCase).
2. Duplicate email/username → `400` with `USER_ALREADY_EXISTS`.
3. Password hashed with bcrypt; user inserted.
4. Success → `200` with `SUCCESS` and new user id (hex string).

Signup does **not** log the user in automatically—the header still shows login/signup until they authenticate.

## UI behavior

| State | Header shows |
|-------|----------------|
| No session | Login + Signup forms |
| Session in storage | `AuthControls` (greeting + logout link) |

`Header` bridges `AuthEventBus` and re-renders on auth changes. Profile route is not gated yet (placeholder page).

## Security notes

- Replace JWT secret per deployment (`YEW_FULLSTACK_JWT_SECRET_FILE` or hex env); never commit `secret.key` (gitignored).
- JWT expiry is one week; `exp` is validated on every decode; session invalidation on logout is server-side.
- CORS allows only origins listed in `YEW_FULLSTACK_CORS_ORIGINS` (comma-separated), defaulting to `YEW_FULLSTACK_FORWARD_FRONTEND_URL` or `http://localhost:8000` — not wildcard.
- Response headers include `X-Content-Type-Options`, `X-Frame-Options`, and `Referrer-Policy`.
- Passwords never returned in API responses (`PublicUserDTO` only has email, username, role, totpEnabled).
- Signup and password change require at least 8 characters; passwords stored as bcrypt hashes.
- Protected routes use `Authorization: bearer <jwt>` with server-side session validation.
- Email-change tokens are not logged; wire a mailer before enabling email change in production.
- Client stores JWT in **session storage** (tab-scoped). Any XSS in the WASM app could exfiltrate tokens — keep dependencies patched and avoid inline third-party scripts.
- TOTP disable requires a valid code when 2FA is enabled.
- Admin role is assigned at signup via `YEW_FULLSTACK_ADMIN_EMAILS`; review admin list carefully.

### Hardening backlog

- Content-Security-Policy header tuned for WASM + any embeds (Instagram/YouTube).
- HTTPS termination and HSTS at the reverse proxy (Caddy/nginx).

## Code map

| Concern | Location |
|---------|----------|
| User model + DB | `backend/src/models/user.rs` |
| Token mint/verify | `backend/src/models/user_token.rs`, `backend/src/utils/token.rs` |
| HTTP handlers | `backend/src/api/account_controller.rs` |
| Client session context | `frontend/src/context/auth.rs` |
| Login UI | `frontend/src/components/auth/login.rs` |
| Logout UI | `frontend/src/components/auth/controls.rs` |
