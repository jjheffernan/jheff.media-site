# Proxmox install

[community-scripts.org](https://community-scripts.org/docs) style install for Proxmox VE — uses `build.func` for LXC creation and shared logic in [`../common/`](../common/).

## One-line install

**Proxmox host** (creates Debian 12 LXC, 2 CPU, 6 GB RAM, 24 GB disk):

```bash
bash -c "$(curl -fsSL https://raw.githubusercontent.com/jjheffernan/jheff.media-site/dev/install/proxmox/install.sh)"
```

**Existing LXC/VM** (same host or elsewhere):

```bash
bash -c "$(curl -fsSL https://raw.githubusercontent.com/jjheffernan/jheff.media-site/dev/install/proxmox/install.sh)"
```

Force existing-target mode from Proxmox:

```bash
MODE=existing bash -c "$(curl -fsSL https://raw.githubusercontent.com/jjheffernan/jheff.media-site/dev/install/proxmox/install.sh)"
```

## Proxmox-only files

```
install/proxmox/
├── install.sh              # entry — auto-detects Proxmox vs existing CT
├── container-install.sh    # in-LXC install (install.func bootstrap)
└── ct/jheffmedia.sh        # host-side LXC create / update
```

Shared compose, env, and guided setup: [`../common/`](../common/).

## Updates

Re-run CT script on Proxmox host and choose update, or inside the container:

```bash
cd /opt/jheffmedia-site && git pull origin dev
cd install/common && docker compose --env-file .env.production up -d --build
systemctl restart site
```

See also [docs/deployment-proxmox.md](../../docs/deployment-proxmox.md).
