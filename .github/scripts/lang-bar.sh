#!/usr/bin/env bash
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

SVG="assets/lang-bar.svg"

count_loc() {
  local total=0
  for ext in "$@"; do
    while IFS= read -r file; do
      lines=$(wc -l < "$file" 2>/dev/null || echo 0)
      total=$((total + lines))
    done < <(git ls-files "*${ext}" 2>/dev/null)
  done
  echo "$total"
}

IDOT_COUNT=$(count_loc ".ido" ".idot")
RUST_COUNT=$(count_loc ".rs")

TOTAL=$((IDOT_COUNT + RUST_COUNT))

if [ "$TOTAL" -eq 0 ]; then
  echo "No source files found"
  exit 1
fi

idot_pct=$((IDOT_COUNT * 100 / TOTAL))
rust_pct=$((RUST_COUNT * 100 / TOTAL))

diff=$((idot_pct + rust_pct - 100))
if [ "$diff" -ne 0 ]; then
  if [ "$IDOT_COUNT" -ge "$RUST_COUNT" ]; then
    idot_pct=$((idot_pct - diff))
  else
    rust_pct=$((rust_pct - diff))
  fi
fi

idot_w=$((idot_pct * 6))
rust_w=$((rust_pct * 6))

cat > "$SVG" <<EOF
<svg xmlns="http://www.w3.org/2000/svg" width="600" height="32" viewBox="0 0 600 32">
  <rect x="0" y="0" width="600" height="32" rx="4" fill="#e0e0e0"/>
  <rect x="0" y="0" width="${idot_w}" height="32" rx="4" fill="#8510d8"/>
  <rect x="${idot_w}" y="0" width="${rust_w}" height="32" rx="4" fill="#dea584"/>
  <text x="$((idot_w / 2))" y="21" text-anchor="middle" font-family="sans-serif" font-size="12" fill="white" font-weight="bold">Idot ${idot_pct}%</text>
  <text x="$((idot_w + rust_w / 2))" y="21" text-anchor="middle" font-family="sans-serif" font-size="12" fill="orange" font-weight="bold">Rust ${rust_pct}%</text>
</svg>
EOF

echo "Updated $SVG: Idot ${idot_pct}%, Rust ${rust_pct}%"
