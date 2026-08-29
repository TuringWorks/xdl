#!/usr/bin/env bash
# Compare the three hand-maintained lists of XDL built-in names:
#
#   1. xdl-stdlib/src/lib.rs                    what the interpreter can dispatch
#   2. xdl-lsp/src/symbols.rs                   what editors advertise
#   3. vscode-xdl/syntaxes/xdl.tmLanguage.json  what gets highlighted
#
# Nothing in the build fails when these drift, so this does. The dangerous
# direction is ADVERTISED-BUT-NOT-IMPLEMENTED: the user gets completion for a
# function that does not exist.
#
# Exit codes: 0 = no NEW advertised-but-missing names, 1 = new drift found.
# Known gaps are recorded in scripts/known-drift.txt.
# Implemented-but-not-advertised is reported as a warning, not a failure —
# 754 dispatch arms against a much smaller curated LSP table is expected.
#
# See AGENTS.md -> Traps a Build Cannot Catch.

set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.." || exit 2

dispatch=$(mktemp) ; lsp=$(mktemp) ; grammar=$(mktemp)
trap 'rm -f "$dispatch" "$lsp" "$grammar"' EXIT

# 1. Dispatch arms: every uppercase string literal to the left of a `=>`,
#    including `"A" | "B" => ...` alias arms.
awk '
  /=>/ {
    lhs = $0
    sub(/=>.*/, "", lhs)
    while (match(lhs, /"[A-Z][A-Z0-9_]*"/)) {
      name = substr(lhs, RSTART + 1, RLENGTH - 2)
      print name
      lhs = substr(lhs, RSTART + RLENGTH)
    }
  }
' xdl-stdlib/src/lib.rs | sort -u > "$dispatch"

# 2. LSP table: tuples are written one field per line, so the name is the first
#    string literal on the line directly after a line that is just "(".
#    Stop at `let sys_vars` — !-prefixed system variables are a separate
#    namespace and are not dispatched as functions.
awk '
  /let sys_vars *= *vec!\[/ { exit }
  { cur = $0; gsub(/^[ \t]+|[ \t]+$/, "", cur) }
  prev == "(" && cur ~ /^"[A-Z][A-Z0-9_]*",$/ {
    print substr(cur, 2, length(cur) - 3)
  }
  { prev = cur }
' xdl-lsp/src/symbols.rs | sort -u > "$lsp"

# 3. TextMate grammar: uppercase words inside the builtin/support match patterns.
if [ -f vscode-xdl/syntaxes/xdl.tmLanguage.json ]; then
  grep -oE '\b[A-Z][A-Z0-9_]{2,}\b' vscode-xdl/syntaxes/xdl.tmLanguage.json \
    | sort -u > "$grammar"
else
  : > "$grammar"
fi

n_dispatch=$(wc -l < "$dispatch" | tr -d ' ')
n_lsp=$(wc -l < "$lsp" | tr -d ' ')

echo "dispatch (xdl-stdlib/src/lib.rs):  $n_dispatch names"
echo "lsp      (xdl-lsp/src/symbols.rs): $n_lsp names"
echo

# Known phantoms are recorded in scripts/known-drift.txt so the set can shrink
# but never silently grow. A gate that is red on arrival teaches everyone to
# ignore it; a ratchet does not.
known=$(mktemp)
trap 'rm -f "$dispatch" "$lsp" "$grammar" "$known"' EXIT
if [ -f scripts/known-drift.txt ]; then
  grep -vE '^\s*(#|$)' scripts/known-drift.txt | tr -d ' \t' | sort -u > "$known"
else
  : > "$known"
fi

phantom=$(comm -13 "$dispatch" "$lsp")
new_phantom=$(printf '%s\n' "$phantom" | grep -v '^$' | comm -23 - "$known")

status=0
if [ -n "$new_phantom" ]; then
  echo "FAIL — NEW names advertised by the LSP but NOT dispatched"
  echo "       (the user gets completion for a function that does not exist):"
  printf '%s\n' "$new_phantom" | sed 's/^/  /'
  echo
  echo "  Fix it, or — only if it is a real known gap — record it in scripts/known-drift.txt"
  echo "  with a reason. See .claude/skills/add-builtin/SKILL.md"
  echo
  status=1
elif [ -n "$phantom" ]; then
  n=$(printf '%s\n' "$phantom" | grep -c .)
  echo "ok — no new phantoms. $n known ones remain (scripts/known-drift.txt):"
  printf '%s\n' "$phantom" | sed 's/^/  /'
  echo
else
  echo "ok — every name the LSP advertises is dispatched."
  echo
fi

# A name in known-drift.txt that is now dispatched should be removed from the
# file, or the ratchet stops ratcheting.
stale=$(comm -12 "$dispatch" "$known")
if [ -n "$stale" ]; then
  echo "warn — these are listed in scripts/known-drift.txt but ARE dispatched now."
  echo "       Delete them from that file:"
  printf '%s\n' "$stale" | sed 's/^/  /'
  echo
fi

hidden=$(comm -23 "$dispatch" "$lsp")
if [ -n "$hidden" ]; then
  count=$(printf '%s\n' "$hidden" | wc -l | tr -d ' ')
  echo "warn — $count dispatched names are missing from the LSP table (no completion, no hover):"
  printf '%s\n' "$hidden" | head -20 | sed 's/^/  /'
  [ "$count" -gt 20 ] && echo "  … and $((count - 20)) more"
  echo
  echo "  Add the ones you touched to xdl-lsp/src/symbols.rs — see .claude/skills/add-builtin/SKILL.md"
  echo
fi

# The uppercase-dispatch trap: dispatch runs name.to_uppercase(), so a match arm
# whose literal contains a lowercase letter can never fire, and never warns.
dead=$(grep -nE '^[[:space:]]*"[^"]*[a-z][^"]*"[[:space:]]*=>' xdl-stdlib/src/lib.rs || true)
if [ -n "$dead" ]; then
  echo "FAIL — dead dispatch arms (dispatch uppercases the name; these can never fire):"
  echo "$dead" | sed 's/^/  /'
  status=1
fi

exit $status
