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

`scripts/setup.sh --with-openclaw` builds the `openclaw:local` image. Before
the smoke can run, you need a configured state directory and a running
container. Openclaw is a third-party tool with its own setup CLI — we use
its standard configure flow rather than scripting around it (the wizard
prompts evolve between openclaw releases; bypassing it tends to break).

**Step 1 — populate the state dir with provider config + your API keys.**
Run openclaw's interactive configure wizard inside the image (writes
provider config + auth into a JSON file under `~/.openclaw/` on the host;
the exact path depends on the openclaw version):

```sh
mkdir -p ~/.openclaw
docker run -it --rm \
  -v ~/.openclaw:/home/node/.openclaw \
  openclaw:local \
  node openclaw.mjs configure
```

The wizard walks you through adding providers and models. For RedForge
smokes you typically need either:

- **OpenRouter (for kimi/qwen via the reasoning shim)** — provider type
  `openai-completions`, baseUrl `http://host.docker.internal:8085/v1`,
  apiKey is anything (the proxy uses `OPENROUTER_API_KEY` from the host
  env), and add a model with the id you'll pass to the harness
  (e.g. `qwen-3.5`).
- **Z.AI (for glm-5)** — provider type `openai-completions`, baseUrl
  `https://api.z.ai/api/paas/v4`, apiKey from your Z.AI dashboard, and a
  model with id `glm-5`.
- **Anthropic via translator proxy (for claude-*)** — provider type
  `openai-completions`, baseUrl `http://host.docker.internal:8767/v1`,
  apiKey is anything (translator proxy uses `ANTHROPIC_API_KEY` from
  host env), and a model with id `claude-sonnet-4-6`.

The `configure` wizard saves to `~/.openclaw/`. Inspect with
`cat ~/.openclaw/agents/main/agent/models.json` to verify.

**Step 2 — start the gateway container with the state mounted.**

```sh
docker run -d --name openclaw \
  -e OPENCLAW_GATEWAY_TOKEN=local-test \
  -v ~/.openclaw:/home/node/.openclaw \
  --add-host=host.docker.internal:host-gateway \
  openclaw:local \
  node openclaw.mjs gateway --allow-unconfigured
```

`--allow-unconfigured` lets the gateway skip its own per-instance setup
(we already configured via step 1). `OPENCLAW_GATEWAY_TOKEN` satisfies
the gateway's auth bootstrap — value doesn't matter for local use.
`--add-host` lets the container reach the reasoning-shim and Anthropic
translator proxies on the host.

Verify the container is healthy:

```sh
docker logs openclaw | grep "gateway.*listening"
```

Should print `[gateway] listening on ws://127.0.0.1:18789`.

**Step 3 — run the smoke.**

```sh
FRAMEWORK=openclaw MODEL=qwen-3.5 SCENARIO=commitments/crud-resolve \
OPENCLAW_CONTAINER=openclaw \
  ./redforge/scripts/smoke_test.sh
```

For matrix runs you can spin up several containers (e.g.
`openclaw_qwen_{1..8}`) and shard scenarios across them via
`--shard i/N` on the fanout CLI. Each container needs its own state
dir mounted (the agent state isn't safe to share concurrently).

## Files

- `adapter.py` — runs a single attempt against an Openclaw container.
- `orchestrator.py` — per-scenario attempt loop. Imports
  `red_attacker`, `sidecar`, and `invariants` from `harness.ironclaw`
  (those modules are framework-agnostic; their location is historical).
- `fanout.py` — multi-scenario CLI; supports `--filter`, `--limit`,
  `--shard` for sharding across containers.
