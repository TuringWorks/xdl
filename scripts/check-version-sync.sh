#!/usr/bin/env bash
# The XDL version is declared in three places that nothing keeps in sync:
#
#   Cargo.toml                        [workspace.package] version  (every crate
#                                     inherits it via version.workspace = true)
#   xdl-chart-viewer/tauri.conf.json  the Tauri app bundle version
#   vscode-xdl/package.json           the VS Code extension version
#
# A user who installs the extension and the app from the same tag should see the
# same number. Exit 1 when they disagree.
#
# See AGENTS.md -> Change-Surface Cookbook -> Version bump.

set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.." || exit 2

ws=$(awk '/^\[workspace\.package\]/{f=1;next} /^\[/{f=0} f && $1=="version"{gsub(/[",]/,"",$3); print $3; exit}' Cargo.toml)
tauri=$(grep -m1 '"version"' xdl-chart-viewer/tauri.conf.json | sed 's/.*: *"\([^"]*\)".*/\1/')
vscode=$(grep -m1 '"version"' vscode-xdl/package.json | sed 's/.*: *"\([^"]*\)".*/\1/')

printf '%-34s %s\n' "Cargo.toml [workspace.package]" "${ws:-<not found>}"
printf '%-34s %s\n' "xdl-chart-viewer/tauri.conf.json" "${tauri:-<not found>}"
printf '%-34s %s\n' "vscode-xdl/package.json" "${vscode:-<not found>}"
echo

status=0
for v in "$ws" "$tauri" "$vscode"; do
  [ -n "$v" ] || { echo "FAIL — a version could not be read."; exit 1; }
done

if [ "$ws" = "$tauri" ] && [ "$ws" = "$vscode" ]; then
  echo "ok — all three agree ($ws)."
else
  echo "FAIL — versions disagree. Bump all three together, plus CHANGELOG.md."
  status=1
fi

# The license is also declared twice and currently disagrees. This is REPORTED
# but does not fail the build: which of the two is correct is a licensing
# decision for the maintainers, not something a script should force. Once it is
# resolved, change `status=1` below back in so it stays resolved.
cargo_license=$(awk '/^\[workspace\.package\]/{f=1;next} /^\[/{f=0} f && $1=="license"{gsub(/[",]/,"",$3); print $3; exit}' Cargo.toml)
license_file=$(head -1 LICENSE 2>/dev/null)
case "$license_file" in
  *MIT*) file_license="MIT" ;;
  *GPL*) file_license="GPL" ;;
  *)     file_license="unknown" ;;
esac
if [ -n "$cargo_license" ] && [ "$file_license" != "unknown" ] \
   && [ "${cargo_license:0:3}" != "${file_license:0:3}" ]; then
  echo "WARN — license mismatch: Cargo.toml says '$cargo_license', LICENSE is $file_license,"
  echo "       and the README badge says MIT. Pick one and make all three agree;"
  echo "       then make this a hard failure so it cannot drift again."
fi

exit $status
