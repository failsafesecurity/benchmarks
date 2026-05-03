# v3-hermes harness TODO

## What's reusable from harness/v2/

- `sidecar.py` — sidecar YAML loader + path rewrite logic. Should work as-is.
- `red_attacker.py` — gpt-5 attacker. Framework-agnostic.
- `invariants.py` — most rules port directly. Two adjustments:
  - Permission invariants reframed as filesystem-path checks (workspace
    tempdir + relative paths) instead of ironclaw's `collections/` namespace.
  - Tool-name vs tool-arg checks: Hermes captures arguments natively, so
    we can drop the `[redforge-dump]` side-channel parsing.
- `orchestrator.py` — attempt loop, scoring, transcript write. Reusable.
- `run.py` — CLI. Reusable with new args (no `--bench-binary`,
  `--bench-config`, `--container`).

## What's new

- `adapter.py` — replaces `harness/v2/runner.py`. Calls
  `AIAgent.run_conversation()` directly. Returns the same RunRecord shape.
- `trace.py` — reads Hermes session DB (or trajectory JSONL) and
  normalizes turns into `{role, content, tool_calls, reasoning}` records
  the existing invariant checker consumes.
- `workspace.py` — stages sidecar's red-controlled files into a temp dir
  the agent treats as cwd. Maps the v2 `collections/<entity>/` paths into
  filesystem locations Hermes' file/terminal toolsets can see.

## First milestone

End-to-end smoke run on a single sidecar (e.g. `signals/query-high-confidence`)
that produces a v2-shape result file. After that, port the rest of v2's
sidecars and run the fanout.

## Open questions

- Which Hermes toolset(s) most closely mirror ironclaw's read/write
  semantics? The agent needs `memory_read`/`memory_search`/`memory_write`
  equivalents. Hermes' `file` toolset reads/writes the real filesystem —
  the workspace tempdir is the natural sandbox boundary.
- Hermes uses `~/.hermes/state.db` as a global session DB. For
  parallel runs, do we override `HERMES_STATE_DB` per run or use distinct
  session_ids in one DB? Distinct session_ids is simpler.
- Reasoning trace shape: SQLite `reasoning` column vs trajectory JSONL.
  JSONL is probably cleaner since it's already structured per-turn.
