# API reference

REST API served by the Actix backend. Base path in dev: `http://localhost:8080`.

All successful JSON responses use the envelope:

```json
{
  "message": "<status code string>",
  "data": <payload>
}
```

Errors use the same envelope with an appropriate HTTP status; `data` is often an empty string.

Implementation: `backend/src/models/response.rs`, handlers in `backend/src/api/account_controller.rs`.

## Endpoints

### `POST /api/auth/signup`

Create a new user.

**Request body** (JSON, camelCase fields on wire):

| Field | Type | Required |
|-------|------|----------|
| `email` | string | yes |
| `username` | string | yes |
| `password` | string | yes |

**Success** `200`

```json
{
  "message": "SUCCESS",
  "data": "<objectId hex>"
}
```

**Errors**

| Status | `message` | When |
|--------|-----------|------|
| 400 | `USER_ALREADY_EXISTS` | Email or username taken |
| 500 | `MONGO_ERROR` | Database insert failed |

### `POST /api/auth/login`

Authenticate and receive a JWT.

**Request body:**

| Field | Type | Required |
|-------|------|----------|
| `emailOrUsername` | string | yes |
| `password` | string | yes |

**Success** `200`

```json
{
  "message": "LOGIN_SUCCESS",
  "data": {
    "token": "<jwt>",
    "token_type": "bearer",
    "user": {
      "email": "...",
      "username": "..."
    }
  }
}
```

**Errors**

| Status | `message` | When |
|--------|-----------|------|
| 404 | `USER_NOT_FOUND` | No matching email/username |
| 400 | `BAD_PASSWORD` | Wrong password or session update failed |

Frontend maps 404 → bad username field, 400 → bad password field.

### `POST /api/auth/logout`

Invalidate server session for the token.

**Headers**

| Header | Value |
|--------|-------|
| `Authorization` | `bearer <jwt>` (case-sensitive prefix in current code) |

**Success** `200`

```json
{
  "message": "LOGOUT_SUCCESS",
  "data": ""
}
```

**Errors**

| Status | `message` | When |
|--------|-----------|------|
| 400 | `MISSING_TOKEN` | No `Authorization` header |
| 500 | `TOKEN_PROCESSING_ERROR` | Invalid token, user not found, or session mismatch |

## CORS

Allowed methods: `GET`, `POST`, `PUT`, `DELETE`.  
Allowed headers include `Authorization`, `Accept`, `Content-Type`.  
Wildcard origin in current config.

## MongoDB

Users collection: `users` (see `USERS_COLLECTION` in `user.rs`).

Document fields (camelCase in DB): `_id`, `email`, `username`, `password` (bcrypt), optional `loginSession`.

## Examples

```bash
# Signup
curl -s -X POST http://localhost:8080/api/auth/signup \
  -H 'Content-Type: application/json' \
  -d '{"email":"a@b.com","username":"alice","password":"secret"}'

# Login
curl -s -X POST http://localhost:8080/api/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"emailOrUsername":"alice","password":"secret"}'

# Logout
curl -s -X POST http://localhost:8080/api/auth/logout \
  -H 'Authorization: bearer <token from login>'
```

Behavior details: [authentication.md](./authentication.md).
