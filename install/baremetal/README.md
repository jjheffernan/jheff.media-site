# Bare metal install

Generic **Debian/Ubuntu** host with Docker. Uses shared stack in [`../common/`](../common/).

## One-line install

Run as **root**:

```bash
bash -c "$(curl -fsSL https://raw.githubusercontent.com/jjheffernan/jheff.media-site/dev/install/baremetal/install.sh)"
```

Default install directory: `/opt/jheffmedia-site`. Override:

```bash
INSTALL_DIR=/srv/site bash -c "$(curl -fsSL .../install/baremetal/install.sh)"
```

## After install

```bash
systemctl status site
curl -s "http://127.0.0.1:8080/api/feed" | head
```

Put Caddy/Nginx in front for TLS — [docs/deployment-proxmox.md](../../docs/deployment-proxmox.md) § reverse proxy applies to any Docker host.

## Files

```
install/baremetal/
└── install.sh    # entry only — logic in ../common/
```
