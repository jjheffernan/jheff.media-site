# Proxmox deployment

Deploy **jheff.media** on a Proxmox host using Docker — one production container (Actix + baked Yew frontend) and MongoDB. `GET /metrics` is there for an existing Prometheus. Photos stay on your self-hosted Immich (or other) instance; the site only proxies thumbnails.

**New here?** Start with the short guide: [deployment.md](./deployment.md). Catalog metadata: [install/registry.json](../install/registry.json).

Related: [architecture.md](./architecture.md) · [deployment-dev.md](./deployment-dev.md) · [backend.md](./backend.md) · [version-matrix.md](./version-matrix.md) · [install/proxmox/README.md](../install/proxmox/README.md)

## One-line install (recommended)

From the **Proxmox host** (creates a new LXC) or **inside an existing LXC/VM** as root:

```bash
bash -c "$(curl -fsSL https://raw.githubusercontent.com/jjheffernan/jheff.media-site/main/install/proxmox/install.sh)"
```

Guided whiptail sections cover repository branch (`main` by default; `testing` for non-prod), site port, bundled MongoDB credentials, and photo feed (`static` demo works without Immich). Unattended: `UNATTENDED=yes MONGO_PASSWORD='…'`. Day-2: `jheffmedia --update` / `--update -d`. Full reference: [install/proxmox/README.md](../install/proxmox/README.md) · other targets: [install/README.md](../install/README.md).

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
| **Site LXC/VM** | Docker: `jheffmedia-site` app and MongoDB. Scrape `GET /metrics` from an existing Prometheus. |
| **Immich** (recommended) | Photo library; API key never exposed to browsers |
| **Reverse proxy** | TLS, optional auth, route `jheff.media` → app |

## 1. Manual path (optional)

Prefer the [one-line install](#one-line-install-recommended). For a hand-rolled LXC/VM:

1. Debian/Ubuntu guest with 2+ GB RAM, 20+ GB disk, Docker (Compose v2), nesting if LXC.
2. Clone the repo and generate a JWT secret once:

```bash
git clone https://github.com/jjheffernan/jheff.media-site.git
cd jheff.media-site
openssl rand -out backend/src/secret.key 32   # runtime mount, not baked into the image
scripts/build.sh                               # or: docker build -t jheffmedia/application .
```

The multi-stage `Dockerfile` runs `trunk build --release` and embeds `frontend/dist/` in the Actix image. JWT secret is loaded at container start via `YEW_FULLSTACK_JWT_SECRET_FILE` (see `install/common/docker-compose.yml`).

## 2. Configure environment

Create `install/common/.env.production` from `install/common/.env.production.example` (do not commit). `scripts/prod.sh` still accepts `scripts/.env.production` if you keep a copy there:

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

Homepage feed is `GET /api/media/feed` (Immich/static/Nextcloud items merge in when configured). Immich/Nextcloud thumbnails are `/api/media/thumbnail/{id}` so the browser never needs the API key.

## 3. Run with Docker Compose

Production compose for install targets lives at `install/common/docker-compose.yml` (no public Mongo port). After guided install, env is at `install/common/.env.production`.

```bash
docker compose -f install/common/docker-compose.yml --env-file install/common/.env.production up -d
```

Default compose exposes **8080**. MongoDB is not published.

## 4. Reverse proxy (TLS)

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

## 5. Verify

```bash
scripts/smoke.sh
# or against public URL:
BASE_URL=https://jheff.media scripts/smoke.sh
# Immich step runs when FEED_PROVIDER=immich (+ URL/key); otherwise skips.

curl -s https://jheff.media/api/media/feed | jq .
# Immich (when configured):
# curl -s 'https://jheff.media/api/media/feed?source=immich&limit=3' | jq .
# curl -sI https://jheff.media/api/media/thumbnail/<asset-id>
```

Homepage should show the photo grid when feed env vars are set.

## 6. Updates

On an installed host, prefer the CLI:

```bash
jheffmedia --update                   # production: pull main
jheffmedia --update -d                # non-prod: pull testing
```

Manual rebuild fallback (keep existing `backend/src/secret.key` — changing it invalidates sessions):

```bash
git checkout main && git pull
scripts/build.sh
docker compose -f install/common/docker-compose.yml --env-file install/common/.env.production up -d --build
scripts/smoke.sh
```

Day-to-day feature work happens on **`dev`**; merge to `main` when ready to ship. Staging channel is **`testing`**.

## 7. Backups

| Data | Backup |
|------|--------|
| MongoDB users/sessions | `mongodump` from the `db` container volume |
| JWT secret | `backend/src/secret.key` — mounted at container start; store in secrets manager; **changing it invalidates sessions** |
| Photos | Immich backup (not in this repo) |

## 8. Troubleshooting

| Symptom | Fix |
|---------|-----|
| Empty homepage feed | Check `YEW_FULLSTACK_FEED_PROVIDER`, Immich URL reachability, album ID |
| Thumbnails 502 | Immich API key, asset ID, or shared key |
| Blank UI after deploy | Rebuild image after frontend changes (`trunk build` in Dockerfile stage) |
| WASM won't load | Ensure proxy passes `*.wasm` with correct MIME; use single origin via reverse proxy |

## Security hardening (LXC)

Advisory checklist for a single-node LXC. Nothing here is applied automatically — run `jheffmedia --postinstall --security` on the CT for a live report (no secret values printed).

### Bind and CORS

Default `HTTP_BIND=0.0.0.0` publishes the app to the LAN (convenient before a proxy exists). When Caddy/Nginx/Tunnel terminates TLS **on the same LXC**, prefer loopback:

```bash
# in install/common/.env.production
HTTP_BIND=127.0.0.1
YEW_FULLSTACK_CORS_ORIGINS=https://jheff.media   # real HTTPS origin(s), comma-separated
# then:
systemctl restart site
```

`PUBLIC_URL` / guided install seeds CORS when unset; override for the public hostname.

### Firewall and SSH (host / CT)

Example **ufw** on Debian/Ubuntu (review before enable — do not lock yourself out):

```bash
ufw default deny incoming
ufw allow OpenSSH
ufw allow 80/tcp
ufw allow 443/tcp
ufw enable
```

Prefer SSH **key-only** (`PasswordAuthentication no`). Keep the CT patched with `unattended-upgrades`.

### Threat notes

| Topic | Note |
|-------|------|
| Nesting + docker group | Docker nesting on an LXC and membership in `docker` ≈ root on that CT — intentional for `jheffmedia` day-2 ops. Treat docker-group users as fully trusted. |
| `GET /metrics` | Unauthenticated Prometheus text on the app port. Restrict at the reverse proxy (allowlist) or keep `HTTP_BIND=127.0.0.1`. |
| Updates | Prefer `jheffmedia --update` over re-running `curl \| bash` install scripts. |
| Pin install | First install: set `INSTALL_REF=<commit-sha>` (or `BRANCH=`) when curling `install.sh`. |

Full secrets / privilege model: [deployment.md](./deployment.md) § Secrets & privilege.

## Checklist

- [ ] LXC/VM with Docker
- [ ] `secret.key` generated on host (mounted at runtime, not image build)
- [ ] MongoDB credentials rotated from dev defaults
- [ ] Immich album + API key configured
- [ ] TLS reverse proxy in front of :8080
- [ ] Harden review: `jheffmedia --postinstall --security` (bind/CORS/firewall)
- [ ] `curl /api/media/feed` returns items (Immich path confirmed when not using static demo)
- [ ] Homepage loads styled UI + photo grid
