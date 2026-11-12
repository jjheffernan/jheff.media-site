# Install targets

Platform-specific one-line installers in the style of [community-scripts.org](https://community-scripts.org). Shared Docker stack, guided setup, and compose live under **`install/common/`**.

**Start here:** [docs/deployment.md](../docs/deployment.md) · catalog metadata: [`registry.json`](./registry.json)

Production installs default to **`main`**. Non-prod channel: **`testing`** (`BRANCH=testing` or day-2 `jheffmedia --update -d`).

## Layout

```
install/
├── registry.json           # app + target catalog (resources, one-liners, tags)
├── common/                 # shared — compose, env, guided setup, deploy
│   ├── docker-compose.yml
│   ├── run-install.sh
│   └── lib/
│       ├── defaults.sh
│       ├── guided-config.sh
│       └── deploy.sh
├── proxmox/                # Proxmox VE (community-scripts build.func)
├── baremetal/              # generic Linux host
├── truenas/                # TrueNAS SCALE (Docker)
└── coolify/                # Coolify Dockerfile-only + local git mirror
```

## Pick a target

| Target | One-line install | Notes |
|--------|------------------|-------|
| [Proxmox](proxmox/README.md) | `.../install/proxmox/install.sh` | New LXC or existing CT/VM |
| [Bare metal](baremetal/README.md) | `.../install/baremetal/install.sh` | Debian/Ubuntu + Docker |
| [TrueNAS](truenas/README.md) | `.../install/truenas/install.sh` | SCALE shell or Custom App |
| [Coolify](coolify/README.md) | — (Dockerfile + [`coolify/mirror.sh`](coolify/mirror.sh)) | Local mirror; Mongo separate |

Replace `...` with:

`https://raw.githubusercontent.com/jjheffernan/jheff.media-site/main`

## Shared behavior

- **MongoDB** bundled in Docker — no external DB required
- **Photo feed** defaults to `static` demo URLs — no Immich required
- **Guided sections**: repository → site → database → feed → admin
- **Admin**: `ADMIN_EMAILS` always set (prompt, env, or generated `admin@….local`); sign up with that email — no seed password
- **Install dir**: `/opt/jheffmedia-site` owned by service user `jheffmedia` (uid 10001)
- **Secrets**: `{INSTALL_DIR}/install/common/.env.production` (0600) + `backend/src/secret.key` (0400) — no `/root/*.creds`
- **Service**: `site.service` runs as `jheffmedia` + `docker` group; app container is non-root
- **Postinstall**: `jheffmedia --postinstall` · paths again: `jheffmedia --postinstall --show-creds`

Unattended:

```bash
UNATTENDED=yes MONGO_PASSWORD='change-me' BRANCH=main \
  bash -c "$(curl -fsSL https://raw.githubusercontent.com/jjheffernan/jheff.media-site/main/install/<target>/install.sh)"
```

## Shell variables

| Variable | Default | Purpose |
|----------|---------|---------|
| `REPO` | GitHub clone URL | Repository to clone |
| `BRANCH` | `main` | Branch to deploy (`testing` for non-prod) |
| `INSTALL_DIR` | `/opt/jheffmedia-site` | Clone target on host |
| `SERVICE_USER` | `jheffmedia` | Host owner + systemd user |
| `SERVICE_UID` | `10001` | Matches app container UID |
| `HTTP_PORT` | `8080` | Published port |
| `HTTP_BIND` | `0.0.0.0` | Use `127.0.0.1` with a local reverse proxy |
| `PUBLIC_URL` | — | Public site URL → seeds `YEW_FULLSTACK_CORS_ORIGINS` |
| `YEW_FULLSTACK_BUILD_ID` | git SHA on deploy | Release id for Admin → Dev (auto on install/update) |
| `YEW_FULLSTACK_CORS_ORIGINS` | from `PUBLIC_URL` or `http://IP:PORT` | Comma-separated browser origins |
| `MONGO_PASSWORD` | generated | MongoDB root password |
| `ADMIN_EMAILS` | generated if blank | Signup emails granted admin |
| `UNATTENDED` | — | Skip whiptail when `yes` |
| `MODE` | `auto` | Proxmox only: `existing` skips LXC create |

Override defaults in `install/common/lib/defaults.sh` or export before running.

## Update / repair (existing install)

Inside the host/LXC as root:

```bash
jheffmedia --update                   # production: pull main
jheffmedia --update -d                # non-prod: pull testing (-d / --dev / --testing)
jheffmedia --autoupdate enable        # optional daily timer → --update (main); off by default
jheffmedia --autoupdate enable -d --interval weekly
jheffmedia --autoupdate status
jheffmedia --autoupdate disable
jheffmedia --cleanup                  # fix cred leaks / ownership / site.service (keeps data)
jheffmedia --reinstall                # cleanup + rebuild (keeps Mongo volumes)
jheffmedia --reinstall --purge        # also drop project containers/images; confirms volumes
jheffmedia --postinstall              # admin email reminder
jheffmedia --postinstall --show-creds # admin email + secret paths (not values)
```

Use `--cleanup` for leftover `/root/site-install.creds` or root-owned trees; `--reinstall` for broken/partial installs; `--update` for day-2 deploys. Autoupdate writes `install/common/autoupdate.env` (`AUTOUPDATE_CHANNEL=main|testing`) and a systemd timer; logs via `journalctl -u jheffmedia-autoupdate.service`. Admin → Dev → Deploy can trigger `--update` / `--update -d` via `var/deploy` + `jheffmedia-gui-update.path` (no docker.sock in the app). See [docs/deployment.md](../docs/deployment.md).

Curl fallback: `bash -c "$(curl -fsSL https://raw.githubusercontent.com/jjheffernan/jheff.media-site/main/install/update.sh)"`.

## Adding a new target

1. Create `install/<platform>/install.sh` — bootstrap + call `install/common/run-install.sh`
2. Add platform-only files (e.g. Proxmox `ct/` scripts) under that directory
3. Document in `install/<platform>/README.md` and add the target to [`registry.json`](./registry.json)

Do **not** duplicate compose or deploy logic — extend `install/common/`.
