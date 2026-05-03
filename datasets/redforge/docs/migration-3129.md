# Migration: removing the forked-Ironclaw workarounds (#3129)

**Status:** TODO before PR opens.
**Depends on:** [nearai/ironclaw#3129](https://github.com/nearai/ironclaw/pull/3129) — merged 2026-05-01.

Until #3129 landed, the benchmark harness captured Ironclaw's reasoning trace
by tailing the docker container's stderr for `[redforge-dump]` lines emitted
by a patched-fork ironclaw. #3129 replaces that with a sanctioned `tracing`
target on `ironclaw::llm::reasoning`. This PR drops the workarounds.

## Workarounds to remove

| File | What it did | Action |
|---|---|---|
| `harness/dump_tail.py` | Tailed docker logs for `[redforge-dump]` reasoning content. | **Delete** the file. |
| `harness/attacker.py:10` | `from .dump_tail import fetch_reasoning` | Remove the import. |
| `harness/v2/runner.py:23` | Same import. | Remove. |
| `harness/v2/runner.py:83` | Sets `env["REDFORGE_DUMP_RAW_LLM"] = "1"` | Remove. |
| `harness/v2/runner.py:138` | `reasoning = _parse_reasoning_from_stderr(proc.stderr or "")` | Replace with reading the new tracing target's output. |
| `harness/v2/runner.py:183` | `_parse_reasoning_from_stderr` helper | Delete. |
| `harness/v2/run.py:42` | `--container` flag for the v2 (Ironclaw) runner. | The flag stays — Ironclaw still runs in a container — but it's no longer used for reasoning capture. |
| `scripts/run_v2_fanout.sh:35` | Passes `--container ironclaw`. | No change (container is still required). |

## What replaces them

Per #3129's PR description:

> *Companion PR to nearai/benchmarks will add a `tracing::Subscriber::Layer`
> in `src/instrumented_llm.rs` to capture this target into
> `TaskResult.reasoning`, surfacing it in `tasks.jsonl` (~25 lines, no
> wire-format break).*

Two pieces of work:

1. **Bench-side Rust** (`nearai/benchmarks/src/instrumented_llm.rs`): add a
   `tracing::Subscriber::Layer` that filters on target `ironclaw::llm::reasoning`
   and writes the reasoning content into `TaskResult.reasoning`.
2. **Bench-side Python harness** (`harness/v2/runner.py`): read the reasoning
   field from the structured `tasks.jsonl` output instead of parsing stderr.

After both land, the harness's reasoning capture goes from "fragile string-grep
on docker logs" to "sanctioned structured field on the task result," and we
can drop the patched-fork ironclaw dependency entirely.

## Verification

The smoke test must pass with reasoning surfaced via the new path before this
PR is ready for review. Specifically: run a Kimi/Hermes `crud-resolve` cell;
verify `result.attempts[0].blue_reasoning` is populated and matches what red
sees between attempts.

## Other harnesses (Hermes, Openclaw)

Hermes and Openclaw are not Ironclaw-based; their reasoning capture goes
through `scripts/reasoning_shim_proxy.py` (an OpenAI-compatible proxy that
injects placeholder `reasoning_content` for non-thinking models, and
forwards it for thinking models). That path is unaffected by #3129.
