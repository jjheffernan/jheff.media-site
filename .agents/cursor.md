# Cursor integration

Cursor discovers Agent Skills from `.cursor/skills/` only. This repo keeps **all skill content** in `.agents/skills/` and mirrors it with symlinks — no duplicated `SKILL.md` files under `.cursor/`.

## Layout

```
.agents/skills/<name>/SKILL.md   ← canonical (edit here)
.cursor/skills/<name>            ← symlink → ../../.agents/skills/<name>
```

Rules and conventions live in `.agents/rules/` (markdown, not Cursor `.mdc`). Invoke behavior skills with `/ponytail`, `/caveman`, etc. in Agent chat.

## After adding a skill

```bash
.agents/scripts/sync-cursor-skills.sh
```

Creates or refreshes symlinks for every directory under `.agents/skills/`.

## Enable in Cursor

1. **Settings → Rules** — turn on **Agent Skills**
2. Reload the window if new skills do not appear in the `/` menu

## Slash skills

| Skill | Invoke | Purpose |
|-------|--------|---------|
| `ponytail` | `/ponytail [lite\|full\|ultra]` | Lazy senior dev — YAGNI, minimal code |
| `caveman` | `/caveman [lite\|full\|ultra\|wenyan-*]` | Terse replies, ~75% fewer tokens |

Other skills (`dev-server`, `rust-fullstack`, `yew-patterns`) auto-apply when relevant or can be typed as `/dev-server`, etc.
