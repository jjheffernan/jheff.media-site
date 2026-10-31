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

### `GET /api/search?q=`

Public substring search across gallery and shoot titles, summaries, locations, dates, and photo titles, plus upcoming public schedule dates. Draft collections are omitted. Fewer than 2 characters returns no hits. At most 24 hits.

**Success** `200`

```json
{
  "query": "laguna",
  "hits": [
    { "kind": "gallery", "id": "laguna-seca", "title": "Track Day — Laguna Seca", "detail": "Laguna Seca" }
  ]
}
```

`kind` is `gallery`, `shoot`, or `event`.

### `GET /api/links`

Public social link list for `/links`. An empty list is `{ "links": [] }`.

**Success** `200`

```json
{
  "links": [
    { "label": "Instagram", "url": "https://instagram.com/jheffmedia" }
  ]
}
```

### `POST /api/telemetry`

Public. Records one page view. Body is `{ "path": "/galleries" }`. The path is the pathname only, 1–80 characters, and may contain letters, numbers, `/`, `-`, `_`, and `.`. Query strings are dropped. Nothing about the visitor is stored. More than 120 events from one address in 10 minutes returns `429`.

**Success** `204`

### `GET /api/admin/analytics`

Admin only. Page-view totals for the last 14 days.

**Success** `200`

```json
{
  "days": [{ "day": "2026-10-30", "n": 4 }],
  "paths": [{ "path": "/", "n": 4 }]
}
```

### `GET /metrics`

Prometheus text. Request counts by method and status, plus `telemetry_events_total`. Scrapes of this path are not counted. Counters reset when the API process restarts.

### `PUT /api/admin/links`

Admin only. Replaces the whole list. At most 12 links. Each label is 1–40 characters. Each URL must start with `http://` or `https://`.

**Success** `200` — the same `{ "links": [...] }` shape.

**Errors** `401` unauthorized, `403` not admin, `400` — `TOO_MANY`, `BAD_LABEL`, or `BAD_URL`.

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

### `PATCH /api/admin/galleries/{id}` / `PATCH /api/admin/shoots/{id}`

Update MongoDB collection metadata. Admin JWT required.

**Request body** (all fields optional): `{ "title", "summary", "date", "location", "status" }`

**Success** `200` — `{ "message": "UPDATED" }`

**Errors** `404` — `COLLECTION_NOT_FOUND` or `ENV_ONLY_COLLECTION` (env JSON collections cannot be edited here)

### `DELETE /api/admin/galleries/{id}` / `DELETE /api/admin/shoots/{id}`

Delete a MongoDB collection and best-effort remove its uploaded files under the content upload dir.

**Success** `200` — `{ "message": "REMOVED" }`

**Errors** `404` — `COLLECTION_NOT_FOUND` or `ENV_ONLY_COLLECTION`

### `DELETE /api/admin/galleries/{id}/media/{asset_id}` / shoots equivalent

Remove one media item from a MongoDB collection and delete its file on disk when present.

**Success** `200` — `{ "message": "REMOVED" }`

**Errors** `404` — `COLLECTION_NOT_FOUND`, `ENV_ONLY_COLLECTION`, or `MEDIA_NOT_FOUND`

### `PUT /api/admin/galleries/{id}/media/order` / shoots equivalent

Reorder media. Body: `{ "mediaIds": ["id1", "id2", ...] }`. Listed ids are placed first in that order; any remaining items keep relative order at the end.

**Success** `200` — `{ "message": "REORDERED" }`

### `POST /api/admin/galleries/{id}/media`

Upload one or more images to an existing MongoDB gallery. Admin JWT required. Env JSON galleries cannot take uploads.

**Headers:** `Authorization: bearer <jwt>`

**Request:** `multipart/form-data` with one or more `file` (or `files`) parts — JPEG, PNG, WebP, or GIF; default max 10 MB each. Optional text fields `setAsCover` and `featured`. `setAsCover` applies only to the first file; `featured` applies to every file in the request. Cover is unchanged when `setAsCover` is omitted.

**Success** `200`

```json
{
  "message": "UPLOADED",
  "data": [
    {
      "id": "<asset-uuid>",
      "thumbnailUrl": "/api/content/media/<asset-uuid>",
      "mediaUrl": "/api/content/media/<asset-uuid>",
      "mediaType": "image/jpeg",
      "featured": false
    }
  ]
}
```

**Errors**

| Status | `message` | When |
|--------|-----------|------|
| 401 | `UNAUTHORIZED` | Missing/invalid JWT |
| 403 | `FORBIDDEN` | Not admin |
| 404 | `COLLECTION_NOT_FOUND` | Gallery id not in MongoDB |
| 404 | `ENV_ONLY_COLLECTION` | Id exists only in env JSON |
| 400 | `MISSING_FILE` | No `file`/`files` field |
| 400 | `UNSUPPORTED_MEDIA_TYPE` | Not an allowed image type |
| 400 | `FILE_TOO_LARGE: max N bytes` | Over size limit |

### `POST /api/admin/shoots/{id}/media`

Upload one or more images to an existing MongoDB shoot. Same contract as gallery media upload.

### `GET /api/admin/library`

Admin JWT. Lists which library sources the server has configured. The response is only `immich` and/or `nextcloud`. API keys and passwords are not returned.

### `GET /api/admin/library/{source}`

Admin JWT. `source` is `immich` or `nextcloud`. Returns at most 12 file names and ids from the configured album or folder. The browser never talks to Immich or Nextcloud.

### `POST /api/admin/library/{source}`

Admin JWT. Copies up to 6 selected files into an existing Mongo gallery or shoot whose status is `draft`. Files are not marked featured and do not become the cover. Draft collections stay off the public lists, detail pages, and feed until the status changes.

```json
{ "kind": "gallery", "collectionId": "track-day", "assetIds": ["file-or-uuid"] }
```

**Errors** `400` — `NOT_DRAFT`, `TOO_MANY`, `BAD_ID`, `EMPTY_SELECTION`, `FILE_TOO_LARGE`, `UNSUPPORTED_MEDIA_TYPE`. `404` — `NOT_CONFIGURED`, `COLLECTION_NOT_FOUND`. `502` — `UPSTREAM`.

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
| `preferredAt` | string | no |

`kind` is `booking` for a booking request and `contact` otherwise. A missing or unknown value is stored as `contact`. Booking mail uses the subject prefix `Booking:`. `preferredAt` is an optional booking time (`2026-11-02T15:45`). Leave it out, or send it empty, to book without a time. A booking with a value that is not a date returns 400 `Preferred time is not a date.` Contact messages drop the field. New documents store `status` as `new`. Older documents with no `status` are treated as `new` when listed. The inbox returns `preferredAt` as an empty string when none was sent.

`website` is a honeypot. Leave it empty. A non-empty value returns the success envelope and is not stored.

**Success** `200` — `{ "message": "CONTACT_SUCCESS", "data": "Message received. We'll get back to you soon." }`

**Errors**

| Status | `message` | When |
|--------|-----------|------|
| 400 | `Name, email, and message are required.` | Blank required field |
| 400 | `Preferred time is not a date.` | Booking `preferredAt` is set and is not a date |
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

### `GET /api/admin/contacts`

Admin JWT. Newest `contacts` first. Documents missing `status` come back as `new`.

**Success** `200` — JSON array. Each item has `id`, `kind`, `name`, `email`, `subject`, `message`, `preferredAt` (empty when none was sent), `createdAt` (RFC3339), and `status`.

**Errors** `401` `UNAUTHORIZED`. `403` `FORBIDDEN`.

### `PATCH /api/admin/contacts/{id}`

Admin JWT. Body: `{ "status": "read" }`. `status` is any non-empty string up to 32 characters (`new`, `read`, and `archived` are the ones the admin inbox offers).

**Success** `200` `{ "message": "UPDATED" }`.

**Errors** `400` `INVALID_ID` or `INVALID_STATUS`. `404` `NOT_FOUND`. `401` `UNAUTHORIZED`. `403` `FORBIDDEN`.

### `DELETE /api/admin/contacts/{id}`

Admin JWT. Removes that contact document.

**Success** `200` `{ "message": "REMOVED" }`.

**Errors** `400` `INVALID_ID`. `404` `NOT_FOUND`. `401` `UNAUTHORIZED`. `403` `FORBIDDEN`.

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

### `PATCH /api/admin/schedule/{id}`

Admin JWT. Same body as create: `title`, `startsAt`, optional `location`, `kind`. Success `200` `{ "message": "UPDATED" }`. `400` `INVALID_ID` or `INVALID_EVENT`. `404` `NOT_FOUND`.

### `GET /api/account/prints/orders`

Auth required. Newest print requests first.

### `GET /api/admin/prints/orders`

Admin JWT. Every print order, newest first. Each row has the account order fields plus `userId`, `email` and `username` when that user still exists, and `items` (the photos on the request).

### `PATCH /api/admin/prints/orders/{id}`

Admin JWT. Body: `status` (`requested`, `fulfilled`, or `cancelled`) and optional `paymentStatus` (`unpaid`, `invoiced`, or `paid`). This records status only and does not charge a card. Success `200` is the updated order in the admin list shape. `400` `INVALID_ID`, `INVALID_STATUS`, or `INVALID_PAYMENT_STATUS`. `404` `NOT_FOUND`.

## CORS

Allowed methods: `GET`, `POST`, `PUT`, `PATCH`, `DELETE`.  
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
