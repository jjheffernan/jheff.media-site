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
| `totpCode` | string | when 2FA is enabled |

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
| 401 | `TOTP_REQUIRED` | Password matched and 2FA is on, but no code was sent. No session or JWT is issued |
| 400 | `TOTP_INVALID` | 2FA code did not match. No session or JWT is issued |
| 429 | `Too many attempts. Try again later.` | More than 8 login attempts from one IP in 10 minutes |

Signup uses the same 429 limit on its own counter. Frontend maps 404 → bad username field, 400 `BAD_PASSWORD` → bad password field, and shows the authenticator step on `TOTP_REQUIRED`.

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

Legacy Immich or static grid. The homepage uses `GET /api/media/feed`. This route remains for the old feed provider.

Public photo feed metadata. Images are **not** stored in the site repo — thumbnails are proxied from Immich (or static URLs).

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

### `GET /api/media/feed`

Homepage feed. Merges Instagram, YouTube, galleries, and shoots. Direct JSON, not the auth envelope.

**Query**

| Param | Type | Default | Purpose |
|-------|------|---------|---------|
| `limit` | integer | `24` | Max items (1–48) |
| `source` | string | empty | `instagram`, `youtube`, `gallery`, or `shoot`. Empty returns every source |

**Success** `200`

```json
{
  "items": [
    {
      "id": "post-or-asset",
      "source": "gallery",
      "mediaType": "image",
      "title": "Optional",
      "thumbnailUrl": "/api/content/media/uuid",
      "mediaUrl": null,
      "linkUrl": "/galleries/my-gallery",
      "collectionId": "my-gallery",
      "collectionKind": "gallery",
      "collectionTitle": "Track Day",
      "featured": false
    }
  ],
  "sources": ["instagram", "gallery"]
}
```

**Errors** `400` `UNKNOWN_SOURCE` for any other `source`.

### `GET /api/media/instagram/featured`

**Success** `200` — featured post JSON when `YEW_FULLSTACK_INSTAGRAM_FEATURED_POST_URL` is set.

**Success** `204` — empty body when that URL is missing. The home card renders nothing.

### `GET /api/content/hub`

Public bundle for the content page. Direct JSON:

```json
{
  "social": { "accounts": [], "posts": [], "source": "none" },
  "shoots": { "items": [], "source": "config" },
  "galleries": { "items": [], "source": "config" }
}
```

Counts are trimmed to 3 posts, 2 shoots, and 2 galleries. Nested shapes match `GET /api/social/hub`, `GET /api/shoots`, and `GET /api/galleries`.

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

**Request:** `multipart/form-data` with field `file` (JPEG, PNG, WebP, or GIF; default max 10 MB). Optional text fields `setAsCover` and `featured` (`true` sets the collection cover or tags the new item). Cover is unchanged when `setAsCover` is omitted.

**Success** `200`

```json
{
  "message": "UPLOADED",
  "data": {
    "id": "<asset-uuid>",
    "thumbnailUrl": "/api/content/media/<asset-uuid>",
    "mediaUrl": "/api/content/media/<asset-uuid>",
    "mediaType": "image/jpeg",
    "featured": false
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

### `POST /api/contact`

Public inquiry. Stores the message in MongoDB `contacts`. When `YEW_FULLSTACK_SMTP_HOST` and `YEW_FULLSTACK_CONTACT_EMAIL` are set, the backend also sends one plain-text email. A skipped or failed send still returns success.

**Request body**

| Field | Type | Required |
|-------|------|----------|
| `name` | string | yes |
| `email` | string | yes |
| `subject` | string | no |
| `message` | string | yes |
| `website` | string | no |
| `kind` | string | no |

`kind` is `booking` for a booking request and `contact` otherwise. A missing or unknown value is stored as `contact`. Booking mail uses the subject prefix `Booking:`.

`website` is a honeypot. Leave it empty. A non-empty value returns the success envelope and is not stored.

**Success** `200` — `{ "message": "CONTACT_SUCCESS", "data": "Message received. We'll get back to you soon." }`

**Errors**

| Status | `message` | When |
|--------|-----------|------|
| 400 | `Name, email, and message are required.` | Blank required field |
| 429 | `Too many messages. Try again later.` | More than 5 submissions from one IP in 10 minutes |

**Env (optional)**

| Variable | Default | Purpose |
|----------|---------|---------|
| `YEW_FULLSTACK_SMTP_HOST` | empty | Skip email when empty |
| `YEW_FULLSTACK_SMTP_PORT` | `587` | STARTTLS |
| `YEW_FULLSTACK_SMTP_USERNAME` | empty | Optional auth |
| `YEW_FULLSTACK_SMTP_PASSWORD` | empty | Optional auth |
| `YEW_FULLSTACK_SMTP_FROM` | contact email | From address |
| `YEW_FULLSTACK_CONTACT_EMAIL` | empty | Recipient. Skip email when empty |

### `GET /api/account/prints`

Auth required. Current print selections, including `collectionId`, `collectionKind`, and `collectionTitle` when the feed item had them.

### `POST /api/account/prints/checkout`

Auth required. Snapshots the current selections into `print_orders` with status `requested`. Does not charge a card and does not clear the selection list.

**Request body:** `{ "size": "8x10", "note": "optional delivery note" }`

`size` is required: `8x10` (2500 cents), `11x14` (4500), or `16x20` (7500). One size applies to every photo. `totalCents` is the unit price times the photo count. `paymentStatus` is `unpaid`.

**Success** `200`

```json
{
  "message": "PRINT_REQUESTED",
  "data": {
    "id": "<objectId>",
    "status": "requested",
    "note": "11x14, matte",
    "itemCount": 2,
    "createdAt": "2026-10-29T00:00:00Z",
    "size": "8x10",
    "unitPriceCents": 2500,
    "totalCents": 5000,
    "paymentStatus": "unpaid"
  }
}
```

**Errors** `400` `NO_SELECTIONS` when the list is empty. `400` `UNKNOWN_SIZE` for any other size. `401` `UNAUTHORIZED`.

### `GET /api/schedule`

Public. `embedUrl` and `source` stay as before. `events` lists public `schedule_events` whose `startsAt` is in the future, oldest first. Each event has `id`, `title`, `startsAt`, `kind` (`shoot`, `instruction`, or `other`), and optional `location`.

### `POST /api/admin/schedule`

Admin JWT. Body: `title`, `startsAt`, optional `location`, `kind`. Success `200` `{ "message": "CREATED", "data": { "id": "<objectId>" } }`. `400` `INVALID_EVENT` when the title, start, or kind is unusable.

### `DELETE /api/admin/schedule/{id}`

Admin JWT. Success `200` `{ "message": "REMOVED" }`.

### `GET /api/account/prints/orders`

Auth required. Newest print requests first.

## CORS

Allowed methods: `GET`, `POST`, `PUT`, `DELETE`.  
Allowed headers include `Authorization`, `Accept`, `Content-Type`.  
Origins come from `YEW_FULLSTACK_CORS_ORIGINS`.

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
