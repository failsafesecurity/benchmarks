# `runs/` — canonical run artifacts

This directory holds the run output the paper cites. Per the post's claim
that *"every cell in the matrix traces back to a verifiable artifact,"*
the artifacts here are the receipts.

## Layout

```
runs/
├── manifest.yaml                ← host:path map for full per-cell sources
└── matrix-2026-05-02/           ← canonical 4×3 matrix
    ├── summary.csv              ← 12 rows: (model, framework) → violation count
    ├── per-attack-family.csv    ← 12 rows × 4 attack families
    └── per-cell/
        ├── claude-sonnet-4-6-hermes/
        │   ├── summary.csv      ← per-scenario verdicts (52 rows)
        │   └── commitments-crud-resolve/
        │       └── full_run.json   ← all 5 attempts, full reasoning traces
        ├── kimi-k2.6-hermes/
        │   ├── summary.csv
        │   └── commitments-crud-resolve/
        │       └── full_run.json
        └── ... (12 cells; per-scenario summary.csv only)
```

## What's preserved verbatim vs. compacted

**Verbatim (full reasoning traces, all five attempts):** the cells the
paper specifically walks through. Today that's:

- `claude-sonnet-4-6-hermes/commitments-crud-resolve/` — the §3 "attack
  misses" contrast.
- `kimi-k2.6-hermes/commitments-crud-resolve/` — the §3 walked-through
  "attack lands" example.

**Compact (per-scenario verdicts):** every cell has a `summary.csv`
listing each scenario's verdict and which attempt(s) violated. Enough to
verify the matrix counts and to inspect *which* scenarios failed,
without shipping every reasoning trace.

The full run trees (all 624 attempts × full reasoning) live on the EC2
boxes referenced in `manifest.yaml`; we ship the selective slice here.

## Reproducing a verbatim cell

```sh
FRAMEWORK=hermes \
MODEL=kimi-k2.6 \
SCENARIO=commitments/crud-resolve \
  ./scripts/smoke_test.sh
```

The smoke driver writes a `full_run.json` to `runs/smoke-<timestamp>/`.
Compare its per-attempt verdicts against
`per-cell/kimi-k2.6-hermes/commitments-crud-resolve/full_run.json`.
Verdict labels should match modulo model-API nondeterminism (some
scenarios are sensitive to sampling temperature); reasoning text will
differ on every run.

## Why this slice (not all 624 runs)

The full corpus is hundreds of MB with all reasoning traces. We ship the
**selective slice** — full receipts for the cells the paper cites,
per-scenario verdicts for everything else. Anyone who wants more detail
on a non-cited cell can re-run; the harness is deterministic modulo
model sampling.

## Generating this directory

The selection logic is in
[`scripts/select_run_artifacts.py`](../scripts/select_run_artifacts.py).
Given a source tree of full run artifacts (typically the EC2 boxes the
matrix was run on; see `manifest.yaml`), it copies the cited cells
verbatim and writes per-scenario `summary.csv` for the rest.
