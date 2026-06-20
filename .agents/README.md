# `.agents` — instructions and skills

Cross-agent project context for Cursor, Codex, Antigravity, Copilot CLI, and other tools that read `AGENTS.md` or `.agents/`.

**Human behavior docs:** [docs/README.md](../docs/README.md) (architecture, API, auth, backend, frontend).

## Layout

```
.agents/
├── README.md                 ← this file
├── rules/                    ← always-on or path-specific instructions (markdown)
│   ├── project.md            ← repo-wide conventions
│   ├── backend.md            ← Actix / MongoDB / JWT
│   └── frontend.md           ← Yew / WASM / webpack
└── skills/                   ← task workflows (read when relevant)
    ├── dev-server/SKILL.md
    └── rust-fullstack/SKILL.md
```

## How agents should use this

1. Start with root **[AGENTS.md](../AGENTS.md)** for orientation.
2. Read **[docs/README.md](../docs/README.md)** for detailed behavior (auth flow, API shapes, routing).
3. Apply **rules** for every change in matching paths.
4. Open a **skill** only when doing that task (dev server, fullstack build, etc.).

## Cursor-specific

- `.cursor/rules/*.mdc` — caveman + ponytail
- `.cursor/skills/` — symlinks to `.agents/skills/`

## Adding content

| Add | Where |
|-----|-------|
| New convention | `.agents/rules/<area>.md` |
| New workflow | `.agents/skills/<name>/SKILL.md` |
| Behavior / API docs | `docs/` — update [docs/README.md](../docs/README.md) index |

Keep rules short. Put long troubleshooting and API tables in `docs/`.
