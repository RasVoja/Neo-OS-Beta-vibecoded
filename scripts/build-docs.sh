#!/bin/sh
# Builds the user guide from Markdown: scripts/build-docs.sh html|pdf|all [en|sr]
# Output goes to build/docs/<lang>/ . Needs pandoc and wkhtmltopdf (free software).
# First version: tested only on the documents in docs/.
set -e
cd "$(dirname "$0")/.."
what=${1:-all}; lang=${2:-sr}
out=build/docs/$lang; mkdir -p "$out"
if [ "$lang" = sr ]; then files=$(ls docs/*.sr.md); else files=$(ls docs/*.md | grep -v '\.sr\.md$'); fi
css=docs/assets/guide.css
if [ "$what" = html ] || [ "$what" = all ]; then
  for f in $files; do
    b=$(basename "$f" .md)
    pandoc "$f" -f gfm -t html5 -s --metadata pagetitle="$b" -c "../../../$css" -o "$out/$b.html"
  done
  # relative links between pages: .md -> .html
  sed -i 's/\.md"/.html"/g; s/\.md#/.html#/g' "$out"/*.html
  { echo '<!doctype html><meta charset="utf-8"><title>Index</title><link rel="stylesheet" href="../../../'"$css"'"><h1>Index</h1><ul>'
    for h in "$out"/*.html; do b=$(basename "$h"); [ "$b" = index.html ] || echo "<li><a href=\"$b\">${b%.html}</a></li>"; done
    echo '</ul>'; } > "$out/index.html"
fi
if [ "$what" = pdf ] || [ "$what" = all ]; then
  pandoc $files -f gfm -t html5 -s --metadata pagetitle="NBPOS" -c "../../../$css" -o "$out/book.html"
  wkhtmltopdf --enable-local-file-access -q "$out/book.html" "$out/guide.pdf"
fi
echo "$out"
