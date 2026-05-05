#!/usr/bin/env bash
# Regenerate paper/01_blog_share.html from paper/01_blog.md.
#
# 01_blog.md is the source of truth. The shareable HTML is produced
# from it via pandoc; rerun this script after editing the markdown so
# the HTML doesn't drift.
#
# Requires pandoc (https://pandoc.org/). Run from anywhere — paths are
# resolved relative to this script.

set -euo pipefail

if ! command -v pandoc >/dev/null 2>&1; then
  echo "ERROR: pandoc not installed." >&2
  echo "  macOS:  brew install pandoc" >&2
  echo "  Ubuntu: sudo apt install pandoc" >&2
  exit 1
fi

cd "$(dirname "${BASH_SOURCE[0]}")"

pandoc 01_blog.md \
  --standalone \
  --embed-resources \
  --css style.css \
  --metadata title="Same brain, different agent" \
  --output 01_blog_share.html

echo "wrote $(pwd)/01_blog_share.html"
