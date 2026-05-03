# `harness/hermes/` — Hermes adapter

Wraps the Hermes agent runtime (planner-first, conversational tool use) for
benchmark runs. Unlike Ironclaw and Openclaw, Hermes runs in-process from
Python — no docker container required.

## Reasoning capture

Hermes uses standard OpenAI-compatible chat completion APIs. For models that
emit reasoning (Qwen-3.5, Kimi K2.6), the harness routes through
[`scripts/reasoning_shim_proxy.py`](../../scripts/reasoning_shim_proxy.py),
which surfaces `reasoning_content` on a sanctioned channel.

For non-thinking models (Sonnet, GLM-5), reasoning is captured from the
provider's native reasoning field where available, or omitted otherwise.

## Running

```sh
python -m harness.run \
  --framework hermes \
  --model claude-sonnet-4-6 \
  --scenario commitments/crud-resolve
```

No container, no `--container` flag.

## Note on Hermes' permission model

Hermes does **not** ship with a workspace-write allowlist by default. This is
the structural gap the paper's [Finding 1](../../paper/01_blog.md) calls out.
The harness does not patch this — the benchmark measures Hermes as it ships,
not as it could be. Adding an allowlist is a framework-side change Hermes
maintainers would land upstream; we document the gap, not paper over it.

## Files

- `__init__.py` — `HermesAdapter` conforming to the harness ABC.
- `runner.py` — per-attempt run logic.
- `fanout.py` — multi-scenario orchestration.
