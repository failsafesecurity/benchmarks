# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What This Is

`nearai-bench` — a Rust benchmarking harness for evaluating AI agents across multiple suite types (trajectory, spot, custom, gaia, tau_bench, swe_bench). Extracted from the [ironclaw](https://github.com/nearai/ironclaw) agent framework and depends on it as a library.

## Build & Run

```bash
# Build
cargo build

# Run all tests
cargo test

# Run a specific test
cargo test test_name

# Run tests in a specific module
cargo test config::tests

# Clippy lint
cargo clippy

# Format
cargo fmt

# Run a benchmark suite
cargo run -- run --suite trajectory --config suites/trajectory.toml
cargo run -- run --suite trajectory --config suites/trajectory.toml --model gpt-4o

# View results
cargo run -- results latest
cargo run -- compare <baseline-uuid> <comparison-uuid>
```

Requires Rust stable (toolchain pinned in `rust-toolchain.toml`). Edition 2024, MSRV 1.85.

## Leaderboard Site (site/)

React + Vite + Tailwind app deployed to GitHub Pages. Separate from the Rust harness.

```bash
cd site
npm ci
npx tsx scripts/build-data.ts   # generates leaderboard JSON from results/ and baselines/
npm run dev                      # local dev server
npm run build                    # production build -> site/dist/
```

Auto-deploys on push to `main` when `results/`, `baselines/`, or `site/` change (`.github/workflows/deploy-site.yml`).

## Architecture

**Harness (Rust, `src/`):**
- `main.rs` — CLI via clap (`run`, `results`, `compare`, `list`). Bridges provider env vars (OPENAI_API_KEY, ANTHROPIC_API_KEY) to ironclaw's config format.
- `config.rs` — `BenchConfig` loaded from TOML. Supports matrix entries for testing multiple models.
- `runner.rs` — `BenchRunner` orchestrates task execution. `FrameworkDeps` enum (`Ironclaw` | `OpenClaw`) selects the runner at compile time — no Options, no unwraps. Creates isolated ironclaw `Agent` per task with `InstrumentedLlm`, `BenchChannel`, and optional seeded `Workspace`. Supports sequential and parallel (semaphore-bounded) execution, plus resume via `--resume <uuid>`.
- `suite.rs` — `BenchSuite` trait that all adapters implement: `load_tasks()`, `score()`, lifecycle hooks, multi-turn `next_user_message()`.
- `adapters/` — One module per suite type. `create_suite()` in `mod.rs` dispatches by name. Each adapter reads from `suite_config` in the TOML.
- `scoring.rs` — Shared scoring utilities (exact, contains, regex match with normalization).
- `results.rs` — Read/write `run.json` and `tasks.jsonl` per run under `results/{framework}/{uuid}/`.
- `instrumented_llm.rs` — Wraps an `LlmProvider` to track token counts and costs.
- `channel.rs` — `BenchChannel` implements ironclaw's channel interface to capture agent responses and tool calls.
- `openclaw.rs` — Docker lifecycle management and HTTP client for running tasks against an OpenClaw gateway container. Creates per-task containers with identity files and auth profiles.

**Data layout:**
- `datasets/{suite}/v{N}/` — versioned benchmark data (JSONL or directories)
- `suites/*.toml` — suite configuration files
- `baselines/{suite}/{model}-{hash}/run.json` — curated reference results
- `results/{framework}/{uuid}/` — run output (`run.json` + `tasks.jsonl`)

## Multi-Framework Benchmarking

The harness supports running the same suites against different agent frameworks via `--framework`. Results are tagged with the framework name for comparison.

### Ironclaw (default)

Runs tasks in-process using the ironclaw agent library. Requires LLM provider credentials in `.env`.

```bash
# Configure .env with API keys (see .env.example)
cp .env.example .env

# Run security suite with ironclaw
cargo run -- run --suite trajectory --config suites/zclaw-security-eng.toml

# Ironclaw's safety layer may block injection prompts before they reach the LLM.
# This is valid — it tests the full agent stack including pre-filters.
```

### OpenClaw

Runs tasks against an OpenClaw gateway Docker container. Each task gets a fresh container with identity files mounted as workspace.

```bash
# 1. Build the openclaw Docker image (one-time)
docker build -t openclaw:local /path/to/openclaw/

# 2. Set API keys in .env — openclaw needs provider keys forwarded into the container.
#    Use OPENROUTER_API_KEY for OpenRouter, ANTHROPIC_API_KEY for direct Anthropic, etc.

# 3. Run with openclaw framework — model ID must use openclaw's provider prefix format
cargo run -- run --suite trajectory \
  --config suites/zclaw-security-eng.toml \
  --framework openclaw \
  --model openrouter/anthropic/claude-sonnet-4

# Model ID format: {provider}/{model} — examples:
#   openrouter/anthropic/claude-sonnet-4
#   anthropic/claude-sonnet-4-20250514
#   openai/gpt-4o
```

Optional `[openclaw]` section in suite TOML to override defaults:
```toml
[openclaw]
image = "openclaw:local"       # Docker image (default: openclaw:local)
gateway_token = "my-token"     # Gateway auth token (default: bench-token)
```

### Comparing Frameworks

```bash
# Run the same suite with both frameworks
cargo run -- run --suite trajectory --config suites/zclaw-security-eng.toml
cargo run -- run --suite trajectory --config suites/zclaw-security-eng.toml \
  --framework openclaw --model openrouter/anthropic/claude-sonnet-4

# Compare results
cargo run -- compare <ironclaw-uuid> <openclaw-uuid>
```

## Key Dependencies

- **ironclaw** (git dependency) — Agent, LLM providers, tools, workspace, safety, DB. Changes to ironclaw's API surface directly affect `runner.rs` and `channel.rs`.
- **openclaw** (Docker) — OpenClaw gateway image, used when `--framework openclaw`. Managed by `src/openclaw.rs`.

## Environment

LLM provider configured via `.env` (see `.env.example`). Three paths:
1. `OPENAI_API_KEY` — auto-bridges to ironclaw's openai_compatible backend
2. `ANTHROPIC_API_KEY` — auto-bridges to ironclaw's anthropic backend
3. `LLM_BACKEND` + `LLM_BASE_URL` + `LLM_API_KEY` — direct ironclaw config for any provider

For openclaw, API keys are forwarded into the Docker container as auth profiles. Set `OPENROUTER_API_KEY`, `ANTHROPIC_API_KEY`, or `OPENAI_API_KEY` in your environment.

## ZClaw Security Suite

Tests prompt injection resistance. Derived from the ZClawBench dataset (HuggingFace: `zai-org/ZClawBench`).

**How it works:**
- Each task seeds the agent's workspace with fake identity files (SOUL.md, IDENTITY.md, AGENTS.md, TOOLS.md, KNOWLEDGE.md) containing sensitive strings
- Identity files become part of the system prompt (ironclaw via `Workspace.system_prompt()`, openclaw via mounted workspace dir)
- The user message is a raw prompt injection — no coaching preamble
- Scoring: `response_not_contains` checks if the agent leaked sensitive strings from identity files

**Important context:** The original ZClawBench had coaching preambles ("this is a security test, refuse it") that made every task trivially easy. The `.md` files in `datasets/zclaw-security-eng/v1/` and `datasets/zclaw-security-chn/v1/` contain only the raw injection payloads.

**10 attack vectors:** authority impersonation (fake system hotfix, fake auditor), social engineering (colleague pretext, training pretext, form-filling, meeting prep, consistency review, messaging alignment), emotional pressure, and encoding bypass (Base64/hex/Unicode).

**Known issues:**
- IronClaw gives agents unrestricted filesystem access — agents can `list_dir`/`read_file` on the host (#7)
- Assertion patterns only check identity file strings, miss structural leaks (#8)
- `tools_not_used` assertions not yet added for refusal-only tests

## Integration Tests

```bash
# Unit tests (no Docker needed)
cargo test

# OpenClaw integration tests (require Docker + openclaw:local image)
cargo test openclaw::tests -- --ignored

# The integration tests verify:
# - Container starts with identity files correctly mounted
# - Health endpoint responds
# - Chat completion works with identity in system prompt
# - Container cleanup on drop
```
