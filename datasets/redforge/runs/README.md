# `runs/` — canonical run artifacts

This directory holds the run output the paper cites. Per the post's claim
that *"every cell in the matrix traces back to a verifiable artifact,"* the
artifacts here are the receipts.

## Layout

```
runs/
└── matrix-2026-05-02/             ← canonical 4×3 matrix
    ├── README.md
    ├── summary.csv                ← 12 rows: (model, framework) → violation count
    ├── per-attack-family.csv      ← 12 rows × 4 attack families
    └── per-cell/                  ← per-cell artifacts
        ├── sonnet-ironclaw/
        │   ├── summary.csv        ← per-scenario verdict (52 rows)
        │   └── verdicts.jsonl.gz  ← compact per-scenario verdicts
        ├── kimi-hermes/
        │   ├── summary.csv
        │   ├── verdicts.jsonl.gz
        │   └── crud-resolve/      ← verbatim raw for cells the paper cites
        │       └── full_run.json  ← all 5 attempts, full reasoning traces
        └── ... (12 cells)
```

## What's preserved verbatim vs. compacted

**Verbatim (full reasoning traces, all five attempts):** the cells the paper
specifically walks through. Today that's:

- `kimi-hermes/crud-resolve/` — the §3 walked-through "attack lands" example
- `sonnet-hermes/crud-resolve/` — the §3 "attack misses" contrast

**Compact (per-scenario verdicts, no reasoning traces):** all other cells.
Each scenario has a one-line verdict record: scenario id, attempt count,
which attempts violated, what invariant tripped, what red planted (the diff
against the clean workspace). Enough to verify the matrix counts and to
inspect *what* red did, without the storage cost of every reasoning trace.

## Reproducing a verbatim cell

```sh
./scripts/smoke_test.sh \
  FRAMEWORK=hermes MODEL=kimi-k2.6 \
  SCENARIO=commitments/crud-resolve ATTEMPTS=5
```

Then `diff` your run's `verdicts.jsonl` against
`per-cell/kimi-hermes/crud-resolve/full_run.json`. The verdict labels should
match; the reasoning text will differ slightly (sampling).

## Why this slice (not all 624 runs)

The full 624-scenario × 5-attempt corpus is hundreds of MB with all reasoning
traces. We ship the **selective slice** — full receipts for the cells the
paper cites, compact verdicts for everything else. Anyone who wants more
detail on a non-cited cell can re-run; the harness is deterministic modulo
sampling.

## Generating this directory

The selection logic is in
[`scripts/select_run_artifacts.py`](../scripts/select_run_artifacts.py).
Given a source tree of full run artifacts (typically the EC2 boxes the
matrix was run on), it copies the cited cells verbatim and compacts the rest
into `verdicts.jsonl.gz`.
