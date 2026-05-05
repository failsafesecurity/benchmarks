# `harness/openclaw/` — Openclaw adapter

Wraps [Openclaw](https://github.com/nearai/openclaw) (an open-source
coding-agent runtime) for benchmark runs. Openclaw runs in a docker
container; the harness talks to it via `docker exec`.

## Reasoning capture

Openclaw uses standard OpenAI-compatible chat completion APIs to call
its underlying LLM. For thinking models (Qwen, Kimi), the harness
routes the container's outbound LLM traffic through
[`scripts/reasoning_shim_proxy.py`](../../scripts/reasoning_shim_proxy.py)
on `host.docker.internal:8085` to surface `reasoning_content`. For
Claude models, traffic routes through
[`scripts/anthropic_translator_proxy.py`](../../scripts/anthropic_translator_proxy.py)
on `host.docker.internal:8767`. The container must be configured to
route to those host ports — `smoke_test.sh` brings up the proxies but
the container side of the routing is part of the image build (see
`scripts/setup.sh --with-openclaw`).

## Running

A single scenario through the smoke driver:

```sh
FRAMEWORK=openclaw \
MODEL=qwen-3.5 \
SCENARIO=commitments/crud-resolve \
OPENCLAW_CONTAINER=openclaw_qwen_1 \
  ./scripts/smoke_test.sh
```

Or directly through the fanout (which is also the entry point used for
multi-scenario runs):

```sh
python -m harness.openclaw.fanout \
  --sidecars-dir sidecars/commitments \
  --scenario-root scenarios \
  --runs-dir runs/my-run \
  --container openclaw_qwen_1 \
  --blue-model qwen-3.5 \
  --filter crud-resolve.yaml \
  --limit 1
```

## Container setup

```sh
docker build -t openclaw:local /path/to/openclaw
docker run -d --name openclaw_qwen_1 \
  --add-host host.docker.internal:host-gateway \
  -v /path/to/state:/home/node/.openclaw \
  openclaw:local
```

For matrix runs we spin up several containers
(`openclaw_qwen_{1..8}` on Box A) and shard scenarios across them via
`--shard i/N`.

## Files

- `adapter.py` — runs a single attempt against an Openclaw container.
- `orchestrator.py` — per-scenario attempt loop. Imports
  `red_attacker`, `sidecar`, and `invariants` from `harness.ironclaw`
  (those modules are framework-agnostic; their location is historical).
- `fanout.py` — multi-scenario CLI; supports `--filter`, `--limit`,
  `--shard` for sharding across containers.
