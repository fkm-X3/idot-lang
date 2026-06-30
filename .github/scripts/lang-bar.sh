#!/usr/bin/env bash
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

README="README.md"

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

cat > /tmp/lang-bar.html <<EOF
### Project Language Breakdown

<table>
  <tr>
    <td width="${idot_pct}%" bgcolor="#8510d8">&nbsp;</td>
    <td width="${rust_pct}%" bgcolor="#dea584">&nbsp;</td>
  </tr>
  <tr>
    <td align="center"><b>Idot</b> ${idot_pct}%</td>
    <td align="center"><b>Rust</b> ${rust_pct}%</td>
  </tr>
</table>

<!-- LANG_BAR_END -->
EOF

awk '
/<!-- LANG_BAR_START -->/ {
  print
  while ((getline line < "/tmp/lang-bar.html") > 0) print line
  skip = 1
  next
}
/<!-- LANG_BAR_END -->/ {
  skip = 0
  next
}
!skip { print }
' "$README" > "${README}.tmp" && mv "${README}.tmp" "$README"

rm -f /tmp/lang-bar.html

echo "Updated language breakdown: Idot ${idot_pct}%, Rust ${rust_pct}%"
