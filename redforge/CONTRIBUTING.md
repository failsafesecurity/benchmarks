# Contributing

This benchmark gets stronger when more people run their own framework,
model, or scenario through it. This doc covers the three common
contributions.

If you're proposing something larger (a new attack family, a new judging
mechanism, a structural redesign), please open an issue first — easier to
align on the shape before writing code.

## Adding a new framework

Drop a new package under `harness/<framework>/`. The simplest path is to
mirror `harness/openclaw/`, which has the smallest surface:

- `adapter.py` — instantiates the framework and runs a single attempt
  (red has already mutated the workspace at this point); returns blue's
  reasoning, tool calls, and final response in a uniform `BlueRunResult`.
- `orchestrator.py` — the per-scenario attempt loop. Imports
  `red_attacker`, `sidecar`, and `invariants` from `harness.ironclaw`
  (these modules are framework-agnostic in practice; their location
  under `harness/ironclaw/` is historical and is what the other
  framework orchestrators import).
- `run.py` or `fanout.py` — CLI entry point.

The `BlueRunResult` shape is defined inline in
[`harness/ironclaw/red_attacker.py`](../redforge/harness/ironclaw/red_attacker.py).
The two fields that matter for adversarial visibility are
`blue_reasoning` (full chain of thought) and `tool_calls` (structured
list of `{name, arguments, result}` dicts).

For framework-specific reasoning capture, look at how the existing
adapters do it: `harness/ironclaw/runner.py` reads Ironclaw's
`tasks.jsonl` (which surfaces `reasoning_content` after
[nearai/ironclaw#3129](https://github.com/nearai/ironclaw/pull/3129));
`harness/hermes/adapter.py` and `harness/openclaw/adapter.py` route
their model traffic through `scripts/reasoning_shim_proxy.py`, which
forwards `reasoning_content` for thinking models.

After your adapter passes the smoke, run it through the corpus by
following the per-framework fanout in `harness/<your-framework>/`. Open
a PR with the adapter, an updated README, and the run artifacts under
`runs/<your-name>-<date>/`. We'll add the new column to the matrix.

## Adding a new model

Most providers route through OpenAI-compatible chat completion APIs.
There's no central model registry — each framework's adapter resolves
the model name to a base URL and API key the same way `smoke_test.sh`
does (`anthropic_translator_proxy.py` for `claude-*`,
`reasoning_shim_proxy.py` for `kimi-*` / `qwen-*`, direct OpenRouter
for `glm-*`).

If your model is a thinking model and emits structured reasoning that
the existing shim proxy doesn't handle, add a translation rule to
[`scripts/reasoning_shim_proxy.py`](scripts/reasoning_shim_proxy.py).

Run the smoke, then a single (your-model, framework) cell, then if
those pass open a PR.

## Adding a new scenario

A scenario is two files:

1. The scenario definition (JSON):
   - For trajectory-style scenarios:
     `datasets/trajectory/v1/structured-data/<category>/<name>.json` —
     these live alongside the upstream structured-data tree contributed
     in [#18](https://github.com/nearai/benchmarks/pull/18) and are also
     usable as a non-adversarial capability benchmark.
   - For RedForge-only variants (e.g. math-heavy ones):
     `redforge/scenarios/math/<name>.json`.
   Each scenario defines a `setup` block (system prompt, workspace
   files, available tools), a user request, and expected answer /
   success criteria.
2. `redforge/sidecars/<category>/<name>.yaml` — the adversarial config:
   - `scenario_ref`: repo-rooted path to the scenario JSON
     (e.g. `datasets/trajectory/v1/structured-data/aggregation/active-per-owner.json`
     or `redforge/scenarios/math/<name>.json`).
   - `red_controlled_fields`: which paths and which fields red can
     edit.
   - `arc_invariants`: per-layer invariants the judge will check
     (`tool`, `workflow`, `permission`, `cross_cutting`).
   - `attempts`, `max_fields_per_attempt`: budget knobs.

Existing sidecars under `redforge/sidecars/commitments/`,
`redforge/sidecars/transactions/`, etc. are good templates.

For the scenario to be useful as a benchmark cell, the success criteria
and the invariants must be **deterministically checkable** by the
judge. The judge implementation in
[`harness/ironclaw/invariants.py`](harness/ironclaw/invariants.py)
supports:

- Path-prefix and exact-match checks on tool calls.
- Regex on the response.
- Numeric tolerance comparisons.
- Iteration counts.
- Markdown-image and identity-write detection.

If your scenario needs a check the judge doesn't support, propose an
extension in the same PR.

## Reproducing a published cell

```sh
FRAMEWORK=hermes \
MODEL=kimi-k2.6 \
SCENARIO=commitments/crud-resolve \
  ./scripts/smoke_test.sh
```

Compare the run output to the artifact under
`runs/matrix-2026-05-02/per-cell/kimi-k2.6-hermes/`. The `summary.csv`
verdict for `commitments-crud-resolve` should match modulo model-API
nondeterminism (some scenarios are sensitive to sampling temperature).

## Reporting a result that disagrees with the matrix

If you reproduce a cell and get a materially different verdict count
(e.g. ≥ 3-violation difference at N=5), open an issue with:

- The cell (model, framework, scenario set).
- Your run output (`runs/your-rerun/`).
- A diff against the published cell.

Differences are real signal — they could indicate model-version drift,
provider-side changes, or genuine variance we should document.

## Code style

- Python: black + ruff. `ruff check . && black --check .` before pushing.
- Rust: `cargo fmt && cargo clippy`.
- Shell: `shellcheck scripts/*.sh`.

## License

Contributions are accepted under MIT.
