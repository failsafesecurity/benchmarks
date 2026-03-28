#!/usr/bin/env bash
# Download PinchBench tasks and assets from GitHub.
# Usage: scripts/pinchbench_download.sh [--force]
#
# Requires: gh CLI (authenticated)
# Output: datasets/pinchbench/v1/tasks/*.md + datasets/pinchbench/v1/assets/*

set -euo pipefail

REPO="pinchbench/skill"
DEST="datasets/pinchbench/v1"
TASKS_DIR="$DEST/tasks"
ASSETS_DIR="$DEST/assets"
EXPECTED_TASKS=23

if [[ "${1:-}" != "--force" ]] && [[ -d "$TASKS_DIR" ]] && [[ "$(ls "$TASKS_DIR"/*.md 2>/dev/null | wc -l)" -ge "$EXPECTED_TASKS" ]]; then
    echo "Already have $EXPECTED_TASKS+ task files in $TASKS_DIR. Use --force to re-download."
    exit 0
fi

mkdir -p "$TASKS_DIR" "$ASSETS_DIR"

echo "Downloading task files..."
TASK_FILES=$(gh api "repos/$REPO/contents/tasks" --jq '.[].name' | grep '^task_')
for f in $TASK_FILES; do
    gh api "repos/$REPO/contents/tasks/$f" --jq '.content' | base64 -d > "$TASKS_DIR/$f"
    echo "  $f"
done

echo "Downloading assets..."
ASSET_FILES=$(gh api "repos/$REPO/contents/assets" --jq '.[].name')
for f in $ASSET_FILES; do
    gh api "repos/$REPO/contents/assets/$f" --jq '.content' | base64 -d > "$ASSETS_DIR/$f"
    echo "  $f"
done

TASK_COUNT=$(ls "$TASKS_DIR"/*.md 2>/dev/null | wc -l | tr -d ' ')
ASSET_COUNT=$(ls "$ASSETS_DIR"/* 2>/dev/null | wc -l | tr -d ' ')
echo "Done: $TASK_COUNT tasks, $ASSET_COUNT assets in $DEST"
