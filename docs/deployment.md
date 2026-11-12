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
7. Runs postinstall once — prints admin signup email (generated if you left admin blank / unattended) and the creds file

Default feed is **`static`** demo images so the homepage works without Immich. After you have an admin account, switch providers in **Admin → Setup** (Mongo `site_config`). Env vars remain bootstrap defaults until Setup is saved.

There is no seeded admin password. Admin is whoever signs up with an address in `ADMIN_EMAILS` (mapped to `YEW_FULLSTACK_ADMIN_EMAILS` in compose).

## After install

```bash
systemctl status site
curl -sS "http://127.0.0.1:8080/" | head
BASE_URL="http://127.0.0.1:8080" scripts/smoke.sh
jheffmedia --postinstall          # admin email reminder
# jheffmedia --postinstall --show-creds   # root: reprint ~/site-install.creds
```

Then:

1. **TLS** — put Caddy/Nginx (or Cloudflare Tunnel) in front of `:8080`. See [deployment-proxmox.md](./deployment-proxmox.md) § reverse proxy.
2. **CORS** — once the public hostname is known, set `YEW_FULLSTACK_CORS_ORIGINS=https://jheff.media` (or your domain) on the app service and restart.
3. **Admin** — open the site, sign up with the printed `ADMIN_EMAILS` address (any password ≥8 chars). That account becomes `admin`. Change the email in `.env.production` later if you generated a bootstrap address.
4. **Feed** — Admin → Setup: pick `none` / `static` / `immich` / `nextcloud` and save. Feed reads Mongo per request. If images look stale after a provider change, run `jheffmedia --restart`.
5. **SMTP (optional)** — contact/booking mail needs `YEW_FULLSTACK_SMTP_*`; contact recipient can also be set in Admin → Setup.

Credentials stay only on the host (`site-install.creds`, chmod 600). Nothing secret is committed to git.

## Updates

Inside the install host/LXC as root:

```bash
jheffmedia --update
```

Also: `jheffmedia --status`, `jheffmedia --restart`. Curl fallback (or older installs):

```bash
bash -c "$(curl -fsSL https://raw.githubusercontent.com/jjheffernan/jheff.media-site/main/install/update.sh)"
```

Keep `backend/src/secret.key` — rotating it logs everyone out. Staging: `jheffmedia --update --branch dev`.

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
