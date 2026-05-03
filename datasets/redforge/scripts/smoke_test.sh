#!/usr/bin/env bash
# Smoke test — runs one cell, one scenario, end-to-end.
#
# Validates that:
#   - Required env vars are set
#   - Python deps are installed
#   - The harness can launch red and blue
#   - The reasoning shim proxy is reachable
#   - The judge produces a verdict
#   - A run artifact lands in runs/
#
# Usage:
#   ./scripts/smoke_test.sh                      # default: hermes/sonnet/crud-resolve
#   FRAMEWORK=ironclaw MODEL=glm-5 ./scripts/smoke_test.sh
#
# Exit codes:
#   0 — smoke test passed
#   1 — env or dep problem
#   2 — harness ran but produced no result
#   3 — result file present but malformed

set -euo pipefail

FRAMEWORK="${FRAMEWORK:-hermes}"
MODEL="${MODEL:-claude-sonnet-4-6}"
SCENARIO="${SCENARIO:-commitments/crud-resolve}"
ATTEMPTS="${ATTEMPTS:-1}"  # smoke = 1 attempt; full run = 5

REDFORGE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RUNS_DIR="${REDFORGE_ROOT}/runs/smoke-$(date -u +%Y%m%dT%H%M%SZ)"

echo "=== smoke test ==="
echo "  framework:  $FRAMEWORK"
echo "  model:      $MODEL"
echo "  scenario:   $SCENARIO"
echo "  attempts:   $ATTEMPTS"
echo "  runs dir:   $RUNS_DIR"
echo

# ─── env var sanity ─────────────────────────────────────────────────────────
require_env() {
  if [[ -z "${!1:-}" ]]; then
    echo "ERROR: \$$1 is not set." >&2
    return 1
  fi
}

case "$MODEL" in
  claude-*)         require_env ANTHROPIC_API_KEY ;;
  gpt-*|openai-*)   require_env OPENAI_API_KEY ;;
  glm-*|qwen-*|kimi-*) require_env OPENROUTER_API_KEY ;;
  *) echo "WARN: unknown model family for $MODEL; not checking env vars" ;;
esac
require_env OPENAI_API_KEY  # red is GPT-5 throughout — always required

# ─── deps ───────────────────────────────────────────────────────────────────
python -c "import yaml, requests, json" 2>/dev/null || {
  echo "ERROR: python deps missing. Run: pip install -r requirements.txt" >&2
  exit 1
}

# ─── reasoning shim proxy (only needed for non-thinking blue models) ────────
if [[ "$MODEL" == kimi-* || "$MODEL" == qwen-* ]]; then
  PROXY_PORT="${PROXY_PORT:-8085}"
  if ! curl -sf "http://localhost:${PROXY_PORT}/v1/models" > /dev/null; then
    echo "Starting reasoning shim proxy on :${PROXY_PORT}..."
    nohup python "${REDFORGE_ROOT}/scripts/reasoning_shim_proxy.py" \
      > /tmp/redforge_shim_smoke.log 2>&1 &
    sleep 3
  fi
fi

# ─── run ────────────────────────────────────────────────────────────────────
mkdir -p "$RUNS_DIR"
python -m harness.run \
  --framework "$FRAMEWORK" \
  --model "$MODEL" \
  --scenario "$SCENARIO" \
  --attempts "$ATTEMPTS" \
  --runs-dir "$RUNS_DIR"

# ─── verify result ──────────────────────────────────────────────────────────
RESULT_JSON=$(find "$RUNS_DIR" -name "*_result.json" | head -1)
if [[ -z "$RESULT_JSON" ]]; then
  echo "ERROR: no result file produced under $RUNS_DIR" >&2
  exit 2
fi

# Required fields
python -c "
import json, sys
d = json.load(open('$RESULT_JSON'))
required = ['scenario_id', 'target_framework', 'target_model', 'attempts', 'any_violation']
missing = [k for k in required if k not in d]
if missing:
    print(f'ERROR: result file missing fields: {missing}', file=sys.stderr)
    sys.exit(3)
if not d.get('attempts'):
    print('ERROR: no attempts recorded', file=sys.stderr)
    sys.exit(3)
print(f'  scenario:   {d[\"scenario_id\"]}')
print(f'  framework:  {d[\"target_framework\"]}')
print(f'  model:      {d[\"target_model\"]}')
print(f'  attempts:   {len(d[\"attempts\"])}')
print(f'  violation:  {d[\"any_violation\"]}')
"

echo
echo "✓ smoke test passed"
echo "  result: $RESULT_JSON"
