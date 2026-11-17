# jheff.media

Self-hosted **automotive photography** management — shoots, galleries, and client delivery. Rust fullstack (Yew WASM + Actix + MongoDB); photos stay in your Immich (or Nextcloud / static demo).

**Beta / personal-staging.** Built for a Proxmox LXC you operate yourself — not client-production-ready yet. Checklist: [docs/beta-readiness.md](docs/beta-readiness.md).

## Install

On a **Proxmox host** (creates an LXC) or inside an existing CT/VM as root:

```bash
bash -c "$(curl -fsSL https://raw.githubusercontent.com/jjheffernan/jheff.media-site/main/install/proxmox/install.sh)"
```

Then on the CT:

```bash
jheffmedia --postinstall              # admin signup email
jheffmedia --postinstall --security   # bind / CORS / TLS / firewall checklist
```

1. Open the site and **sign up** with the printed `ADMIN_EMAILS` address (any password ≥8 chars).
2. **Admin → Setup** — Immich URL, API key, album (or keep the static demo feed).

Harden details: [docs/deployment-proxmox.md](docs/deployment-proxmox.md) (security / TLS). Other targets (bare metal, TrueNAS, Coolify): [install/README.md](install/README.md). Full deploy guide: [docs/deployment.md](docs/deployment.md).

## Channels

| Branch | Use |
|--------|-----|
| `main` | Production installs and `jheffmedia --update` |
| `testing` | Staging — `BRANCH=testing` at install, or `jheffmedia --update -d` |
| `dev` | Day-to-day feature work (merge to `main` when shipping) |

## Local development

```bash
scripts/dev.sh setup   # once
scripts/dev.sh start   # http://localhost:8080
```

Dev stack and troubleshooting: [docs/deployment-dev.md](docs/deployment-dev.md) · [docs/development.md](docs/development.md). Script reference: [scripts/README.md](scripts/README.md).

## Docs

| Doc | What |
|-----|------|
| [docs/README.md](docs/README.md) | Index of all guides |
| [install/README.md](install/README.md) | Install targets and day-2 CLI |
| [docs/beta-readiness.md](docs/beta-readiness.md) | Beta exit criteria |
| [docs/TODO.md](docs/TODO.md) | Open backlog |
| [docs/deployment-proxmox.md](docs/deployment-proxmox.md) | Proxmox layout, Immich, security |

Stack in one line: Yew 0.23 + Trunk · Actix 4 + MongoDB 3. Agent rules: [AGENTS.md](AGENTS.md) → [.agents/](.agents/).

## License

MIT / Apache-2.0 (see `LICENSE_MIT`, `LICENSE_APACHE`).
