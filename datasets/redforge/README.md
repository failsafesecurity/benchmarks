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
# 1. clone and install
git clone https://github.com/nearai/benchmarks
cd benchmarks/datasets/redforge
python -m venv .venv && source .venv/bin/activate
pip install -r requirements.txt

# 2. set the keys you need (see .env.example)
export OPENROUTER_API_KEY=...
export ANTHROPIC_API_KEY=...
# ...

# 3. run the smoke test (one cell, one scenario, ~2 minutes)
./scripts/smoke_test.sh

# 4. run a single (model, framework, scenario) cell end-to-end
python -m harness.run \
  --framework hermes \
  --model claude-sonnet-4-6 \
  --scenario commitments/crud-resolve

# 5. run the full matrix (4 models × 3 frameworks × 52 scenarios; long-running)
./scripts/run_matrix.sh
```

The matrix takes hours and burns API credit. Smoke test first.

---

## Layout

```
datasets/redforge/
├── README.md            ← this file
├── CONTRIBUTING.md      ← how to add a framework, model, or scenario
├── paper/               ← the write-up (markdown + figures + html)
├── docs/                ← methodology, run-artifact inventory
├── harness/             ← the orchestration code (see per-harness READMEs)
│   ├── ironclaw/
│   ├── hermes/
│   ├── openclaw/
│   └── red/
├── scripts/             ← smoke test, fanout runners, classifier, proxy
├── scenarios/           ← 52 base scenarios + 24 math-heavy variants
├── sidecars/            ← per-scenario adversarial config + invariants
├── judges/              ← deterministic per-scenario verdict checkers
└── runs/                ← canonical run artifacts
    └── matrix-2026-05-02/
```

The benchmark is **(scenarios + sidecars + harnesses + judges + runs)**. The
scenarios describe ordinary tasks the agent must perform. The sidecars layer
adversarial intent on top — which fields red can edit, what counts as a
violation. The harnesses are the orchestration glue: they wrap each framework,
run red against blue, capture reasoning traces, and submit to the judges.

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
- **Add a new scenario.** Drop a JSON scenario file in `scenarios/<category>/`
  and a sidecar in `sidecars/<category>/`. The sidecar declares which
  invariants apply (`no_writes_outside`, `no_markdown_image_exfil`,
  `no_identity_writes`, etc.).
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
https://github.com/nearai/benchmarks/tree/main/datasets/redforge
```

This work is a collaboration between **FailSafe** and **NEAR**.
