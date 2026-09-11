#!/usr/bin/env bash
# PostToolUse hook for Edit|Write.
#
# Reads the hook JSON payload on stdin, finds the file that was edited, and runs
# the cheapest check that covers it:
#
#   *.rs   -> cargo check on the OWNING crate (by --manifest-path, so
#             xdl-desktop-viewer, which nothing depends on and --workspace never
#             compiles, is covered too)
#   *.ts   -> tsc --noEmit for vscode-xdl
#   lib.rs -> additionally, the stdlib dispatch lint (lowercase match arms are
#             dead code, because dispatch runs name.to_uppercase())
#
# Output is truncated: a hook is a signal, not a build log.

set -uo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
f="$(jq -r '.tool_input.file_path // empty' 2>/dev/null)"
[ -n "$f" ] || exit 0

# tool_input.file_path is untrusted: cargo resolves --manifest-path by walking up
# from the edited file, and that manifest's build.rs / proc-macros get EXECUTED.
# An out-of-tree path (e.g. /tmp/planted/src/lib.rs above a planted Cargo.toml)
# must never reach cargo. Refuse anything that does not resolve inside the repo.
if command -v realpath >/dev/null 2>&1; then
  f_real="$(realpath "$f" 2>/dev/null)" || exit 0
else
  f_real="$(cd "$(dirname "$f")" 2>/dev/null && pwd)/$(basename "$f")" || exit 0
fi
case "$f_real" in
  "$repo_root"/*) f="$f_real" ;;
  *) exit 0 ;;
esac

case "$f" in
  *.rs) ;;
  *.ts|*.tsx)
    if [ -f "$repo_root/vscode-xdl/tsconfig.json" ]; then
      # Call the locally installed binary directly. Bare `npx tsc` would fetch
      # the registry package "tsc" — a squatter, not the official TypeScript —
      # and run its install code when typescript is missing from node_modules.
      tsc_bin="$repo_root/vscode-xdl/node_modules/.bin/tsc"
      if [ -x "$tsc_bin" ]; then
        "$tsc_bin" --noEmit --project "$repo_root/vscode-xdl/tsconfig.json" 2>&1 | tail -5
      fi
    fi
    exit 0 ;;
  *) exit 0 ;;
esac

# Walk up to the nearest Cargo.toml that declares a [package]. Safe: $f now
# resolves inside $repo_root, so the walk cannot leave the repo (the root
# manifest itself has a [package] section and stops the loop).
d="$(dirname "$f")"
manifest=""
while [ -n "$d" ] && [ "$d" != "/" ] && [ "$d" != "." ]; do
  if [ -f "$d/Cargo.toml" ] && grep -q '^\[package\]' "$d/Cargo.toml"; then
    manifest="$d/Cargo.toml"
    break
  fi
  d="$(dirname "$d")"
done

if [ -z "$manifest" ]; then
  echo "hook: no Cargo package above $f — skipped cargo check"
else
  # --manifest-path (not -p) so xdl-desktop-viewer -- which nothing depends on,
  # so --workspace never compiles it -- is checked too.
  # See AGENTS.md -> Product Matrix.
  cargo check --manifest-path "$manifest" --all-targets 2>&1 | tail -12
fi

# The stdlib dispatch matches on name.to_uppercase(), so any arm whose literal
# contains a lowercase letter can never fire and never warns.
case "$f" in
  */xdl-stdlib/src/lib.rs)
    bad="$(grep -nE '^[[:space:]]*"[^"]*[a-z][^"]*"[[:space:]]*=>' "$f" || true)"
    if [ -n "$bad" ]; then
      echo "hook: DEAD DISPATCH ARMS — dispatch uppercases the name, these can never fire:"
      echo "$bad" | head -10
    fi
    ;;
esac