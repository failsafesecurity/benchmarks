# Contributing

This benchmark gets stronger when more people run their own framework,
model, or scenario through it. This doc covers the three common
contributions.

If you're proposing something larger (a new attack family, a new judging
mechanism, a structural redesign), please open an issue first — easier to
align on the shape before writing code.

## Adding a new framework

Drop an adapter under `harness/<framework>/` that conforms to the harness ABC
(see `harness/__init__.py`).

Required interface:

- `setup(scenario, sidecar) -> SessionHandle` — prepare the framework with
  the scenario's workspace and tools.
- `run_attempt(session, user_request, attempt_n) -> AttemptResult` — execute
  one attempt; return blue's reasoning, tool calls, and final response.
- `teardown(session)` — clean up.

The `AttemptResult` shape is in `harness/types.py`. The two fields that
matter for adversarial visibility are `blue_reasoning` (full chain of
thought, captured *somehow*) and `tool_calls` (structured list with
`name`, `arguments`, `result`).

For framework-specific reasoning capture, see how the existing adapters do
it: `harness/ironclaw/` uses Ironclaw's `tracing` channel; `harness/hermes/`
and `harness/openclaw/` route through the reasoning shim proxy.

After your adapter passes the smoke test (`./scripts/smoke_test.sh`), run it
through the full corpus:

```sh
./scripts/run_matrix.sh --frameworks <your-framework> --models all
```

Open a PR with the adapter, a per-framework README, and the run artifacts
under `runs/<your-name>-<date>/`. We'll add the new column to the matrix.

## Adding a new model

Most providers route through the existing OpenAI-compatible adapters.
Add the model's identifier to `harness/models.py` — typically just a config
entry mapping the model name to its provider, base URL, and any special
parameters (reasoning effort, max tokens).

If your model is a thinking model (emits structured reasoning), make sure
`scripts/reasoning_shim_proxy.py` either passes through the reasoning field
or has a translation rule for your model's reasoning convention.

Run the smoke test, then a single (your-model, framework) cell, then if
those pass open a PR.

## Adding a new scenario

A scenario is two files:

1. `scenarios/<category>/<name>.json` — the scenario definition: `setup`
   block (system prompt, workspace files, available tools), `user_input`
   (the user's request), `success_criteria` (what blue must do).
2. `sidecars/<category>/<name>.yaml` — the adversarial config:
   - `red_controlled_fields`: which paths and which fields red can edit.
   - `arc_invariants`: per-layer invariants the judge will check
     (`tool`, `workflow`, `permission`, `cross_cutting`).
   - `attempts`, `max_fields_per_attempt`: budget knobs.

Existing scenarios under `scenarios/commitments/`,
`scenarios/transactions/`, etc. are good templates.

For the scenario to be useful as a benchmark cell, the success criteria and
the invariants must be **deterministically checkable** by the judge. The
judge in `judges/verdict.py` supports:

- Path-prefix and exact-match checks on tool calls.
- Regex on the response.
- Numeric tolerance comparisons.
- Iteration counts.
- Markdown-image and identity-write detection.

If your scenario needs a check the judge doesn't support, propose an
extension in the same PR.

## Reproducing a published cell

```sh
./scripts/smoke_test.sh \
  FRAMEWORK=hermes \
  MODEL=kimi-k2.6 \
  SCENARIO=commitments/crud-resolve \
  ATTEMPTS=5
```

Compare the run output to the artifact under
`runs/matrix-2026-05-02/per-cell/kimi-hermes/commitments-crud-resolve/`.
Verdicts should match modulo model-API nondeterminism (some scenarios are
sensitive to sampling temperature).

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
