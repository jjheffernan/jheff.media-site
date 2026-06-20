# Proxmox deployment

Deploy **jheff.media** on a Proxmox host using Docker — single production container (Actix + baked Yew frontend) plus MongoDB. Photos stay on your self-hosted Immich (or other) instance; the site only proxies thumbnails.

Related: [architecture.md](./architecture.md) · [deployment-dev.md](./deployment-dev.md) · [backend.md](./backend.md) · [version-matrix.md](./version-matrix.md)

## Target layout

```
                    ┌─────────────────────────────────────┐
  Internet          │ Proxmox host                        │
      │             │  ┌─────────────┐  ┌──────────────┐  │
      ▼             │  │ LXC/VM      │  │ LXC (opt.)   │  │
  Reverse proxy     │  │ site stack  │  │ Immich       │  │
  (Caddy/Nginx)     │  │ :8080       │  │ :2283        │  │
      │             │  │ Mongo :27017│  │ photo library│  │
      └─────────────┼──┤             │  └──────────────┘  │
                    │  └─────────────┘                   │
                    └─────────────────────────────────────┘
```

| Component | Role |
|-----------|------|
| **Site LXC/VM** | Docker: `jheffmedia-site` app + MongoDB |
| **Immich** (recommended) | Photo library; API key never exposed to browsers |
| **Reverse proxy** | TLS, optional auth, route `jheff.media` → app |

## 1. Proxmox prerequisites

1. Create an **LXC** (Debian 12/Ubuntu 24.04) or **VM** with 2+ GB RAM, 20+ GB disk.
2. Install Docker (Compose v2):

```bash
curl -fsSL https://get.docker.com | sh
```

3. Clone the repo (or pull release image after CI):

```bash
git clone https://github.com/jjheffernan/jheff.media-site.git
cd jheff.media-site
```

## 2. Build production image

From repo root:

```bash
openssl rand -out backend/src/secret.key 32   # once per environment
scripts/build.sh                                # or: docker build -t jheffmedia-site .
```

The multi-stage `Dockerfile` runs `trunk build --release` and embeds `frontend/dist/` in the Actix binary image.

## 3. Configure environment

Create `scripts/.env.production` (do not commit):

```bash
# App
YEW_FULLSTACK_HOST=0.0.0.0
YEW_FULLSTACK_PORT=8080
YEW_FULLSTACK_DB_CONNSTR=mongodb://root:CHANGE_ME@db:27017/jheffmedia?authSource=admin
YEW_FULLSTACK_DB_NAME=jheffmedia

# Photo feed — Immich (recommended)
YEW_FULLSTACK_FEED_PROVIDER=immich
YEW_FULLSTACK_IMMICH_URL=https://immich.internal.example.com
YEW_FULLSTACK_IMMICH_API_KEY=your-api-key-here
YEW_FULLSTACK_IMMICH_ALBUM_ID=album-uuid-for-homepage-feed

# Optional: shared-link key if thumbnails need ?key=
# YEW_FULLSTACK_IMMICH_SHARED_KEY=

# Dev/demo without Immich — static external URLs (no repo bloat)
# YEW_FULLSTACK_FEED_PROVIDER=static
# YEW_FULLSTACK_FEED_STATIC_JSON='[{"id":"1","thumbnailUrl":"https://picsum.photos/seed/a/800/600","title":"Sample"}]'
```

**Immich setup**

1. Deploy Immich (official compose) on the same Proxmox host or another LXC.
2. Create an API key: **Account → API Keys**.
3. Create a public **Album** for the homepage feed; copy its UUID to `YEW_FULLSTACK_IMMICH_ALBUM_ID`.
4. Ensure the site container can reach `YEW_FULLSTACK_IMMICH_URL` (DNS or `/etc/hosts` on the LXC).

The backend proxies `/api/feed` and `/api/feed/thumbnail/{id}` so the browser never needs the Immich API key.

## 4. Run with Docker Compose

Extend `scripts/docker-compose.yml` or run:

```bash
docker compose -f scripts/docker-compose.yml --env-file scripts/.env.production up -d
```

Default compose exposes **8080**. MongoDB should not be published publicly in production — remove the `27017` port mapping or bind to localhost only.

## 5. Reverse proxy (TLS)

Example **Caddy** on the same LXC or a dedicated proxy VM:

```caddy
jheff.media {
    reverse_proxy localhost:8080
}
```

Example **Nginx**:

```nginx
server {
    listen 443 ssl;
    server_name jheff.media;
    ssl_certificate     /path/to/fullchain.pem;
    ssl_certificate_key /path/to/privkey.pem;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

Use **http://localhost:8080** only behind the proxy; users hit `https://jheff.media`.

## 6. Verify

```bash
curl -s -o /dev/null -w "%{http_code}\n" https://jheff.media/
curl -s https://jheff.media/api/feed | jq .
```

Homepage should show the photo grid when feed env vars are set.

## 7. Updates (main branch)

Production deploys should track **`main`**:

```bash
git checkout main && git pull
openssl rand -out backend/src/secret.key 32   # only if missing
scripts/build.sh
docker compose -f scripts/docker-compose.yml --env-file scripts/.env.production up -d --build
```

Day-to-day feature work happens on **`dev`**; merge to `main` when ready to ship.

## 8. Backups

| Data | Backup |
|------|--------|
| MongoDB users/sessions | `mongodump` from the `db` container volume |
| JWT secret | `backend/src/secret.key` — store in secrets manager; **changing it invalidates sessions** |
| Photos | Immich backup (not in this repo) |

## 9. Troubleshooting

| Symptom | Fix |
|---------|-----|
| Empty homepage feed | Check `YEW_FULLSTACK_FEED_PROVIDER`, Immich URL reachability, album ID |
| Thumbnails 502 | Immich API key, asset ID, or shared key |
| Blank UI after deploy | Rebuild image after frontend changes (`trunk build` in Dockerfile stage) |
| WASM won't load | Ensure proxy passes `*.wasm` with correct MIME; use single origin via reverse proxy |

## Checklist

- [ ] LXC/VM with Docker
- [ ] `secret.key` generated before image build
- [ ] MongoDB credentials rotated from dev defaults
- [ ] Immich album + API key configured
- [ ] TLS reverse proxy in front of :8080
- [ ] `curl /api/feed` returns items
- [ ] Homepage loads styled UI + photo grid
