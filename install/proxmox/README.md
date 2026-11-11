# Proxmox install

[community-scripts](https://community-scripts.org/docs) Core layout for Proxmox VE — `ct/` creates the LXC, `install/` runs inside it. Shared stack lives in [`../common/`](../common/).

Starting guide: [docs/deployment.md](../../docs/deployment.md) · catalog: [`../registry.json`](../registry.json)

## One-line install (recommended)

**Proxmox host** (creates Debian 13 LXC, 2 CPU, 6 GB RAM, 24 GB disk):

```bash
bash -c "$(curl -fsSL https://raw.githubusercontent.com/jjheffernan/jheff.media-site/main/install/proxmox/install.sh)"
```

This uses [community-scripts Core](https://github.com/community-scripts/core) (`core/build.func`). The old `ProxmoxVE/misc/build.func` URL is gone and will 404.

## Alternative — community-scripts template, then install

If you prefer their [All Templates](https://community-scripts.org/scripts/all-templates?from=scripts&fromQ=template) / Debian LXC first:

**1. On the Proxmox host**, create a Debian LXC (pick nesting / Docker-friendly options when prompted):

```bash
bash -c "$(curl -fsSL https://raw.githubusercontent.com/community-scripts/ProxmoxVE/main/ct/debian.sh)"
```

Or browse templates:

```bash
bash -c "$(curl -fsSL https://raw.githubusercontent.com/community-scripts/ProxmoxVE/main/tools/addon/all-templates.sh)"
```

**2. Inside that LXC as root**, install the site:

```bash
MODE=existing bash -c "$(curl -fsSL https://raw.githubusercontent.com/jjheffernan/jheff.media-site/main/install/proxmox/install.sh)"
```

## Existing LXC/VM (any Debian/Ubuntu with nesting)

```bash
MODE=existing bash -c "$(curl -fsSL https://raw.githubusercontent.com/jjheffernan/jheff.media-site/main/install/proxmox/install.sh)"
```

Staging / `dev` branch:

```bash
BRANCH=dev bash -c "$(curl -fsSL https://raw.githubusercontent.com/jjheffernan/jheff.media-site/dev/install/proxmox/install.sh)"
```

## Proxmox-only files

```
install/proxmox/
├── install.sh                      # entry — Proxmox host vs existing CT
├── ct/jheffmedia.sh                # host-side LXC create (Core build.func)
├── install/jheffmedia-install.sh   # in-LXC install (Core install.func)
└── container-install.sh            # shim for older links
```

Shared compose, env, and guided setup: [`../common/`](../common/).

## Updates

Re-run the CT script on the Proxmox host and choose update, or inside the container:

```bash
cd /opt/jheffmedia-site && git pull origin main
cd install/common && docker compose --env-file .env.production up -d --build
systemctl restart site
```

See also [docs/deployment-proxmox.md](../../docs/deployment-proxmox.md).
