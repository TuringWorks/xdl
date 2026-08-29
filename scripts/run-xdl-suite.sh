#!/usr/bin/env bash
# Run the XDL language suite against a known-passing allowlist.
#
# Why an allowlist: of the .xdl programs under tests/, most do not currently run
# to completion (parse errors and unimplemented built-ins), and two never
# terminate. Running `tests/*.xdl` would be red on arrival and would hang, which
# teaches everyone to ignore the job. `tests/passing.txt` is the ratchet instead:
# these scripts pass today, and CI fails if one of them stops passing.
#
# When you make a currently-failing script pass, ADD IT TO tests/passing.txt in
# the same commit. That is how the list grows.
#
# Usage:
#   scripts/run-xdl-suite.sh                  # run the allowlist
#   scripts/run-xdl-suite.sh --survey         # run every tests/*.xdl and report
#
# Each script gets a wall-clock limit (XDL_SUITE_TIMEOUT, default 30s), because
# a hung interpreter is a failure, not a pause.

set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.." || exit 2

XDL_BIN="${XDL_BIN:-./target/release/xdl}"
LIMIT="${XDL_SUITE_TIMEOUT:-30}"

if [ ! -x "$XDL_BIN" ]; then
  echo "no interpreter at $XDL_BIN — run: cargo build --release -p xdl-cli" >&2
  exit 2
fi

# `timeout` is GNU-only; perl's alarm is everywhere. Exit 142 = SIGALRM.
run_one() { perl -e 'alarm shift; exec @ARGV' "$LIMIT" "$XDL_BIN" "$1"; }

survey() {
  local pass=0 fail=0 hung=0
  for f in tests/*.xdl; do
    run_one "$f" >/dev/null 2>&1
    case $? in
      0)       pass=$((pass+1)); echo "pass  $f" ;;
      142|14)  hung=$((hung+1)); echo "HUNG  $f" ;;
      *)       fail=$((fail+1)); echo "fail  $f" ;;
    esac
  done
  echo
  echo "pass=$pass fail=$fail hung=$hung"
}

if [ "${1:-}" = "--survey" ]; then
  survey
  exit 0
fi

list="tests/passing.txt"
[ -f "$list" ] || { echo "missing $list" >&2; exit 2; }

# Graphics scripts write their output into the current directory, so run the
# suite from a scratch dir — otherwise a test run leaves *.png in the repo root.
# See AGENTS.md -> Test Isolation.
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
repo="$PWD"

failed=0
total=0
while read -r f; do
  case "$f" in ''|'#'*) continue ;; esac
  [ -f "$f" ] || { echo "MISSING  $f (listed in $list but not on disk)"; failed=$((failed+1)); continue; }
  total=$((total+1))
  out="$(cd "$work" && perl -e 'alarm shift; exec @ARGV' "$LIMIT" "$repo/$XDL_BIN" "$repo/$f" 2>&1)"
  rc=$?
  case $rc in
    0)      echo "pass  $f" ;;
    142|14) echo "HUNG  $f (>${LIMIT}s)"; failed=$((failed+1)) ;;
    *)      echo "FAIL  $f (exit $rc)"; echo "$out" | tail -5 | sed 's/^/        /'; failed=$((failed+1)) ;;
  esac
done < "$list"

echo
echo "$((total - failed))/$total passing"
[ "$failed" -eq 0 ] || exit 1
