# Deploy jheff.media

Starting guide for putting this site on a real host. Local development stays in [deployment-dev.md](./deployment-dev.md).

This project ships [community-scripts](https://community-scripts.org)-style one-liners under [`install/`](../install/README.md). App metadata (resources, tags, install URLs) lives in [`install/registry.json`](../install/registry.json) — the same kind of catalog entry their site uses to browse containers.

## Pick a path

| Host | Install | Notes |
|------|---------|-------|
| **Proxmox VE** | [install/proxmox](../install/proxmox/README.md) | Creates a Debian 12 LXC (2 CPU / 6 GB / 24 GB) or installs into an existing CT |
| **Debian / Ubuntu** | [install/baremetal](../install/baremetal/README.md) | Docker on a plain Linux box |
| **TrueNAS SCALE** | [install/truenas](../install/truenas/README.md) | Shell install or Custom App compose |
| **Coolify** | [install/coolify](../install/coolify/README.md) | Dockerfile-only + local git mirror (no GitHub required) |

Cloudflare Workers is **not** a deploy target. The site is Actix + MongoDB; use Docker on one of the hosts above, then put Cloudflare (or any reverse proxy) in front for DNS/TLS if you want.

## One-line install (Proxmox)

On the Proxmox host as root:

```bash
bash -c "$(curl -fsSL https://raw.githubusercontent.com/jjheffernan/jheff.media-site/main/install/proxmox/install.sh)"
```

That loads [community-scripts Core](https://github.com/community-scripts/core) (`core/build.func`). The old `ProxmoxVE/misc/build.func` path 404s after their Core split — if you see `header_info: command not found`, you hit that stale URL; use the command above from `main`.

Production tracks **`main`**. For staging:

```bash
BRANCH=dev bash -c "$(curl -fsSL https://raw.githubusercontent.com/jjheffernan/jheff.media-site/dev/install/proxmox/install.sh)"
```

### Or: community-scripts template, then install

Create a Debian LXC with their [All Templates](https://community-scripts.org/scripts/all-templates?from=scripts&fromQ=template) / Debian script, then install inside it:

```bash
# On Proxmox host — Debian LXC (enable nesting for Docker)
bash -c "$(curl -fsSL https://raw.githubusercontent.com/community-scripts/ProxmoxVE/main/ct/debian.sh)"

# Inside that LXC as root
MODE=existing bash -c "$(curl -fsSL https://raw.githubusercontent.com/jjheffernan/jheff.media-site/main/install/proxmox/install.sh)"
```

Unattended (creates LXC + installs in one shot via our CT script):

```bash
UNATTENDED=yes MONGO_PASSWORD='choose-a-strong-password' \
  bash -c "$(curl -fsSL https://raw.githubusercontent.com/jjheffernan/jheff.media-site/main/install/proxmox/install.sh)"
```

Bare metal and TrueNAS use the same pattern with `install/baremetal/install.sh` or `install/truenas/install.sh`.

## What the installer does

1. Clones the repo (default `/opt/jheffmedia-site`)
2. Writes `install/common/.env.production` (guided prompts, or defaults when unattended)
3. Generates a 32-byte JWT secret at `backend/src/secret.key` (mounted at runtime, not baked into the image)
4. Builds and starts the Docker stack: Actix app + MongoDB
5. Installs a `site` systemd unit
6. Writes credentials to `/root/site-install.creds` (or `~/site-install.creds`)

Default feed is **`static`** demo images so the homepage works without Immich. Switch to Immich later in `.env.production`.

## After install

```bash
systemctl status site
curl -sS "http://127.0.0.1:8080/" | head
BASE_URL="http://127.0.0.1:8080" scripts/smoke.sh
```

Then:

1. **TLS** — put Caddy/Nginx (or Cloudflare Tunnel) in front of `:8080`. See [deployment-proxmox.md](./deployment-proxmox.md) § reverse proxy.
2. **CORS** — once the public hostname is known, set `YEW_FULLSTACK_CORS_ORIGINS=https://jheff.media` (or your domain) on the app service and restart.
3. **Admin** — set `ADMIN_EMAILS` to your email in `.env.production`, restart, sign up once with that email. That account becomes `admin`.
4. **Feed (optional)** — set `FEED_PROVIDER=immich` plus Immich URL, API key, and album id when ready.
5. **SMTP (optional)** — contact/booking mail needs `YEW_FULLSTACK_SMTP_*` and `CONTACT_EMAIL`.

Credentials and env paths are printed at the end of install and stored in the creds file.

## Updates

```bash
cd /opt/jheffmedia-site
git checkout main && git pull
# Keep backend/src/secret.key — rotating it logs everyone out
cd install/common
docker compose --env-file .env.production up -d --build
systemctl restart site
```

On Proxmox you can also re-run the CT script and choose update.

## Checklist

- [ ] Host with Docker (or Proxmox install that creates the LXC)
- [ ] Installer finished; `systemctl status site` is active
- [ ] `secret.key` exists on the host and is backed up somewhere safe
- [ ] Mongo password is not a docs placeholder
- [ ] TLS (or Tunnel) in front of `:8080`
- [ ] Smoke check against the public URL
- [ ] Admin account created via `ADMIN_EMAILS`
- [ ] Immich or static feed confirmed on the homepage
- [ ] Mongo backup plan (`mongodump` from the `db` container)

## Related

| Doc | When |
|-----|------|
| [`install/registry.json`](../install/registry.json) | Catalog metadata (resources, one-liners, tags) |
| [`install/README.md`](../install/README.md) | All install targets and env overrides |
| [deployment-proxmox.md](./deployment-proxmox.md) | Manual compose, Immich, proxy examples, backups |
| [deployment-dev.md](./deployment-dev.md) | Local Trunk + Actix development |
| [version-matrix.md](./version-matrix.md) | Env var names the container reads |
