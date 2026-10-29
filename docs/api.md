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

### `GET /api/feed`

Public photo feed metadata for the homepage grid. Images are **not** stored in the site repo — thumbnails are proxied from Immich (or static URLs).

**Query**

| Param | Type | Default | Purpose |
|-------|------|---------|---------|
| `limit` | integer | `24` | Max items (1–48) |

**Success** `200` — direct JSON (not the auth `{ message, data }` envelope):

```json
{
  "items": [
    {
      "id": "asset-uuid",
      "title": "optional filename",
      "thumbnailUrl": "/api/feed/thumbnail/asset-uuid",
      "width": 4032,
      "height": 3024
    }
  ],
  "source": "immich"
}
```

`source` is `immich`, `static`, or `none` when unconfigured.

**Errors** use the standard envelope with HTTP 4xx/5xx.

Backend env: [backend.md](./backend.md#photo-feed).

### `GET /api/feed/thumbnail/{asset_id}`

Proxies a preview thumbnail from Immich for use in `<img src="...">`. Only available when `YEW_FULLSTACK_FEED_PROVIDER=immich`.

**Success** `200` — image bytes (`image/jpeg` or `image/webp`).

**Success** `200` — image bytes (`image/jpeg`, `image/png`, `image/webp`, or `image/gif`).

### `GET /api/galleries`

List gallery summaries (env JSON merged with MongoDB).

**Success** `200` — direct JSON:

```json
{
  "items": [
    {
      "id": "my-gallery",
      "title": "Track Day",
      "summary": "Optional blurb",
      "coverUrl": "/api/content/media/uuid",
      "status": "published",
      "date": "2026-06-01",
      "location": "Laguna Seca",
      "mediaCount": 12,
      "hasVideo": false
    }
  ],
  "source": "config"
}
```

### `GET /api/galleries/{id}`

Gallery detail with media items.

**Success** `200` — `ContentCollection` JSON (camelCase).

**Errors** `404` — `{ "message": "GALLERY_NOT_FOUND", "data": "" }`

### `GET /api/shoots`

List shoot summaries (same shape as galleries).

### `GET /api/shoots/{id}`

Shoot detail with media items.

**Errors** `404` — `{ "message": "SHOOT_NOT_FOUND", "data": "" }`

### `GET /api/content/media/{asset_id}`

Serve an uploaded shoot/gallery image from disk. Public read.

**Success** `200` — image bytes.

**Errors** `404` — `{ "message": "MEDIA_NOT_FOUND", "data": "" }`

### `POST /api/admin/galleries`

Create a gallery collection (admin JWT required).

**Headers:** `Authorization: bearer <jwt>`

**Request body:** `ContentCollection` JSON (`id`, `title`, optional metadata; `media` usually empty on create).

**Success** `200` — `{ "message": "CREATED" }`

### `POST /api/admin/shoots`

Create a shoot collection (admin JWT required). Same shape as gallery create.

### `POST /api/admin/galleries/{id}/media`

Upload an image to an existing MongoDB gallery. Admin JWT required.

**Headers:** `Authorization: bearer <jwt>`

**Request:** `multipart/form-data` with field `file` (JPEG, PNG, WebP, or GIF; default max 10 MB).

**Success** `200`

```json
{
  "message": "UPLOADED",
  "data": {
    "id": "<asset-uuid>",
    "thumbnailUrl": "/api/content/media/<asset-uuid>",
    "mediaUrl": "/api/content/media/<asset-uuid>",
    "mediaType": "image/jpeg"
  }
}
```

**Errors**

| Status | `message` | When |
|--------|-----------|------|
| 401 | `UNAUTHORIZED` | Missing/invalid JWT |
| 403 | `FORBIDDEN` | Not admin |
| 404 | `COLLECTION_NOT_FOUND` | Gallery id not in MongoDB |
| 400 | `MISSING_FILE` | No `file` field |
| 400 | `UNSUPPORTED_MEDIA_TYPE` | Not an allowed image type |
| 400 | `FILE_TOO_LARGE: max N bytes` | Over size limit |

### `POST /api/admin/shoots/{id}/media`

Upload an image to an existing MongoDB shoot. Same contract as gallery media upload.

**Env (optional)**

| Variable | Default | Purpose |
|----------|---------|---------|
| `YEW_FULLSTACK_CONTENT_UPLOAD_DIR` | `uploads/content` | On-disk image storage |
| `YEW_FULLSTACK_CONTENT_UPLOAD_MAX_BYTES` | `10485760` | Per-file upload limit |

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
