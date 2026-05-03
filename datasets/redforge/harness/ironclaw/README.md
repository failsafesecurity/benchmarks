# `harness/ironclaw/` — Ironclaw adapter

Wraps [Ironclaw](https://github.com/nearai/ironclaw) (a Rust-based agent
runtime) for benchmark runs. Ironclaw runs in a docker container; the harness
spawns the container per run, injects the scenario's workspace, and captures
blue's reasoning + tool calls.

## Reasoning capture

After [nearai/ironclaw#3129](https://github.com/nearai/ironclaw/pull/3129)
landed, reasoning content is captured via Ironclaw's `tracing` channel on
target `ironclaw::llm::reasoning`. The bench-side `tracing::Subscriber::Layer`
in `nearai/benchmarks/src/instrumented_llm.rs` reads it into
`TaskResult.reasoning`, which the harness picks up from the structured
`tasks.jsonl` output.

The previous workaround — tailing docker logs for `[redforge-dump]` lines
emitted by a patched-fork ironclaw — has been removed. See
[migration-3129.md](../../docs/migration-3129.md) for the migration record.

## Running

```sh
python -m harness.run \
  --framework ironclaw \
  --model claude-sonnet-4-6 \
  --scenario commitments/crud-resolve \
  --container ironclaw      # docker container name
```

## Container setup

The `--container` flag names a running Ironclaw container the harness will
exec into. Build the image:

```sh
docker build -t ironclaw:local /path/to/ironclaw
docker run -d --name ironclaw \
  --add-host host.docker.internal:host-gateway \
  -v /path/to/state:/home/node/.ironclaw \
  ironclaw:local
```

The container needs network egress to whichever model API blue is using
(or to the reasoning shim proxy on `host.docker.internal:8085` for Kimi/Qwen).

## Files

- `__init__.py` — `IronclawAdapter` conforming to the harness ABC.
- `runner.py` — per-attempt run logic; spawns ironclaw, applies red's edits,
  captures the trace.
- `fanout.py` — multi-scenario / multi-attempt orchestration.
