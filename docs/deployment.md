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

1. Clones the repo to **`/opt/jheffmedia-site`** (override with `INSTALL_DIR`)
2. Creates host user **`jheffmedia`** (uid **10001**, `docker` group) and owns the install tree
3. Writes `install/common/.env.production` (0600) — guided prompts, or defaults when unattended
4. Generates a 32-byte JWT secret at `backend/src/secret.key` (0400; mounted read-only at runtime, not baked into the image)
5. Builds and starts the Docker stack (`jheffmedia-*` containers): Actix app (non-root uid 10001) + MongoDB (not published to the host)
6. Installs `site.service` (runs as `jheffmedia` + `docker` group)
7. Runs postinstall once — prints admin signup email; removes any legacy `/root/site-install.creds`

Default feed is **`static`** demo images so the homepage works without Immich. After you have an admin account, switch providers in **Admin → Setup** (Mongo `site_config`). Env vars remain bootstrap defaults until Setup is saved.

There is no seeded admin password. Admin is whoever signs up with an address in `ADMIN_EMAILS` (mapped to `YEW_FULLSTACK_ADMIN_EMAILS` in compose).

## After install

```bash
systemctl status site
curl -sS "http://127.0.0.1:8080/" | head
BASE_URL="http://127.0.0.1:8080" scripts/smoke.sh
jheffmedia --postinstall          # admin email reminder
# jheffmedia --postinstall --show-creds   # admin email + secret paths (not values)
```

Then:

1. **TLS** — put Caddy/Nginx (or Cloudflare Tunnel) in front of `:8080`. See [deployment-proxmox.md](./deployment-proxmox.md) § reverse proxy. For a proxy on the same LXC, set `HTTP_BIND=127.0.0.1` in `.env.production`.
2. **CORS** — once the public hostname is known, set `YEW_FULLSTACK_CORS_ORIGINS=https://jheff.media` (or your domain) on the app service and restart.
3. **Admin** — open the site, sign up with the printed `ADMIN_EMAILS` address (any password ≥8 chars). That account becomes `admin`. Change the email in `.env.production` later if you generated a bootstrap address.
4. **Feed** — Admin → Setup: pick `none` / `static` / `immich` / `nextcloud` and save. Feed reads Mongo per request. If images look stale after a provider change, run `jheffmedia --restart`.
5. **SMTP (optional)** — contact/booking mail needs `YEW_FULLSTACK_SMTP_*`; contact recipient can also be set in Admin → Setup.
6. **Admin Dev mode (optional)** — on the live site, signed-in admins can open **Admin → Dev** and toggle a browser-local troubleshooting UI (build/version, API base, feed provider name, probe links to `/metrics` + feed/schedule, recent failed-fetch statuses, copyable masked diagnostics JSON). This is not the install/community-scripts `DEV_MODE` flag, does not change server log verbosity, and never includes JWT/passwords/API keys. Non-admins never see the controls.

### Secrets & privilege (threat model)

| What | Where | Perms |
|------|-------|-------|
| Mongo / Immich / `ADMIN_EMAILS` | `/opt/jheffmedia-site/install/common/.env.production` | `0600` `jheffmedia` |
| JWT signing key | `/opt/jheffmedia-site/backend/src/secret.key` | `0400` `jheffmedia` |
| App process | container uid `10001` | no docker.sock, `cap_drop: ALL`, `no-new-privileges` |
| Day-2 CLI | root **or** `jheffmedia` (docker group) | docker group ≈ root — intentional |

Root on the LXC can always read env. Goal: no world-readable dumps, no `/root/*.creds`, no secrets in the image or git. Nothing secret is committed to git.

**Migrate an older root-owned install** (or leftover `/root/site-install.creds`):

```bash
# as root inside the LXC
jheffmedia --cleanup          # preview: --dry-run
jheffmedia --postinstall --show-creds
# if the stack itself is broken, prefer --reinstall (keeps DB) over hand-editing
```

### `--cleanup` vs `--reinstall` vs `--update`

| Command | Use when | Touches data? |
|---------|----------|---------------|
| `--update` | Routine pull on a **working** install | No — rebuild/restart only |
| `--cleanup` | Cred dumps, wrong owner, bad `site.service` User=, exited junk containers | No — never deletes Mongo volumes or `.env.production` |
| `--reinstall` | Partial/broken install; need cleanup + pull/rebuild | Keeps volumes by default; `--purge` drops app containers/images and **asks** before volumes |

```bash
jheffmedia --cleanup --dry-run
jheffmedia --cleanup -y
jheffmedia --reinstall            # keep DB/uploads
jheffmedia --reinstall --purge    # tear project containers/images; confirm volumes
```

## Updates

Inside the install host/LXC as root or `jheffmedia`:

```bash
jheffmedia --update
```

Also: `jheffmedia --status`, `jheffmedia --restart`, `jheffmedia --cleanup`, `jheffmedia --reinstall`.

### Signup fails with database / MONGO errors

Fresh Proxmox/Docker installs create admin by signing up with `ADMIN_EMAILS` (often a generated `admin@…….local`). If signup fails:

```bash
jheffmedia --status
# Expect site active and both containers healthy:
#   jheffmedia-application   … Up
#   jheffmedia-database      … Up (healthy)

cd /opt/jheffmedia-site/install/common   # or your INSTALL_DIR
docker compose -f docker-compose.yml --env-file .env.production ps
docker compose -f docker-compose.yml --env-file .env.production logs --tail=80 db application
```

Typical ops causes: `db` not healthy yet, wrong `MONGO_PASSWORD` / `DB_CONNSTR` vs an existing mongo volume, or app started before mongod finished init. Fix: wait for healthy db, or `jheffmedia --restart`. Password mismatch after a reinstall usually needs aligning `.env.production` with the volume (or a deliberate volume reset). Backend logs include the real Mongo driver error; the UI shows a short human message only.

Curl fallback (or older installs):

```bash
bash -c "$(curl -fsSL https://raw.githubusercontent.com/jjheffernan/jheff.media-site/main/install/update.sh)"
```

Keep `backend/src/secret.key` — rotating it logs everyone out. Staging: `jheffmedia --update --branch dev`.

## Process model (single replica)

Production is **one** `application` container (Actix is multi-threaded inside that process). Do not scale replicas horizontally unless uploads share storage: shoot/gallery files live on the durable named volume `jheffmedia-content-uploads` mounted at `/data/content-uploads` (`YEW_FULLSTACK_CONTENT_UPLOAD_DIR`). Mongo data is the separate `jheffmedia-mongo` volume. Compose healthchecks: Mongo `mongosh` ping, app `GET /healthz` (includes a cheap Mongo ping).

Canonical compose file: [`install/common/docker-compose.yml`](../install/common/docker-compose.yml). `scripts/docker-compose.yml` only `include`s that file for older commands — same volume keys, so existing installs are not orphaned.

JWT: the image never bakes `secret.key`. Runtime loads `YEW_FULLSTACK_JWT_SECRET_FILE` (compose mounts the host file at `/run/secrets/jwt_secret`) or hex `YEW_FULLSTACK_JWT_SECRET`. Optional release id: set `YEW_FULLSTACK_BUILD_ID` (git SHA) as a compose build-arg/env; Admin → Dev / `GET /api/admin/diagnostics` shows `buildId`.

`GET /metrics` is unauthenticated Prometheus text on the app port. On a shared LXC, prefer `HTTP_BIND=127.0.0.1` (or scrape only via a reverse-proxy allowlist) so metrics are not world-reachable.

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
