#!/usr/bin/env bash
#
# A/B benchmark two ironclaw versions on the same suite.
#
# Usage:
#   ./scripts/bench-ab.sh <ref-a> <ref-b> --suite <suite> [--config <toml>] [extra args...]
#
# Examples:
#   # Compare staging vs a feature branch
#   ./scripts/bench-ab.sh staging feat/learning-v2 --suite trajectory --config suites/ironclaw-v2.toml
#
#   # Compare two specific commits
#   ./scripts/bench-ab.sh 8acdd08 f3a1b2c --suite spot --model gpt-4o
#
#   # Compare a release tag vs HEAD
#   ./scripts/bench-ab.sh v0.22.0 staging --suite trajectory --config suites/ironclaw-v2.toml
#
# The script:
#   1. Builds the harness against ref-a, runs the suite, captures run ID
#   2. Builds the harness against ref-b, runs the suite, captures run ID
#   3. Compares the two runs
#
set -euo pipefail

if [ $# -lt 4 ]; then
    echo "Usage: $0 <ref-a> <ref-b> --suite <suite> [--config <toml>] [extra args...]"
    echo ""
    echo "Examples:"
    echo "  $0 staging feat/learning-v2 --suite trajectory --config suites/ironclaw-v2.toml"
    echo "  $0 8acdd08 f3a1b2c --suite spot --model gpt-4o"
    echo "  $0 v0.22.0 staging --suite trajectory --config suites/ironclaw-v2.toml"
    exit 1
fi

REF_A="$1"
REF_B="$2"
shift 2

echo "========================================"
echo "  A/B Benchmark: ironclaw"
echo "  Ref A: $REF_A"
echo "  Ref B: $REF_B"
echo "  Args:  $*"
echo "========================================"
echo ""

# Helper: run a benchmark for a given ref and capture the run ID from output
run_bench() {
    local ref="$1"
    shift

    echo "--- Building and running with ironclaw @ $ref ---"
    # Run and capture output to a temp file so we can both display it and parse the run ID.
    local tmpout
    tmpout=$(mktemp)
    trap "rm -f '$tmpout'" RETURN

    cargo run --release -- run --ironclaw-rev "$ref" "$@" 2>&1 | tee "$tmpout"
    local run_id
    run_id=$(grep -oE 'Run complete: [0-9a-f-]+' "$tmpout" | tail -1 | sed 's/Run complete: //')

    rm -f "$tmpout"

    if [ -z "$run_id" ]; then
        echo "ERROR: Could not extract run ID for ref $ref" >&2
        exit 1
    fi

    echo "$run_id"
}

echo ""
echo "=== Phase 1/3: Running benchmark with ironclaw @ $REF_A ==="
echo ""
RUN_A=$(run_bench "$REF_A" "$@")
echo ""
echo "Ref A ($REF_A) run ID: $RUN_A"

echo ""
echo "=== Phase 2/3: Running benchmark with ironclaw @ $REF_B ==="
echo ""
RUN_B=$(run_bench "$REF_B" "$@")
echo ""
echo "Ref B ($REF_B) run ID: $RUN_B"

echo ""
echo "=== Phase 3/3: Comparing results ==="
echo ""
cargo run --release -- compare "$RUN_A" "$RUN_B"

echo ""
echo "========================================"
echo "  A/B Complete"
echo "  Ref A ($REF_A): $RUN_A"
echo "  Ref B ($REF_B): $RUN_B"
echo "========================================"
echo ""
echo "To re-compare later:"
echo "  cargo run -- compare $RUN_A $RUN_B"
