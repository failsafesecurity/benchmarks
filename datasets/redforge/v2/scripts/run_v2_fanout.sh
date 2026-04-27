#!/usr/bin/env bash
# Run all RedForge v2 sidecars sequentially against the structured-data bench.
# Writes one v2_result.json per scenario to runs/, plus a summary log.
#
# Run from the repo root:
#   nohup bash datasets/redforge/v2/scripts/run_v2_fanout.sh \
#     /path/to/nearai-bench /path/to/suites/structured-data.toml \
#     > runs/v2-fanout.log 2>&1 &

set -uo pipefail

if [ "$#" -lt 2 ]; then
    echo "usage: $0 <bench-binary> <bench-config>" >&2
    exit 64
fi
BENCH="$1"
CONFIG="$2"
ROOT="$(pwd)"
SUMMARY="$ROOT/runs/v2-fanout-summary-$(date -u +%Y%m%dT%H%M%SZ).txt"
mkdir -p "$ROOT/runs"

# shellcheck disable=SC1091
source .venv/bin/activate 2>/dev/null || true
export PYTHONPATH="$ROOT/datasets/redforge/v2"

SIDECARS=()
while IFS= read -r p; do SIDECARS+=("$p"); done < <(find datasets/redforge/v2/sidecars -type f -name '*.yaml' | sort)

echo "fanout starts $(date -u --iso-8601=seconds): ${#SIDECARS[@]} sidecars" | tee "$SUMMARY"
i=0
for sc in "${SIDECARS[@]}"; do
    i=$((i+1))
    name="$(basename "$sc" .yaml)"
    coll="$(basename "$(dirname "$sc")")"
    echo "" | tee -a "$SUMMARY"
    echo "[$i/${#SIDECARS[@]}] $coll/$name $(date -u +%H:%M:%S)" | tee -a "$SUMMARY"
    out=$(python3 -m harness.run \
        --sidecar "$sc" \
        --bench-binary "$BENCH" \
        --bench-config "$CONFIG" \
        --container ironclaw \
        --timeout-secs 180 2>&1)
    status=$?
    if [ $status -ne 0 ]; then
        echo "  ERROR exit=$status" | tee -a "$SUMMARY"
        echo "$out" | tail -10 | sed 's/^/    /' | tee -a "$SUMMARY"
        continue
    fi
    echo "$out" | grep -E '^v2 result:|^sidecar=|^  attempt' | tee -a "$SUMMARY"
done

echo "" | tee -a "$SUMMARY"
echo "fanout done $(date -u --iso-8601=seconds)" | tee -a "$SUMMARY"
