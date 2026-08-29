#!/usr/bin/env bash
# Check that relative links and image references in docs/, design-system/, and
# the root markdown files (README, AGENTS, CLAUDE, SOUL) point at something that
# exists. Jekyll publishes docs/ to GitHub Pages,
# where a broken relative link is a 404 nobody sees locally.
#
# Skips: absolute URLs, mailto:, anchors-only (#foo), and Jekyll/Liquid links
# that contain {{ ... }}.
#
# Exit 1 if any target is missing.

set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.." || exit 2

missing=0
checked=0

files=$(find docs design-system -name '*.md' -not -path '*/node_modules/*' 2>/dev/null; ls *.md 2>/dev/null)

for f in $files; do
  dir=$(dirname "$f")
  # [text](target) and ![alt](target) — take the target, drop any "title" part.
  targets=$(grep -oE '\]\([^)]+\)' "$f" 2>/dev/null | sed 's/^](//; s/)$//' | awk '{print $1}')
  for t in $targets; do
    case "$t" in
      http://*|https://*|mailto:*|'#'*|*'{{'*|'') continue ;;
    esac
    # Strip a trailing anchor.
    path="${t%%#*}"
    [ -n "$path" ] || continue
    checked=$((checked + 1))
    # Root-relative links resolve against the site baseurl (/xdl), not the repo.
    case "$path" in
      /xdl/*) resolved="docs/${path#/xdl/}" ;;
      /*)     resolved="docs${path}" ;;
      *)      resolved="$dir/$path" ;;
    esac
    if [ ! -e "$resolved" ] && [ ! -e "${resolved%.html}.md" ] && [ ! -e "$resolved/index.md" ]; then
      echo "BROKEN  $f -> $t"
      missing=$((missing + 1))
    fi
  done
done

echo
echo "checked $checked relative links, $missing broken"
[ "$missing" -eq 0 ] || exit 1

# ---------------------------------------------------------------------------
# Anchor checking. A link to a heading that has been renamed resolves to the top
# of the page instead of 404ing, so it is invisible until a reader follows it.
# AGENTS.md and CLAUDE.md cross-reference heavily, which is exactly where this
# rots. Needs python3; skipped (loudly) without it.
# ---------------------------------------------------------------------------
if ! command -v python3 >/dev/null 2>&1; then
  echo "note: python3 not found — anchor checking skipped"
  exit 0
fi

python3 - <<'PYEOF'
import os, re, sys, glob

def slug(heading: str) -> str:
    """GitHub's heading-to-anchor rule: lowercase, drop punctuation that is not a
    hyphen or a space, then spaces -> hyphens. Em dashes vanish, leaving the two
    spaces around them as two hyphens."""
    s = heading.strip().lower()
    s = re.sub(r'`([^`]*)`', r'\1', s)            # inline code keeps its text
    s = re.sub(r'\[([^\]]*)\]\([^)]*\)', r'\1', s)  # links keep their text
    s = re.sub(r'[^\w\s-]', '', s, flags=re.UNICODE)
    return s.replace(' ', '-')

def headings(path):
    out = set()
    try:
        with open(path, encoding='utf-8') as fh:
            fenced = False
            for line in fh:
                if line.lstrip().startswith('```'):
                    fenced = not fenced
                    continue
                if fenced:
                    continue
                m = re.match(r'\s{0,3}(#{1,6})\s+(.*?)\s*#*\s*$', line)
                if m:
                    out.add(slug(m.group(2)))
    except OSError:
        return None
    return out

files = sorted(
    glob.glob('docs/**/*.md', recursive=True)
    + glob.glob('design-system/**/*.md', recursive=True)
    + glob.glob('*.md')
)

cache, broken, checked = {}, 0, 0
link_re = re.compile(r'\]\(([^)\s]+)(?:\s+"[^"]*")?\)')

for f in files:
    base = os.path.dirname(f)
    with open(f, encoding='utf-8') as fh:
        text = fh.read()
    for target in link_re.findall(text):
        if '#' not in target:
            continue
        if target.startswith(('http://', 'https://', 'mailto:')) or '{{' in target:
            continue
        path_part, _, anchor = target.partition('#')
        if not anchor:
            continue
        target_file = f if path_part == '' else (
            os.path.join('docs', path_part[len('/xdl/'):]) if path_part.startswith('/xdl/')
            else os.path.join('docs', path_part.lstrip('/')) if path_part.startswith('/')
            else os.path.normpath(os.path.join(base, path_part))
        )
        if target_file not in cache:
            cache[target_file] = headings(target_file)
        hs = cache[target_file]
        if hs is None:
            continue  # the path check above already reported a missing file
        checked += 1
        if anchor.lower() not in hs:
            print(f'BROKEN ANCHOR  {f} -> {target}')
            broken += 1

print()
print(f'checked {checked} anchors, {broken} broken')
sys.exit(1 if broken else 0)
PYEOF
