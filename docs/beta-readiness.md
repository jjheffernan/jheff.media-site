# Beta readiness

Definition of **beta** for jheff.media-site, plus the exit checklist before a first public beta tag.

## What “beta” means here

Self-hostable on a **Proxmox LXC** (or equivalent Docker host) for **personal / staging** use:

| In scope | Out of scope (deferred) |
|----------|-------------------------|
| Install from `main`, day-2 `jheffmedia --update` | Client-production hardening / pen test |
| Immich **optional** (static demo feed works; Admin → Setup when ready) | Social **OAuth** (paste keys/tokens only) |
| Print requests **unpaid** + admin status | Card payment / checkout charging |
| JWT + 2FA, CORS, TLS via reverse proxy | Authenticated `/metrics` |

This is not a claim of client-production readiness. Scores and caveats: [readiness-rubric.md](./readiness-rubric.md). Open work: [TODO.md](./TODO.md).

## Exit criteria

Complete both columns before tagging. Unchecked items block the public tag; this doc does **not** create the tag.

### Operator (deployed host)

- [ ] Install from **`main`** ([deployment.md](./deployment.md) / [deployment-proxmox.md](./deployment-proxmox.md) one-liner or compose)
- [ ] Follow **`jheffmedia --postinstall`** (admin signup reminder) and **`jheffmedia --postinstall --security`** (harden report — bind, CORS presence, firewall/SSH notes; no secret values). See [install/postinstall.sh](../install/postinstall.sh) · [Security hardening (LXC)](./deployment-proxmox.md#security-hardening-lxc)
- [ ] **CORS + TLS** documented and applied — `YEW_FULLSTACK_CORS_ORIGINS` / `PUBLIC_URL` for the real HTTPS origin; reverse proxy in front of `:8080` ([deployment-proxmox.md § Reverse proxy](./deployment-proxmox.md#4-reverse-proxy-tls); prefer `HTTP_BIND=127.0.0.1` when the proxy is on the same LXC)
- [ ] **Smoke** against the deploy:
  - [ ] `GET /` (homepage)
  - [ ] `GET /healthz`
  - [ ] Login (admin signup with `ADMIN_EMAILS` address if first boot)
  - [ ] `GET /api/media/feed`
  - [ ] Immich (when available): Admin → Setup → **Test Immich** (or `scripts/smoke.sh` with Immich env — operator-owned; no keys in git)
- [ ] **Secrets**: `install/common/.env.production` mode **0600**, `backend/src/secret.key` mode **0400**, both backed up off-box ([deployment.md § Secrets](./deployment.md#secrets--privilege-threat-model) · [Backups](./deployment-proxmox.md#7-backups))

### Maintainer (repo / CI)

- [ ] **CI green on `main`**: Trunk build, backend build, backend + frontend host tests, backend Clippy (`-D warnings`) — [`.github/workflows/rust.yml`](../.github/workflows/rust.yml)
- [ ] Known limitations accepted for this beta (below)
- [ ] Rubric re-checked: [readiness-rubric.md](./readiness-rubric.md)

## Known limitations (beta)

- No social **OAuth** — Admin → Links paste keys/tokens only
- **Print payment** shelved — unpaid requests + admin status only
- **`GET /metrics`** is unauthenticated Prometheus text on the app port — restrict at the reverse proxy or keep loopback bind ([`--security` report](../install/postinstall.sh))
- **Immich E2E** is operator-owned — needs a live Immich; not asserted in CI ([TODO.md](./TODO.md#hardening--quality))

## Future step (not this pass)

After the checklist above is green:

```text
git tag v0.1.0-beta.1
# publish / announce when ready
```

Do not cut the tag from this documentation pass alone.

## Related

- [deployment-proxmox.md](./deployment-proxmox.md) — Proxmox/LXC detail · [Security hardening (LXC)](./deployment-proxmox.md#security-hardening-lxc)
- [deployment.md](./deployment.md) — production start · postinstall · secrets
- [install/postinstall.sh](../install/postinstall.sh) — `--postinstall` / `--security`
- [readiness-rubric.md](./readiness-rubric.md) — category scores
- [TODO.md](./TODO.md) — backlog and deferred product
