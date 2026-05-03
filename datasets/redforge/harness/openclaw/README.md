# `harness/openclaw/` — Openclaw adapter

Wraps [Openclaw](https://github.com/nearai/openclaw) (an open-source
coding-agent runtime) for benchmark runs. Openclaw runs in a docker container
and exposes an HTTP API on port 18789 inside the container; the harness
talks to it via `docker exec curl`.

## Reasoning capture

Openclaw uses standard OpenAI-compatible chat completion APIs to call its
underlying LLM. For thinking models (Qwen, Kimi), the harness routes the
container's outbound LLM traffic through
[`scripts/reasoning_shim_proxy.py`](../../scripts/reasoning_shim_proxy.py)
on `host.docker.internal:8085` to surface `reasoning_content`.

## Running

```sh
python -m harness.run \
  --framework openclaw \
  --model claude-sonnet-4-6 \
  --scenario commitments/crud-resolve \
  --container openclaw      # docker container name
```

## Container setup

```sh
docker build -t openclaw:local /path/to/openclaw
docker run -d --name openclaw \
  --add-host host.docker.internal:host-gateway \
  -v /path/to/state:/home/node/.openclaw \
  openclaw:local
```

For matrix runs the harness spins up four parallel containers
(`openclaw_qwen_{1..4}`) and round-robins scenarios across them.

## Files

- `__init__.py` — `OpenclawAdapter` conforming to the harness ABC.
- `runner.py` — per-attempt run logic.
- `fanout.py` — multi-scenario orchestration; supports container parallelism.
