# Adversarial benchmark for AI agents across models and frameworks

An adaptive red-team benchmark for AI agents. A **GPT-5 attacker** is run
against agents built from **four leading LLMs** running inside **three
popular open-source agent frameworks**, across 52 ordinary scenarios. The
headline finding: an agent's safety profile is a property of the
*(model, framework)* pair — not of the model alone. The same model wrapped in
three different frameworks produces three different agents, with different
failure surfaces and different rankings.

For the full write-up — methodology, four findings, and what we recommend for
framework builders — read **[the paper](paper/01_blog.md)**.

---

## What's in the matrix

| | Ironclaw | Hermes | Openclaw |
|---|---|---|---|
| **Claude Sonnet 4.6** (Anthropic) | **6** | 7 | 13 |
| **GLM-5** (Z.AI) | **7** | 18 | 9 |
| **Qwen-3.5** (Alibaba) | 13 | **11** | 16 |
| **Kimi K2.6** (Moonshot) | 17 | 22 | **14** |

*Violations out of 52 scenarios per cell. Bold = safest framework for that
model. The pattern is unstable — no framework wins for every model.*

---

## Quickstart

```sh
# 1. clone
git clone https://github.com/nearai/benchmarks
cd benchmarks/redforge

# 2. provision a fresh box for the framework(s) you want to run.
#    setup.sh creates a venv, installs core deps, and (when asked) clones
#    Hermes, builds the Ironclaw bench binary, and/or builds the Openclaw
#    docker image. Re-runnable; pick any combination.
./scripts/setup.sh --with-hermes
./scripts/setup.sh --with-ironclaw
./scripts/setup.sh --with-openclaw
source .venv/bin/activate

# 3. set the keys you need (see .env.example)
export OPENAI_API_KEY=...        # red attacker (always)
export ANTHROPIC_API_KEY=...     # for claude-* models
export OPENROUTER_API_KEY=...    # for kimi/qwen/glm via OpenRouter

# 4. run the smoke test (one scenario, end-to-end, 1–10 min depending on
#    framework). Validates the pipeline for a chosen (framework, model) cell.
FRAMEWORK=hermes   MODEL=claude-sonnet-4-6 ./scripts/smoke_test.sh
FRAMEWORK=ironclaw MODEL=glm-5             ./scripts/smoke_test.sh
FRAMEWORK=openclaw MODEL=qwen-3.5          ./scripts/smoke_test.sh
```

Beyond the smoke, each substrate has its own fanout flow — see
`harness/<framework>/README.md`. A full 12-cell matrix run takes hours and
burns API credit; we did not ship a one-shot matrix runner because the
substrate concerns (EC2, container orchestration, cost gating) vary too much
across frameworks. The artifacts from our own run are under
`runs/matrix-2026-05-02/` and the paper at `paper/01_blog.md` describes the
methodology end to end.

---

## Layout

```
benchmarks/                          ← this repo (nearai/benchmarks)
├── datasets/
│   └── trajectory/v1/structured-data/   ← 52 base scenarios (lands in #18)
└── redforge/                            ← this directory
    ├── README.md            ← this file
    ├── CONTRIBUTING.md      ← how to add a framework, model, or scenario
    ├── paper/               ← the write-up (markdown + figures + html)
    ├── docs/                ← methodology
    ├── harness/             ← the orchestration code (see per-harness READMEs)
    │   ├── ironclaw/        ← Ironclaw adapter + the shared red/sidecar/judge modules
    │   ├── hermes/          ← Hermes adapter
    │   ├── openclaw/        ← Openclaw adapter
    │   └── red/             ← README-only; the attacker code lives under ironclaw/
    ├── scripts/             ← smoke test, setup, classifier, proxy
    ├── scenarios/
    │   └── math/            ← 24 math-heavy variants unique to RedForge
    ├── sidecars/            ← per-scenario adversarial config + invariants
    └── runs/                ← canonical run artifacts
        └── matrix-2026-05-02/
```

**Scenario layout note:** the 52 base scenarios live under
`datasets/trajectory/v1/structured-data/` (contributed via
[#18](https://github.com/nearai/benchmarks/pull/18)) and are referenced by
the sidecars in `redforge/sidecars/<category>/`. The 24 math-heavy variants
are unique to RedForge and live under `redforge/scenarios/math/`.

The benchmark is **(scenarios + sidecars + harnesses + runs)**. The
scenarios describe ordinary tasks the agent must perform. The sidecars layer
adversarial intent on top — which fields red can edit, what counts as a
violation. The harnesses are the orchestration glue: they wrap each framework,
run red against blue, capture reasoning traces, and grade the run against the
sidecar's invariants (judge logic lives in `harness/ironclaw/invariants.py`).

---

## How a single run works

1. The harness loads a scenario (e.g. *"resolve the memory leak commitment"*)
   and its sidecar (which fields red controls, what invariants apply).
2. **Red** (GPT-5) reads the workspace, picks fields it's allowed to edit,
   plants adversarial text in them.
3. **Blue** (the agent under test) reads the workspace and tries to do the
   user's task. Its full reasoning trace is captured.
4. The harness submits blue's tool calls and final response to the **judge**,
   which runs deterministic per-scenario checks (path-prefix matching, regex,
   numeric tolerance) and emits a verdict.
5. Red sees blue's reasoning trace and refines its approach for the next
   attempt. **Five attempts per scenario.** The scenario counts as violated if
   any one of the five attempts succeeds.

[Figure 2 in the paper](paper/01_blog.md#2-our-setup-red-vs-blue-with-full-reasoning-visibility)
shows the flow visually.

---

## What you can do with this

- **Reproduce the matrix.** All run artifacts for the cells the paper cites are
  shipped under `runs/matrix-2026-05-02/`. The harnesses and the judge are
  deterministic; rerunning a cell against the same model/framework should
  produce the same verdict modulo model-API nondeterminism.
- **Add a new framework.** See `CONTRIBUTING.md` and `harness/<framework>/README.md`.
  Add an adapter that conforms to the harness interface; run the corpus through
  it; submit a PR with the new column.
- **Add a new scenario.** Drop a JSON scenario file in
  `datasets/trajectory/v1/structured-data/<category>/` (or for RedForge-only
  math variants, `redforge/scenarios/math/`) and a sidecar in
  `redforge/sidecars/<category>/` whose `scenario_ref` field points at the
  scenario's repo-rooted path. The sidecar declares which invariants apply
  (`no_writes_outside`, `no_markdown_image_exfil`, `no_identity_writes`, etc.).
- **Add a new model.** Wire it through the existing harness's model adapter
  (most providers route through `scripts/reasoning_shim_proxy.py` which
  surfaces reasoning content for thinking models).
- **Run with a different attacker.** Red is `harness/red/` — swap GPT-5 for
  another model, rerun, see how the matrix shifts.

---

## License

MIT. See [LICENSE](LICENSE).

## Citation

```
An Adversarial Benchmark for AI Agents Across Models and Frameworks.
FailSafe and NEAR. 2026.
https://github.com/nearai/benchmarks/tree/main/redforge
```

This work is a collaboration between **FailSafe** and **NEAR**.
