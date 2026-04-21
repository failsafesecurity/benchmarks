# PinchBench Dataset

Task definitions and assets from [PinchBench](https://pinchbench.com) — a benchmarking system for evaluating LLM models as coding agents.

## Source

- **Repository**: https://github.com/pinchbench/skill
- **Website**: https://pinchbench.com
- **License**: MIT
- **Authors**: [Kilo Code](https://kilo.ai) — @olearycrew, @arpitg1991, @DJRHails, @evanjacobson, @iJaack

## Contents

- `v1/tasks/` — task definitions as markdown files with YAML frontmatter
- `v1/assets/` — supporting files (PDFs, spreadsheets, text) referenced by tasks, stored in Git LFS

## Setup

Assets are stored in Git LFS. After cloning, either pull them via LFS:

```bash
git lfs install
git lfs pull
```

Or re-download directly from upstream (does not require LFS):

```bash
scripts/pinchbench_download.sh
```

## Updating

To re-download assets from the latest upstream:

```bash
scripts/pinchbench_download.sh --force
```

## Running

Assets must be present for both frameworks — tasks with `workspace_files: [{source, dest}]` copy from `v1/assets/` into each task workspace, and missing assets degrade scores silently (logged as warnings).

### Ironclaw

```bash
cargo run -- run --suite pinchbench --config suites/pinchbench.toml
```

`.env`:

```bash
# Judge model (suite uses openrouter/anthropic/claude-haiku-4.5)
OPENROUTER_API_KEY=sk-or-...

# Agent provider — pick ONE:

# Option A: OpenAI
OPENAI_API_KEY=sk-...

# Option B: Anthropic direct
ANTHROPIC_API_KEY=sk-ant-...

# Option C: OpenRouter (or any OpenAI-compatible)
LLM_BACKEND=openai_compatible
LLM_BASE_URL=https://openrouter.ai/api/v1
LLM_API_KEY=sk-or-...
LLM_MODEL=anthropic/claude-sonnet-4
```

### OpenClaw

Prereq: `docker build -t openclaw:local /path/to/openclaw/`.

```bash
cargo run -- run --suite pinchbench --config suites/pinchbench.toml \
  --framework openclaw --model openrouter/anthropic/claude-sonnet-4
```

`.env`:

```bash
# Judge model
OPENROUTER_API_KEY=sk-or-...

# Agent provider — set the key matching your --model prefix.
# OPENROUTER_API_KEY above covers openrouter/... models.
# Use ANTHROPIC_API_KEY for anthropic/... or OPENAI_API_KEY for openai/...
```
