#!/usr/bin/env bash
# Symlink .agents/skills/* → .cursor/skills/* for Cursor discovery.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
AGENTS_SKILLS="$ROOT/.agents/skills"
CURSOR_SKILLS="$ROOT/.cursor/skills"

mkdir -p "$CURSOR_SKILLS"

for skill_dir in "$AGENTS_SKILLS"/*/; do
  [[ -d "$skill_dir" ]] || continue
  name="$(basename "$skill_dir")"
  link="$CURSOR_SKILLS/$name"
  target="../../.agents/skills/$name"

  if [[ -e "$link" && ! -L "$link" ]]; then
    echo "error: $link exists and is not a symlink — remove or move it first" >&2
    exit 1
  fi

  ln -sfn "$target" "$link"
  echo "linked $name"
done

echo "Done. See .agents/cursor.md"
