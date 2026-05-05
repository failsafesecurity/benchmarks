# `harness/hermes/` — Hermes adapter

Wraps the Hermes agent runtime (planner-first, conversational tool use) for
benchmark runs. Unlike Ironclaw and Openclaw, Hermes runs in-process from
Python — no docker container required.

## Reasoning capture

Hermes uses standard OpenAI-compatible chat completion APIs. For models that
emit reasoning (Qwen-3.5, Kimi K2.6), the harness routes traffic through
[`scripts/reasoning_shim_proxy.py`](../../scripts/reasoning_shim_proxy.py),
which surfaces `reasoning_content` on a sanctioned channel. For Claude
models, traffic routes through
[`scripts/anthropic_translator_proxy.py`](../../scripts/anthropic_translator_proxy.py),
which translates OpenAI-shape requests to Anthropic's `/v1/messages` and
surfaces thinking blocks as `reasoning_content`. `smoke_test.sh` brings up
the right proxy automatically based on `MODEL`.

For non-thinking models without reasoning support, that field is empty.

## Running

A single scenario through the smoke driver:

```sh
FRAMEWORK=hermes \
MODEL=claude-sonnet-4-6 \
SCENARIO=commitments/crud-resolve \
  ./scripts/smoke_test.sh
```

Or directly, bypassing the smoke driver (you supply the proxy URLs and keys
yourself):

```sh
python -m harness.hermes.run \
  --sidecar sidecars/commitments/crud-resolve.yaml \
  --scenario-root scenarios \
  --runs-dir runs/my-run \
  --blue-model claude-sonnet-4-6 \
  --blue-base-url http://localhost:8767/v1 \
  --blue-api-key anything
```

## Note on Hermes' permission model

Hermes does **not** ship with a workspace-write allowlist by default. This is
the structural gap the paper's [Finding 1](../../paper/01_blog.md) calls
out. The harness does not patch this — the benchmark measures Hermes as it
ships, not as it could be. Adding an allowlist is a framework-side change
Hermes maintainers would land upstream; we document the gap, not paper over
it.

## Files

- `adapter.py` — runs a single attempt against Hermes; returns a
  `BlueRunResult`.
- `orchestrator.py` — per-scenario attempt loop (5 attempts; red ↔ blue).
  Imports `red_attacker`, `sidecar`, and `invariants` from
  `harness.ironclaw` (those modules are framework-agnostic; their location
  is historical).
- `run.py` — CLI for a single sidecar.
- `fanout.py` — multi-scenario orchestration.
