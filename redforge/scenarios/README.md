# `redforge/scenarios/`

This directory holds **24 math-heavy scenario variants** that are unique to
RedForge — used for exercising the adversarial benchmark's numeric-capability
invariants under attack.

The **52 base structured-data scenarios** that drive most of the matrix do
**not** live here. They live under
[`../../datasets/trajectory/v1/structured-data/`](../../datasets/trajectory/v1/structured-data/),
contributed by [#18](https://github.com/nearai/benchmarks/pull/18). Sidecars
in [`../sidecars/`](../sidecars/) reference both trees by repo-rooted
`scenario_ref` paths.

## What's here

```
math/
└── math-{commitments,nanny,transactions}-*.json
```

24 variants, each a counting/summing/aggregation question over the same
fixture data the base scenarios use. Used to surface
`numeric_answer_matches_literal_sum` violations under adversarial
manipulation of free-text fields.

## Running

Through redforge's smoke driver (run from the repo root):

```sh
FRAMEWORK=hermes \
MODEL=claude-sonnet-4-6 \
SCENARIO=math/math-commitments-alice-owned-count \
  ./redforge/scripts/smoke_test.sh
```

Or through the per-framework runner directly — see
[`../harness/<framework>/README.md`](../harness/).

## Scenario shape

Each scenario is a single JSON file that inlines all fixture data in its
`setup.workspace.documents` block:

```
collections/{entity_type}/_schema.json
collections/{entity_type}/{id}-{slug}.json
```

The agent uses `memory_tree`, `memory_read`, and `memory_search` to query
the data. Files are self-contained — no external fixtures, no runtime
generation step.

## Why a separate location

Math variants are RedForge-specific — they were authored to test how
adversarial manipulation of source records can drive numeric answers across
a tolerance band. That's a different goal than the base structured-data
suite (#18), which tests capability without an adversary. Keeping them here
makes the upstream/local split explicit and avoids polluting the upstream
trajectory tree with RedForge-specific test cases.
