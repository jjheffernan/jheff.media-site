# Coolify — Dockerfile only (+ local git mirror)

Self-hosted Coolify often cannot use GitHub. Use a **local bare mirror** as the git source, then deploy with Coolify’s **Dockerfile** resource (not Docker Compose).

Repo root [`Dockerfile`](../../Dockerfile) builds the Actix + Yew image. **MongoDB is not in that image** — add a Mongo service in Coolify and point `YEW_FULLSTACK_DB_CONNSTR` at it.

## 1. Local mirror (on a host that can fetch the source)

```bash
# From a checkout, or curl the script from raw GitHub once:
MIRROR_DIR=/srv/git/jheff.media-site.git bash install/coolify/mirror.sh
```

Refresh later with the same command (fetch --all).

Point Coolify’s git URL at the mirror, for example:

| Coolify on same host | Coolify elsewhere |
|----------------------|-------------------|
| `file:///srv/git/jheff.media-site.git` | `git@mirror-host:/srv/git/jheff.media-site.git` |

Branch: **`main`**.

## 2. Coolify resource

1. **New Resource → Dockerfile** (not Compose).
2. Repository = mirror URL above · Branch = `main`.
3. **Base Directory:** `/` · **Dockerfile:** `Dockerfile`.
4. **Ports:** `8080`.
5. Persistent storage (optional): mount → `/data/content-uploads`.
6. Paste env from [`env.example`](./env.example). Set a real `YEW_FULLSTACK_JWT_SECRET` and Mongo URL.
7. Deploy. First build needs ~6 GB RAM and several minutes (Rust + Trunk).

## 3. Mongo alongside the app

In Coolify, add a **MongoDB 7** service (or any Mongo). Use the same password in:

`YEW_FULLSTACK_DB_CONNSTR=mongodb://root:<password>@<mongo-hostname>:27017/jheffmedia?authSource=admin`

Put the app and Mongo on the same Coolify network so the hostname resolves.

## 4. Check

- `https://your-domain/` loads  
- Sign up with an address listed in `YEW_FULLSTACK_ADMIN_EMAILS` → that user is admin  

## Files

```
install/coolify/
├── README.md      # this file
├── mirror.sh      # bare git mirror create/update
└── env.example    # Coolify environment variables
```

Compose + bundled Mongo stays under [`../common/`](../common/) for Proxmox / bare metal / TrueNAS.
