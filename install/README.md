# Install targets

Platform-specific one-line installers in the style of [community-scripts.org](https://community-scripts.org). Shared Docker stack, guided setup, and compose live under **`install/common/`**.

**Start here:** [docs/deployment.md](../docs/deployment.md) · catalog metadata: [`registry.json`](./registry.json)

Production installs should use **`main`**. Scripts still default `BRANCH=dev` for staging; override with `BRANCH=main`.

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
└── truenas/                # TrueNAS SCALE (Docker)
```

## Pick a target

| Target | One-line install | Notes |
|--------|------------------|-------|
| [Proxmox](proxmox/README.md) | `.../install/proxmox/install.sh` | New LXC or existing CT/VM |
| [Bare metal](baremetal/README.md) | `.../install/baremetal/install.sh` | Debian/Ubuntu + Docker |
| [TrueNAS](truenas/README.md) | `.../install/truenas/install.sh` | SCALE shell or Custom App |

Replace `...` with:

`https://raw.githubusercontent.com/jjheffernan/jheff.media-site/main`

## Shared behavior

- **MongoDB** bundled in Docker — no external DB required
- **Photo feed** defaults to `static` demo URLs — no Immich required
- **Guided sections**: repository → site → database → feed → admin
- **Credentials**: `/root/site-install.creds` (or `~/site-install.creds`)
- **Env file**: `{INSTALL_DIR}/install/common/.env.production`
- **Service**: `systemctl restart site`

Unattended:

```bash
UNATTENDED=yes MONGO_PASSWORD='secret' BRANCH=main \
  bash -c "$(curl -fsSL https://raw.githubusercontent.com/jjheffernan/jheff.media-site/main/install/<target>/install.sh)"
```

## Shell variables

| Variable | Default | Purpose |
|----------|---------|---------|
| `REPO` | GitHub clone URL | Repository to clone |
| `BRANCH` | `dev` | Branch to deploy (`main` for production) |
| `INSTALL_DIR` | `/opt/jheffmedia-site` | Clone target on host |
| `HTTP_PORT` | `8080` | Published port |
| `MONGO_PASSWORD` | generated | MongoDB root password |
| `UNATTENDED` | — | Skip whiptail when `yes` |
| `MODE` | `auto` | Proxmox only: `existing` skips LXC create |

Override defaults in `install/common/lib/defaults.sh` or export before running.

## Adding a new target

1. Create `install/<platform>/install.sh` — bootstrap + call `install/common/run-install.sh`
2. Add platform-only files (e.g. Proxmox `ct/` scripts) under that directory
3. Document in `install/<platform>/README.md` and add the target to [`registry.json`](./registry.json)

Do **not** duplicate compose or deploy logic — extend `install/common/`.
