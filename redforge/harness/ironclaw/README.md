# `harness/ironclaw/` — Ironclaw adapter

Wraps [Ironclaw](https://github.com/nearai/ironclaw) (a Rust-based agent
runtime) for benchmark runs. Ironclaw is invoked through the
`nearai-bench` binary built from this repository's root; the harness
shells out per attempt, supplies the scenario's workspace, and reads
back blue's reasoning + tool calls from the structured `tasks.jsonl`
output.

## Reasoning capture

Reasoning content is captured via Ironclaw's `tracing` channel on
target `ironclaw::llm::reasoning`. The bench-side
`tracing::Subscriber::Layer` in
[`src/instrumented_llm.rs`](../../../../src/instrumented_llm.rs)
filters that target into `TaskResult.reasoning`, which the harness picks
up from `tasks.jsonl`. See
[nearai/ironclaw#3129](https://github.com/nearai/ironclaw/pull/3129)
for the upstream tracing target.

## Running

A single scenario through the smoke driver:

```sh
FRAMEWORK=ironclaw \
MODEL=glm-5 \
SCENARIO=commitments/crud-resolve \
  ./scripts/smoke_test.sh
```

Or directly:

```sh
python -m harness.ironclaw.run \
  --sidecar sidecars/commitments/crud-resolve.yaml \
  --scenario-root scenarios \
  --bench-binary /path/to/target/release/nearai-bench \
  --bench-config /path/to/suites/trajectory.toml \
  --runs-dir runs/my-run
```

## Bench binary

The runner needs a built `nearai-bench`. From this repository's root:

```sh
cargo build --release --bin nearai-bench
```

Or, equivalently, `./scripts/setup.sh --with-ironclaw`. The binary path
can be passed with `--bench-binary` or env var `BENCH_BIN`.

## Files

- `runner.py` — invokes `nearai-bench` and reads back the structured
  result.
- `orchestrator.py` — per-scenario attempt loop (5 attempts; red ↔
  blue). The shared modules (`red_attacker.py`, `sidecar.py`,
  `invariants.py`) live in this directory but are imported by all three
  framework orchestrators.
- `red_attacker.py` — GPT-5 attacker (framework-agnostic; named
  `ironclaw/` for historical reasons).
- `sidecar.py` — sidecar YAML loader + path rewrite logic.
- `invariants.py` — judge invariant checks.
- `run.py` — CLI for a single sidecar.
