## Summary

A new top-level `redforge/` directory adds an adversarial benchmark for
AI agents: a GPT-5 attacker run against 12 *(model × framework)* agent
configurations across 52 scenarios — 4 leading LLMs (Claude Sonnet 4.6,
GLM-5, Qwen-3.5, Kimi K2.6) inside 3 open-source frameworks (Ironclaw,
Hermes, Openclaw).

The headline finding is that an agent's safety profile is a property
of the **(model, framework)** pair, not of the model alone — see the
matrix in [`redforge/README.md`](redforge/README.md) and the full
write-up in [`redforge/paper/01_blog.md`](redforge/paper/01_blog.md).

## Depends on

- **[#18](https://github.com/nearai/benchmarks/pull/18)** — the 52
  base scenarios live under `datasets/trajectory/v1/structured-data/`,
  contributed by that PR. Sidecars in `redforge/sidecars/` reference
  those paths.

## What's in the patch

- **Harness** — per-framework runners for Ironclaw, Hermes, Openclaw,
  plus a shared GPT-5 red attacker
  (`redforge/harness/ironclaw/red_attacker.py`, framework-agnostic;
  cross-imported by the other two orchestrators).
- **76 sidecars** — adversarial overlay for the 52 base scenarios from
  #18 plus 24 RedForge-only math-heavy variants under
  `redforge/scenarios/math/`. Deterministic invariant checks in
  `redforge/harness/ironclaw/invariants.py`.
- **Setup + smoke** — `redforge/scripts/setup.sh` provisions a fresh
  box for any combination of frameworks;
  `redforge/scripts/smoke_test.sh` validates one cell end-to-end.
- **Run artifacts** — `redforge/runs/matrix-2026-05-02/` contains the
  12-cell results referenced by the paper (per-cell summaries + 2
  verbatim full traces).
- **Paper** — `redforge/paper/01_blog.md` + figures, plus a tiny
  pandoc render script so the shareable HTML stays in sync with the
  markdown.

## Verifying locally

A single smoke takes ~1–10 min per framework and exercises the full
pipeline (run from the repo root after #18 is merged or rebased in):

    ./redforge/scripts/setup.sh --with-hermes
    export OPENAI_API_KEY=... ANTHROPIC_API_KEY=...
    FRAMEWORK=hermes MODEL=claude-sonnet-4-6 ./redforge/scripts/smoke_test.sh

Repeat with `--with-ironclaw` / `FRAMEWORK=ironclaw` and
`--with-openclaw` / `FRAMEWORK=openclaw` to exercise the other two
substrates. Full quickstart and layout in
[`redforge/README.md`](redforge/README.md).

## Replaces

- Closes #17 (single-framework Ironclaw/GLM-5 prototype).

## Test plan

- [ ] #18 merged or rebased in (sidecar scenario_ref paths resolve)
- [ ] `redforge/scripts/setup.sh --with-hermes` provisions cleanly on a fresh box
- [ ] Hermes smoke passes end-to-end
- [ ] Ironclaw smoke passes end-to-end
- [ ] Openclaw smoke passes end-to-end
- [ ] `redforge/paper/figures/figure_*.py` regenerate from `redforge/runs/matrix-2026-05-02/`
