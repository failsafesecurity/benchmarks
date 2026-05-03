# Structured Data Benchmark

52 scenarios testing agent ability to manage structured JSON data across 8 entity types.

## Running

```bash
# All scenarios
nearai-bench run --suite trajectory --config suites/structured-data.toml

# By category
nearai-bench run --suite trajectory --config suites/structured-data.toml --tags commitments
nearai-bench run --suite trajectory --config suites/structured-data.toml --tags transactions
```

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

## Storage variants

Scenarios are generated from shared fixtures via `scripts/generate-structured-data.py`.
The generator supports a `--variant` flag that controls how data is seeded:

### `workspace` (default, implemented)

Data is seeded as individual JSON files in `setup.workspace.documents`:
```
collections/{entity_type}/_schema.json
collections/{entity_type}/{id}-{slug}.json
```

The agent uses `memory_tree`, `memory_read`, and `memory_search` to query data.
This tests how well agents can manage structured data using file-per-record storage.

### `collections` (future, not yet implemented)

When IronClaw collection tools land ([nearai/ironclaw#1937](https://github.com/nearai/ironclaw/pull/1937)),
a `collections` variant can seed data via the collection API and use typed
query/mutate tools instead of memory tools. The questions and expected answers
are identical across variants — only the seeding and tool surface change.

To add a new variant:
1. Add a branch to `make_scenario()` that builds the appropriate `setup` block
2. Define what tools are available and how data is seeded
3. Run `python3 scripts/generate-structured-data.py --variant <name>`

## Regenerating scenarios

```bash
# Default (workspace)
python3 scripts/generate-structured-data.py

# Explicit variant
python3 scripts/generate-structured-data.py --variant workspace
```

Scenarios are self-contained — each JSON file inlines all fixture data in its
`setup` block. The generator reads from `scripts/structured-data-fixtures.json`
and produces files that the harness can load independently.

## What this benchmark measures

These scenarios become progressively harder as item counts increase:

- **5-10 items** (decisions, parked ideas): manageable within tool iteration limits
- **12-13 items** (grocery, nanny): borderline
- **60 items** (transactions): impossible to read all files within iteration limits

This exposes the scaling limitation of file-per-record workspace storage for
filtering and aggregation queries. Structured query tools (like collections)
would handle these queries in a single tool call regardless of item count.
