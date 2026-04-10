#!/usr/bin/env bash
# Download PinchBench assets from GitHub.
# Usage: scripts/pinchbench_download.sh [--force]
#
# Requires: gh CLI (authenticated), curl
# Output: datasets/pinchbench/v1/assets/*

set -euo pipefail

REPO="pinchbench/skill"
ASSETS_DIR="datasets/pinchbench/v1/assets"

if [[ "${1:-}" != "--force" ]] && [[ -d "$ASSETS_DIR" ]] && [[ "$(ls "$ASSETS_DIR"/* 2>/dev/null | wc -l)" -gt 0 ]]; then
    echo "Assets already present in $ASSETS_DIR. Use --force to re-download."
    exit 0
fi

mkdir -p "$ASSETS_DIR"

echo "Downloading assets..."
gh api "repos/$REPO/contents/assets" --jq '.[] | select(.type == "file") | [.name, .download_url] | @tsv' | while IFS=$'\t' read -r name url; do
    curl -fsSL "$url" -o "$ASSETS_DIR/$name"
    echo "  $name"
done

ASSET_COUNT=$(ls "$ASSETS_DIR"/* 2>/dev/null | wc -l | tr -d ' ')
echo "Done: $ASSET_COUNT assets in $ASSETS_DIR"
