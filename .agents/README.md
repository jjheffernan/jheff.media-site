# `.agents` — instructions and skills

Cross-agent project context for Cursor, Codex, Antigravity, Copilot CLI, and other tools that read `AGENTS.md` or `.agents/`.

**Human behavior docs:** [docs/README.md](../docs/README.md) (architecture, API, auth, backend, frontend).

## Layout

```
.agents/
├── README.md                 ← this file
├── cursor.md                 ← Cursor symlinks, slash skills (/ponytail, /caveman)
├── scripts/
│   └── sync-cursor-skills.sh ← refresh .cursor/skills/ symlinks
├── rules/                    ← conventions (short; details in docs/)
│   ├── project.md            ← repo-wide
│   ├── backend.md            ← Actix / MongoDB / JWT
│   └── frontend.md           ← Yew / Trunk / WASM
└── skills/                   ← workflows (read or invoke when relevant)
    ├── caveman/SKILL.md      ← /caveman — terse communication
    ├── dev-server/SKILL.md
    ├── ponytail/SKILL.md     ← /ponytail — lazy senior dev mode
    ├── rust-fullstack/SKILL.md
    └── yew-patterns/SKILL.md
```

## How agents should use this

1. Start with root **[AGENTS.md](../AGENTS.md)** — index only, no duplicate long-form content.
2. Read **[docs/README.md](../docs/README.md)** for detailed behavior (auth flow, API shapes, routing).
3. Apply **rules** for every change in matching paths.
4. Open a **skill** when doing that task, or invoke slash skills in Cursor (`/ponytail`, `/caveman`).

## Cursor

Cursor cannot read `.agents/skills/` directly. Symlinks under `.cursor/skills/` point here. See **[cursor.md](cursor.md)**. After adding a skill: `scripts/sync-cursor-skills.sh` from repo root, or `.agents/scripts/sync-cursor-skills.sh`.

## Adding content

| Add | Where |
|-----|-------|
| New convention | `.agents/rules/<area>.md` |
| New workflow | `.agents/skills/<name>/SKILL.md` then run `sync-cursor-skills.sh` |
| Behavior / API docs | `docs/` — update [docs/README.md](../docs/README.md) index |

Keep rules short. Put long troubleshooting and API tables in `docs/`.

## Rules vs skills

| | Rules | Skills |
|---|-------|--------|
| **Format** | Plain markdown | `SKILL.md` with YAML frontmatter |
| **Scope** | Always apply to matching work | On demand or slash-invoked |
| **Examples** | `backend.md`, `frontend.md` | `dev-server`, `ponytail`, `caveman` |

Do not copy rule content into skills or `AGENTS.md` — link instead.
