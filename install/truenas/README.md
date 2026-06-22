# TrueNAS install

**TrueNAS SCALE** (Docker available). Shared stack in [`../common/`](../common/).

## Option A — shell install

Root shell on SCALE:

```bash
bash -c "$(curl -fsSL https://raw.githubusercontent.com/jjheffernan/jheff.media-site/dev/install/truenas/install.sh)"
```

Default install dir: `/mnt/.ix-apps/jheffmedia-site`. Prefer a dataset path:

```bash
INSTALL_DIR=/mnt/tank/apps/site bash -c "$(curl -fsSL .../install/truenas/install.sh)"
```

## Option B — Custom App (manual)

1. Clone repo to a dataset: `git clone -b dev … /mnt/tank/apps/jheffmedia-site`
2. Copy or generate `install/common/.env.production` (run baremetal install once on a staging host, or copy from `install/common/lib/` guided defaults)
3. In TrueNAS **Apps → Discover Custom App**, point compose at `install/common/docker-compose.yml` with project context `/mnt/tank/apps/jheffmedia-site/install/common`

Compose build context is repo root (`../..` from `install/common`).

## Files

```
install/truenas/
└── install.sh    # SCALE-oriented entry — logic in ../common/
```
