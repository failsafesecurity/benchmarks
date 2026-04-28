# Structured-data scenarios

52 scenarios that test an agent's ability to query, filter, aggregate, and write
structured JSON data stored as workspace files. The agent must use
`memory_tree` + `memory_read` to enumerate and read collection files, then
reason across them.

## Categories

| Category      | Scenarios |
|---------------|-----------|
| commitments   | 12 |
| signals       | 4  |
| decisions     | 3  |
| parked-ideas  | 2  |
| aggregation   | 4  |
| cross-entity  | 5  |
| grocery       | 6  |
| todos         | 5  |
| nanny         | 4  |
| transactions  | 7  |

## Scenario shape

Each scenario is a single JSON file with:

- `setup.identity` — path → content map. `AGENTS.md` is injected into the
  system prompt; everything else is stored in the workspace memory store and
  read by the agent at runtime via `memory_*` tools.
- `turns[0].user_input` — the question the agent answers.
- `turns[0].assertions.response_contains` — substring checks on the final
  response.
- `timeout_secs`, `max_tool_iterations` — per-scenario limits.

All fields are part of the mainline trajectory adapter contract — no new
harness APIs are introduced.

## Item-count tiers

Difficulty scales with the number of items the agent must enumerate:

- 5–10 items (decisions, parked-ideas): comfortably within iteration limits
- 12–13 items (grocery, nanny): borderline
- 60 items (transactions): the agent cannot read every file individually
  within the iteration budget, so it must search rather than enumerate
