#!/usr/bin/env bash
# Smoke test — runs one cell, one scenario, end-to-end.
#
# Manages the proxies the harness needs:
#   - anthropic_translator_proxy.py  (when blue is anthropic/claude-*)
#   - reasoning_shim_proxy.py        (when blue is a thinking model: kimi, qwen)
#
# Usage:
#   ./scripts/smoke_test.sh                                       # default: hermes/sonnet/crud-resolve
#   FRAMEWORK=ironclaw MODEL=claude-sonnet-4-6 ./scripts/smoke_test.sh
#   FRAMEWORK=openclaw MODEL=kimi-k2.6 ./scripts/smoke_test.sh
#
# Prereqs:
#   ./scripts/setup.sh --with-<framework>     # for the framework you target
#   .env or exported keys: OPENAI_API_KEY, ANTHROPIC_API_KEY, OPENROUTER_API_KEY (as needed)

set -euo pipefail

FRAMEWORK="${FRAMEWORK:-hermes}"
MODEL="${MODEL:-claude-sonnet-4-6}"
SCENARIO="${SCENARIO:-commitments/crud-resolve}"

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
RUNS_DIR="${RUNS_DIR:-runs/smoke-$(date -u +%Y%m%dT%H%M%SZ)}"

echo "=== smoke test ==="
echo "  framework:  $FRAMEWORK"
echo "  model:      $MODEL"
echo "  scenario:   $SCENARIO"
echo "  runs dir:   $RUNS_DIR"
echo

# ─── env var checks ─────────────────────────────────────────────────────────
[[ -z "${OPENAI_API_KEY:-}" ]] && { echo "ERROR: OPENAI_API_KEY is required (red is GPT-5)"; exit 1; }

case "$MODEL" in
  claude-*|anthropic/claude-*)
    [[ -z "${ANTHROPIC_API_KEY:-}" ]] && { echo "ERROR: ANTHROPIC_API_KEY required for $MODEL"; exit 1; } ;;
  glm-*|qwen-*|kimi-*|openrouter/*)
    [[ -z "${OPENROUTER_API_KEY:-}" ]] && { echo "ERROR: OPENROUTER_API_KEY required for $MODEL"; exit 1; } ;;
esac

# ─── proxy lifecycle ────────────────────────────────────────────────────────
PROXY_PIDS=()
cleanup() {
  for pid in "${PROXY_PIDS[@]}"; do
    kill "$pid" 2>/dev/null || true
  done
}
trap cleanup EXIT

# Anthropic translator proxy: only for anthropic/* models
ANTHROPIC_PROXY_PORT="${ANTHROPIC_PROXY_PORT:-8767}"
case "$MODEL" in
  claude-*|anthropic/claude-*)
    if ! curl -sf "http://localhost:${ANTHROPIC_PROXY_PORT}/v1/models" >/dev/null 2>&1; then
      echo "==> starting anthropic_translator_proxy on :${ANTHROPIC_PROXY_PORT}"
      UPSTREAM_API_KEY="$ANTHROPIC_API_KEY" PROXY_PORT="$ANTHROPIC_PROXY_PORT" \
        nohup python scripts/anthropic_translator_proxy.py >/tmp/anth_proxy.log 2>&1 &
      PROXY_PIDS+=($!)
      sleep 2
    fi
    BLUE_BASE_URL="http://localhost:${ANTHROPIC_PROXY_PORT}/v1"
    BLUE_API_KEY="anything"
    ;;
esac

# Reasoning shim proxy: only for thinking models behind Hermes/Openclaw
SHIM_PROXY_PORT="${SHIM_PROXY_PORT:-8085}"
case "$MODEL" in
  kimi-*|qwen-*)
    if ! curl -sf "http://localhost:${SHIM_PROXY_PORT}/v1/models" >/dev/null 2>&1; then
      echo "==> starting reasoning_shim_proxy on :${SHIM_PROXY_PORT}"
      UPSTREAM_BASE_URL=https://openrouter.ai/api UPSTREAM_API_KEY="$OPENROUTER_API_KEY" \
        PROXY_PORT="$SHIM_PROXY_PORT" \
        nohup python scripts/reasoning_shim_proxy.py >/tmp/shim_proxy.log 2>&1 &
      PROXY_PIDS+=($!)
      sleep 2
    fi
    BLUE_BASE_URL="http://localhost:${SHIM_PROXY_PORT}/v1"
    BLUE_API_KEY="$OPENROUTER_API_KEY"
    ;;
esac

# Ensure Hermes is on PYTHONPATH if it's been vendored
if [[ -d "$ROOT/vendor/hermes" ]]; then
  export PYTHONPATH="$ROOT/vendor/hermes:${PYTHONPATH:-}"
fi

# ─── per-framework smoke ────────────────────────────────────────────────────
SIDECAR="$ROOT/sidecars/$SCENARIO.yaml"
SCENARIO_ROOT="$ROOT/scenarios"

case "$FRAMEWORK" in
  hermes)
    python -m harness.hermes.run \
      --sidecar "$SIDECAR" \
      --scenario-root "$SCENARIO_ROOT" \
      --runs-dir "$RUNS_DIR" \
      --blue-model "$MODEL" \
      ${BLUE_BASE_URL:+--blue-base-url "$BLUE_BASE_URL"} \
      ${BLUE_API_KEY:+--blue-api-key "$BLUE_API_KEY"} \
      --red-reasoning-effort low
    ;;
  ironclaw)
    BENCH_BIN="${BENCH_BIN:-$(cd "$ROOT/../.." && pwd)/target/release/nearai-bench}"
    BENCH_CONFIG="${BENCH_CONFIG:-$(cd "$ROOT/../.." && pwd)/suites/trajectory.toml}"
    [[ -x "$BENCH_BIN" ]] || { echo "ERROR: bench binary not found at $BENCH_BIN. Run scripts/setup.sh --with-ironclaw"; exit 1; }
    python -m harness.ironclaw.run \
      --sidecar "$SIDECAR" \
      --scenario-root "$SCENARIO_ROOT" \
      --bench-binary "$BENCH_BIN" \
      --bench-config "$BENCH_CONFIG" \
      --runs-dir "$RUNS_DIR" \
      --red-reasoning-effort low
    ;;
  openclaw)
    OPENCLAW_CONTAINER="${OPENCLAW_CONTAINER:-openclaw}"
    docker inspect "$OPENCLAW_CONTAINER" >/dev/null 2>&1 || {
      echo "ERROR: openclaw container '$OPENCLAW_CONTAINER' not running. Run scripts/setup.sh --with-openclaw and start it."
      exit 1
    }
    python -m harness.openclaw.fanout \
      --sidecar "$SIDECAR" \
      --scenario-root "$SCENARIO_ROOT" \
      --runs-dir "$RUNS_DIR" \
      --container "$OPENCLAW_CONTAINER" \
      --blue-model "$MODEL" \
      ${BLUE_BASE_URL:+--blue-base-url "$BLUE_BASE_URL"} \
      ${BLUE_API_KEY:+--blue-api-key "$BLUE_API_KEY"} \
      --red-reasoning-effort low
    ;;
  *)
    echo "ERROR: unknown FRAMEWORK=$FRAMEWORK (expected hermes|ironclaw|openclaw)"
    exit 1
    ;;
esac

echo
echo "✓ smoke test passed"
echo "  result dir: $RUNS_DIR"
