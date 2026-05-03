# `scenarios/`

52 base scenarios (plus 24 math-heavy variants) testing agent ability to
manage structured JSON data across 8 entity types. Scenarios were
adapted from the upstream structured-data work in
[nearai/benchmarks#14](https://github.com/nearai/benchmarks/pull/14);
the adversarial layer is added by the sidecar files under
[`../sidecars/`](../sidecars/), and the runner is the redforge harness
under [`../harness/`](../harness/).

## Running

Through redforge's smoke driver:

```sh
FRAMEWORK=hermes \
MODEL=claude-sonnet-4-6 \
SCENARIO=commitments/crud-resolve \
  ../scripts/smoke_test.sh
```

Or through the per-framework runner directly — see
[`../harness/<framework>/README.md`](../harness/).

## Entity types

| Type | Items | Description |
|------|-------|-------------|
| commitments | 10 | Team work items with status, urgency, owner |
| signals | 10 | Detected obligations from slack/email/meetings |
| decisions | 5 | Recorded decisions with context and participants |
| parked_ideas | 5 | Low-priority ideas for later |
| grocery | 12 | Shopping list with store, priority, quantity |
| todos | 8 | Tasks with due dates, priority, completion status |
| nanny_hours | 13 | Childcare shift logs with hours |
| transactions | 60 | Financial records with categories and amounts |

## Scenario shape

Each scenario is a single JSON file that inlines all fixture data in
its `setup.workspace.documents` block:

```
collections/{entity_type}/_schema.json
collections/{entity_type}/{id}-{slug}.json
```

The agent uses `memory_tree`, `memory_read`, and `memory_search` to
query the data. Files are self-contained — no external fixtures, no
runtime generation step.

## Scaling

The scenarios become progressively harder as item counts increase:

- **5–10 items** (decisions, parked_ideas): manageable within tool
  iteration limits.
- **12–13 items** (grocery, nanny_hours): borderline.
- **60 items** (transactions): impossible to read all files within
  iteration limits; the agent must rely on `memory_search` or fail.

This exposes the scaling limitation of file-per-record workspace
storage for filtering and aggregation queries. Structured query tools
(planned upstream in
[nearai/ironclaw#1937](https://github.com/nearai/ironclaw/pull/1937))
would handle these queries in a single tool call regardless of item
count.

## Regenerating

Scenario JSON files are produced by the generator in
[nearai/benchmarks#14](https://github.com/nearai/benchmarks/pull/14)
and copied here. Sidecars are generated from scenarios by
[`../scripts/generate_sidecars.py`](../scripts/generate_sidecars.py).
