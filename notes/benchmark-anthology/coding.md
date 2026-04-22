# Coding Benchmarks

### SWE-Bench Pro
1. **description**: Scale AI's successor to SWE-Bench Verified, built because Verified saturated (~80% on Claude Opus 4.5) due to training contamination. Top frontier models score ~23% on the public set vs. 70%+ on Verified. Long-horizon, real-world issue-resolution tasks curated to resist contamination; has a private commercial split.
2. **github**: https://github.com/scaleapi/SWE-bench_Pro-os — fully open source (MIT). Dataset on HuggingFace.
3. **tools**: Agent-as-is. Harness supplies the container; you plug in your agent (SWE-agent, mini-swe-agent, or custom).
4. **data**: Per-instance prebuilt Docker images on Docker Hub (`jefzda/sweap-images`) with `dockerhub_tag` per instance. Issue text + repo snapshot + hidden test suite.
5. **tasks**: Public set + commercial private set. Python dominant (~58%) with Dockerfile/shell scaffolding. Repo-level bug fix / feature patch. Patches evaluated end-to-end.
6. **evaluation**: Deterministic. Agent outputs a patch; `swe_bench_pro_eval.py` applies it inside the instance container and runs hidden tests. Pass/fail.
7. **playground**: Docker-based; Modal or local. Same shape as SWE-Bench so the existing `swe_bench` adapter should transplant with minimal effort.
8. **category**: coding — repo-level bug fix / feature patch (agentic).

### SWE-Gym
1. **description**: "The first environment for training real-world SWE agents." 2,438 real tasks from 11 Python repos with executable runtime, pre-configured dependencies, and unit tests. Designed as a training gym but usable as eval — paper shows fine-tuned agents reaching competitive SWE-Bench Verified scores.
2. **github**: https://github.com/SWE-Gym/SWE-Gym — fully open source (Apache-2.0). Dataset on HuggingFace.
3. **tools**: Agent-as-is. Drop-in compatible with OpenHands and MoatlessTools harnesses.
4. **data**: Per-instance prebuilt Docker images on Docker Hub. `SWE-Gym-Lite` subset of 234 instances for faster iteration.
5. **tasks**: 2,438 tasks, Python-only, repo-level. Each task ships with a failing test suite that passes after the correct patch.
6. **evaluation**: Deterministic. Test-suite pass/fail inside the container, SWE-Bench style.
7. **playground**: Docker per instance. Same harness shape as SWE-Bench, so the existing adapter pattern applies directly.
8. **category**: coding — repo-level bug fix / feature patch (agentic, training-oriented).

### R2E-Gym
1. **description**: Scalable executable RL/eval environment. 8.1K problems across 13 repos with auto-synthesized unit tests via the SWE-GEN recipe (no reliance on human PRs). Open-weight agent trained on R2E-Gym hits 51% on SWE-Bench Verified.
2. **github**: https://github.com/R2E-Gym/R2E-Gym — fully open source (Apache-2.0).
3. **tools**: Agent-as-is; ships with SFT trajectory data for training runners.
4. **data**: Docker images ~300-500 MB per instance. `R2E-Gym-Lite` / `R2E-Gym-Full` / SWE-Bench-aligned subsets.
5. **tasks**: 8,100 Python tasks at repo scale. Each has NL description + synthesized equivalence tests.
6. **evaluation**: Deterministic, executable unit tests inside the per-instance container.
7. **playground**: Docker per instance, uv-based install.
8. **category**: coding — repo-level bug fix / feature patch (agentic).

### Multi-SWE-Bench (ByteDance)
1. **description**: Multilingual extension of SWE-Bench — 1,632 curated instances (from 2,456 candidates via 68 expert annotators) across 7 languages. Directly fills SWE-Bench's Python-only gap. Companion Multi-SWE-RL set has 4,723 RL-oriented instances.
2. **github**: https://github.com/multi-swe-bench/multi-swe-bench — fully open source (Apache-2.0). HF dataset.
3. **tools**: Agent-as-is. Ships adapters for Agentless, SWE-agent, OpenHands.
4. **data**: Prebuilt Docker container images per instance, released publicly.
5. **tasks**: 1,632 instances across Java, TypeScript, JavaScript, Go, Rust, C, C++. Real GitHub issues with human-written fixes as ground truth.
6. **evaluation**: Deterministic. Patch applied in container, project-native test suite executed, pass/fail.
7. **playground**: Docker per instance; `make install` + config-driven worker pool.
8. **category**: coding — repo-level bug fix, multilingual (agentic).

### SWE-Lancer
1. **description**: OpenAI's "can LLMs earn $1M on Upwork" benchmark. Real freelance SWE tasks (bug fix + manager-style decision tasks) with payouts attached. Runs offline with `iptables` network isolation. IC SWE tasks graded by end-to-end tests; manager tasks graded against hidden rubrics.
2. **github**: https://github.com/openai/preparedness/tree/main/project/swelancer — open source (MIT). Original repo archived; active fork under preparedness.
3. **tools**: Includes `DummySolver` + `SimpleAgentSolver` reference agents for OpenAI/OpenRouter, but designed for plug-in agents.
4. **data**: Prebuilt per-task Docker images (~14 GB each, 10-20 min build) on Docker Hub under `swelancer` org; monolith image also available.
5. **tasks**: 198 verified tasks (237 in paper minus 39 excluded). Mostly full-stack web (Expensify codebase — React Native, Python). IC tasks + manager "which proposal wins" tasks.
6. **evaluation**: Deterministic end-to-end tests (Playwright-style) for IC tasks; for manager tasks a fixed correct-answer oracle. Payout-weighted scoring.
7. **playground**: Docker, network-isolated, uv-based.
8. **category**: coding — repo-level feature/bug fix + manager-decision hybrid (agentic, economic grounding).

### Commit0
1. **description**: Rebuild 57 real Python libraries from scratch given only specs, documentation, and their test suites. Exercises long-horizon scaffolding, typing, linting — not just patching. Supports Modal for distributed test execution.
2. **github**: https://github.com/commit-0/commit0 — fully open source (MIT). `pip install commit0`.
3. **tools**: CLI-driven. Provides `setup`, `build`, `test`, `lint`, `evaluate` subcommands. Agent writes into a working tree; harness runs tests.
4. **data**: HF dataset `wentingzhao/commit0_combined`. Isolated per-library environments.
5. **tasks**: 57 Python libraries, each with significant test coverage, docs, specs, type checks. Function-level skeletons to be filled in across the whole repo.
6. **evaluation**: Deterministic — library's own test suite + typecheck + lint.
7. **playground**: Local isolated envs or Modal cloud runner. No Docker per instance by default (but isolated venvs).
8. **category**: coding — from-scratch library implementation (agentic, repo-scale authoring).

### Aider Polyglot
1. **description**: Aider's flagship benchmark, built on Exercism exercises across many languages. Focuses on "translate a natural-language request into executable code and edit the file correctly" — tests both reasoning and file-edit format fidelity. Widely cited head-to-head for coding agents.
2. **github**: https://github.com/Aider-AI/aider/tree/main/benchmark (harness) + https://github.com/Aider-AI/polyglot-benchmark (exercises). Fully open source (Apache-2.0).
3. **tools**: Aider-specific harness by default (measures editor-format compliance too); can be adapted. Runs inside Docker for safety.
4. **data**: Exercism polyglot exercise set (C++, Go, Java, JavaScript, Python, Rust, plus others).
5. **tasks**: 225 exercises across 6+ languages. Function/file-level with provided tests.
6. **evaluation**: Deterministic — each exercise's shipped unit tests.
7. **playground**: Docker container (harness explicitly runs LLM code inside a container).
8. **category**: coding — function/file-level multilingual completion (tests edit-format fidelity, not repo-scale).

### LiveCodeBench
1. **description**: Contamination-free competitive-programming benchmark. Continuously scrapes new problems from LeetCode, AtCoder, Codeforces; releases time-windowed slices so you can evaluate only on problems posted after a model's training cutoff. Covers generation, self-repair, test-output prediction, and code execution.
2. **github**: https://github.com/LiveCodeBench/LiveCodeBench — fully open source (MIT). HF dataset + data explorer.
3. **tools**: Agent-as-is; vLLM for local inference, multiprocessing for APIs.
4. **data**: v1 (400 problems, May '23-Mar '24) through v6 (1,055 problems, May '23-Apr '25). Python only.
5. **tasks**: ~1,055 competitive-programming problems with hidden test cases. Function-level generation.
6. **evaluation**: Deterministic — modified APPS checker runs generated code against held-out tests. pass@1 / pass@5.
7. **playground**: Dataset + eval scripts (no Docker per task; execution is inline).
8. **category**: coding — competitive programming, contamination-resistant.

### BigCodeBench
1. **description**: 1,140 SWE-flavored function-generation tasks with complex instructions and many external library calls (far beyond HumanEval). Two splits: `Complete` (docstring-based) and `Instruct` (chat-style). `BigCodeBench-Hard` = 148 high-signal tasks. Not yet saturated.
2. **github**: https://github.com/bigcode-project/bigcodebench — fully open source (Apache-2.0). PyPI package.
3. **tools**: Agent-as-is. Multiple backends: local, e2b sandbox, Gradio, remote API.
4. **data**: HF dataset; pre-generated model samples bundled for reuse.
5. **tasks**: 1,140 Python function-level tasks; 148-task Hard subset. Heavy use of scientific / web / data-wrangling libraries.
6. **evaluation**: Deterministic unit tests in a sandbox.
7. **playground**: Docker image provided for safe exec; also e2b/Gradio/local.
8. **category**: coding — function-level with real-library integration (not agentic, but useful as a baseline).

### Long Code Arena (LCA)
1. **description**: JetBrains' suite of six long-context, repo-scale code tasks: library-based code generation, CI build repair, project-level completion, commit message generation, bug localization, module summarization. CI-build-repair and bug-localization are the most agent-relevant.
2. **github**: https://github.com/JetBrains-Research/lca-baselines — open source (MIT). Datasets on HuggingFace.
3. **tools**: Task-specific — some are pure-eval (summarization), others (CI repair) expect a full agent+harness pipeline.
4. **data**: Per-task HF datasets. Bug-localization: 14,958 points across Python/Java/Kotlin. CI repair: real failing builds with logs.
5. **tasks**: Six sub-benchmarks. Mixed granularity from single commit to full repo. Python, Java, Kotlin.
6. **evaluation**: Mixed. CI-build-repair = deterministic (build passes/fails). Bug localization = file-set precision/recall. Commit message / summarization = LLM-judge or reference-similarity (weaker signal).
7. **playground**: Baselines repo per task; CI-repair sub-task is container-style but not a unified harness.
8. **category**: coding — mixed (repo-level completion, bug localization, build repair). Best single pick for benchmark diversity.

### APPS
1. **description**: Classic competitive-programming benchmark — 10,000 problems across Introductory / Interview / Competition tiers. Still useful for tier-stratified analysis, and its checker is the basis for LiveCodeBench's evaluator.
2. **github**: https://github.com/hendrycks/apps — fully open source (MIT). HF dataset.
3. **tools**: Agent-as-is. Eval script runs generated Python against hidden I/O tests.
4. **data**: ~1.3 GB, 10k problems with tests + reference solutions.
5. **tasks**: 10,000 problems, Python. Stratified: Introductory / Interview / Competition.
6. **evaluation**: Deterministic — stdin/stdout match on hidden tests. Well-known issue: non-robust tests (EvalPlus-style problems) inflate scores.
7. **playground**: Dataset + eval scripts; no Docker. Known contamination concerns on the easy tier (frontier models near-saturate Introductory). Competition tier still signal-bearing.
8. **category**: coding — competitive programming, function-level. Deprioritize relative to LiveCodeBench (contamination).

### CRUXEval
1. **description**: Code-reasoning benchmark — CRUXEval-I (predict input given output) and CRUXEval-O (predict output given input) on 800 short Python functions. Not a code-generation test; a code-execution-reasoning test. Useful as a cheap smoke test of an agent's ability to simulate code mentally.
2. **github**: https://github.com/facebookresearch/cruxeval — fully open source (MIT).
3. **tools**: Agent-as-is. Pure text-in / text-out.
4. **data**: 800 Python functions with I/O pairs, shipped as JSONL.
5. **tasks**: 800 problems, Python. Function-level, all short (<1 min for a human).
6. **evaluation**: Deterministic — generated input/output executed and compared to ground truth. pass@1 / pass@5.
7. **playground**: Scripts + HF/OpenAI inference; no Docker.
8. **category**: coding — code-execution reasoning (not generation). Complementary signal, but partially saturated on frontier models — use for cheap regression detection.

### KernelBench
1. **description**: Stanford's "Can LLMs write efficient GPU kernels?" benchmark (ICML 2025). Tasks models with generating correct and fast CUDA (or other DSL) kernels for PyTorch programs on a target GPU. Best frontier models match PyTorch performance in <20% of cases — nowhere near saturated. Introduces the `fast_p` metric (fraction of generated kernels that are correct AND faster than baseline by threshold p).
2. **github**: https://github.com/ScalingIntelligence/KernelBench — open source (MIT). A `KernelBench-v2` fork (Lossfunk) adds Torch-to-Triton problems.
3. **tools**: Agent-as-is; harness provides PyTorch reference + hardware-executed profiling. Iterative-refinement (execution + profiler feedback) loops supported.
4. **data**: 250 PyTorch tasks, structured into 4 levels. Reference implementations shipped; no Docker, but needs a CUDA-capable GPU at eval time.
5. **tasks**: Level 1 (100 single ops: convs, matmuls), Level 2 (100 fused ops), Level 3 (50 full architectures: MobileNet, MiniGPT), Level 4 (20 HuggingFace optimization tasks).
6. **evaluation**: Deterministic — compiled kernel must produce numerically correct outputs AND run faster than the PyTorch baseline. Pass gated on both correctness and speedup threshold.
7. **playground**: Local eval with a CUDA GPU. No per-task container; measures real hardware throughput.
8. **category**: coding — GPU kernel synthesis (niche, hardware-grounded, not saturated).

### TritonBench
1. **description**: First comprehensive benchmark for Triton operator generation (ACL Findings 2025). Two channels: `TritonBench-G` (184 production kernels harvested from high-starred GitHub repos — attention, matmul, softmax, normalization, fused/pipelined) and `TritonBench-T` (166 synthetic tasks built by fusing PyTorch operators). Current code LLMs struggle badly — clear headroom.
2. **github**: https://github.com/thunlp/TritonBench (per arxiv 2502.14752) — open source.
3. **tools**: Agent-as-is. Ships reference PyTorch implementations + evaluation harness for correctness + performance comparison.
4. **data**: 350 kernel-generation tasks. Real-world and synthetic Triton operators.
5. **tasks**: Triton (Python-like GPU DSL) kernel generation. Function-level, but with performance constraints.
6. **evaluation**: Deterministic. Metrics: CodeBLEU similarity, kernel throughput (memory BW, FLOPs), GPU efficiency (fraction of theoretical peak). Correctness gate first, then performance.
7. **playground**: Requires CUDA GPU + Triton. Similar shape to KernelBench but DSL-specific.
8. **category**: coding — Triton DSL kernel synthesis (niche, GPU-grounded, complementary to KernelBench).

### MLE-Bench
1. **description**: OpenAI's benchmark for AI agents doing machine-learning engineering. 75 curated Kaggle competitions covering real ML engineering (train models, prepare datasets, run experiments, iterate). Graded against the public Kaggle leaderboards — agents get bronze/silver/gold medals. Highly agentic, long-horizon. Best setup (o1-preview + AIDE) earns a medal in 16.9% pass@1 / 34.1% pass@8.
2. **github**: https://github.com/openai/mle-bench — open source.
3. **tools**: Agent-as-is. Ships reference scaffolds (AIDE, OpenHands, MLAB) but designed for plug-in agents.
4. **data**: 75 Kaggle competitions with problem statements, datasets, evaluation scripts, and reference submissions. Runs inside a Docker image that mirrors a Kaggle-like env.
5. **tasks**: 75 ML engineering tasks. Heavy use of pandas, scikit-learn, PyTorch, and domain libraries. Long time budgets (hours-to-days of agent time).
6. **evaluation**: Deterministic. Submission CSV scored by the competition's grading script against Kaggle's human leaderboard → medal tier or percentile. pass@k over multiple attempts supported.
7. **playground**: Docker per run, internet access optional (can simulate offline). Adapter shape is similar to SWE-Bench but with much longer time budgets.
8. **category**: coding — ML engineering (highly agentic, long-horizon, economic grounding).

### PaperBench
1. **description**: OpenAI's "can agents replicate AI research from scratch" benchmark (ICML 2025). Agents must replicate 20 ICML 2024 Spotlight/Oral papers — read the paper, write code, run experiments. 8,316 gradable sub-tasks organized into hierarchical rubrics co-developed with paper authors. Claude 3.5 Sonnet scores ~21%, top ML PhDs score ~41%. Complements MLE-Bench on the research-engineering side.
2. **github**: https://github.com/openai/preparedness/tree/main/project/paperbench — open source (MIT).
3. **tools**: Agent-as-is. Ships BasicAgent + IterativeAgent reference scaffolds. LLM-as-judge auto-grades rubric leaves (with a separate JudgeEval benchmark to validate the judge).
4. **data**: 20 papers + curated code releases + hierarchical rubric trees (8,316 leaf criteria). Each paper has a per-instance container.
5. **tasks**: Long-horizon scientific reproduction. Python + ML stack. Time budgets run into many hours.
6. **evaluation**: Semi-deterministic — rubric-leaf grading is done by a calibrated LLM judge; rubrics are concrete enough (compiles, runs, produces number X within tolerance Y) that signal is reliable. Replication-score is the weighted rubric pass rate.
7. **playground**: Docker per task; no internet to arxiv/GitHub at eval time (prevents the agent from just cloning the authors' repo).
8. **category**: coding — research reproduction (highly agentic, long-horizon, LLM-judge-assisted).

### SWE-Smith
1. **description**: Stanford/Princeton toolkit + dataset (NeurIPS 2025 D&B Spotlight) for *generating* SWE-agent training data at scale. Given any Python codebase, SWE-Smith auto-builds an exec env and synthesizes hundreds-to-thousands of test-breaking task instances. Released 50k instances across 128 repos — an order of magnitude larger than SWE-Gym. Trained SWE-agent-LM-32B to 40.2% pass@1 on SWE-Bench Verified (SOTA open-source).
2. **github**: https://github.com/SWE-bench/SWE-smith — open source. Site: https://swesmith.com/.
3. **tools**: Generates evaluable task instances from arbitrary Python repos. Eval harness is SWE-Bench-compatible, so the existing `swe_bench` adapter works.
4. **data**: 50,000 task instances, 128 Python repos, all with per-instance containers and auto-synthesized breaking tests. Designed for RL / SFT training but usable directly as eval.
5. **tasks**: 50k tasks. Python-only currently. Each: break test(s) in real repo, patch to fix.
6. **evaluation**: Deterministic, SWE-Bench-style — patch applied in container, project test suite run, pass/fail.
7. **playground**: Docker per instance, same pattern as SWE-Gym / SWE-Bench. Compatible with existing harness.
8. **category**: coding — repo-level bug fix (agentic, training-scale). Use as training data OR as a large contamination-resistant eval pool.

### SWE-PolyBench
1. **description**: Amazon's 2025 multilingual successor to SWE-Bench (arXiv Apr 2025). 2,110 repo-level instances from 21 repos across Java, JavaScript, TypeScript, and Python. Includes file-level + CST-node-level retrieval metrics on top of patch-pass, so you get much richer diagnostics than SWE-Bench. Directly complements Multi-SWE-Bench (different language set, different retrieval metrics).
2. **github**: https://github.com/amazon-science/SWE-PolyBench — open source. HF dataset `AmazonScience/SWE-PolyBench`.
3. **tools**: Fully-automated evaluation harness. Drop-in compatible with SWE-agent, OpenHands, Agentless scaffolds.
4. **data**: 2,110 total; `SWE-PolyBench_Verified` (384 human-verified) and `SWE-PolyBench500` (500 stratified). Per-instance Docker images.
5. **tasks**: Bug fixes, feature additions, refactors. Covers Java, JS, TS, Python. Heavy emphasis on multi-file edits (perf degrades sharply when >3 files need changes).
6. **evaluation**: Deterministic test-suite pass + file-level retrieval precision/recall + CST-node-level retrieval metrics.
7. **playground**: Docker per instance; harness mirrors SWE-Bench, so the existing adapter transplants with language-specific tweaks.
8. **category**: coding — multilingual repo-level bug fix (agentic). Complementary to Multi-SWE-Bench.

### Nemotron-CORTEXA (as eval scaffold)
1. **description**: NVIDIA's SOTA SWE-agent (ICML 2025) — 68.2% on SWE-Bench Verified at $3.28/problem. Not a new benchmark per se but a reference agent + eval harness that ships NV-EmbedCode (code-embedding model tuned for bug→code localization) and a reference scaffold for localization + multi-solution + LLM-judge selection. Useful as a strong baseline to run head-to-head against ironclaw/openclaw agents on SWE-Bench / SWE-Bench Pro.
2. **github**: https://github.com/NVIDIA/Nemotron-CORTEXA — open source. HF dataset `nvidia/Nemotron-SWE-v1` (RL trajectories).
3. **tools**: Full agent scaffold — localization → patch generation → unit-test synth → LLM-judge selection. Works over the SWE-Bench instance format.
4. **data**: Runs on standard SWE-Bench Verified + Lite + Pro instance sets; ships fine-tuning data as separate artifacts.
5. **tasks**: SWE-Bench-compatible (Python repo-level bug fix).
6. **evaluation**: Delegates to SWE-Bench harness (deterministic, test-suite).
7. **playground**: Docker per instance (SWE-Bench style). Primary value: a high-quality, open-weight baseline to compare frameworks against.
8. **category**: coding — SWE baseline scaffold (use as comparison baseline, not as a fresh task set).

### SciCode
1. **description**: Scientist-curated research coding benchmark (NeurIPS 2024 D&B). 80 main problems decomposed into 338 sub-problems across 16 natural-science sub-fields (math, physics, chemistry, biology, materials science). Drawn from scripts actually used in published research. Extremely challenging: Claude 3.5 Sonnet solves only ~4.6% of main problems; GPT-4o ~1.5%. Strong scientific-reasoning signal, low contamination risk.
2. **github**: https://github.com/scicode-bench/SciCode — open source. Site: https://scicode-bench.github.io/.
3. **tools**: Agent-as-is. Eval script runs generated Python against scientist-written unit tests.
4. **data**: 80 problems × ~4 sub-problems each. Python, with heavy scipy/numpy usage and domain-specific libraries.
5. **tasks**: Multi-step scientific scripts. Sub-problems chain (later sub-problems depend on earlier ones), so partial credit is well-defined.
6. **evaluation**: Deterministic — scientist-authored unit tests. Scored at sub-problem and main-problem granularity.
7. **playground**: Scripts + HF-style inference; no Docker. Dependencies = standard scientific Python stack.
8. **category**: coding — scientific research code (not agentic by default, but very low-saturation). Good complement to SUPER/PaperBench on a shorter time budget.

### USACO
1. **description**: Princeton's Olympiad-programming benchmark (ICLR 2024). 307 USA Computing Olympiad problems across Bronze / Silver / Gold / Platinum tiers, with high-quality hidden unit tests, reference solutions, and official analyses. Problems demand creative ad-hoc algorithmic reasoning — unlike LeetCode-style interview problems. GPT-4 scores 8.7% zero-shot CoT pass@1, rising to 20.2% with retrieval+reflection inference-time methods.
2. **github**: https://github.com/princeton-nlp/USACO — open source. Site: https://princeton-nlp.github.io/USACOBench/.
3. **tools**: Agent-as-is. Ships inference-method implementations (retrieval over episodic knowledge, self-reflection, both combined). HAL leaderboard tracks agent harnesses.
4. **data**: 307 problems with narratives, hidden tests, reference code, and official editorials.
5. **tasks**: Competitive-programming algorithmic problems, tier-stratified. Python (but language-agnostic if harness compiles).
6. **evaluation**: Deterministic — stdin/stdout match on hidden tests. pass@1 / pass@k.
7. **playground**: Dataset + eval scripts; no Docker. Light compute.
8. **category**: coding — competitive programming (olympiad-level, contamination-aware tiering). Complements LiveCodeBench with creativity-heavy problems.

### LiveCodeBench Pro
1. **description**: Successor to LiveCodeBench (NeurIPS 2025) built by international olympiad medalists. 584 problems from Codeforces, ICPC, IOI, curated up to Apr 2025, continuously updated to avoid contamination. Every problem is annotated with algorithm category and every failed model submission is line-by-line diagnosed. Best model scores 53% pass@1 on medium and 0% on hard — much harder than original LiveCodeBench.
2. **github**: Leaderboard + data: https://livecodebenchpro.com/. Paper: arXiv 2506.11928.
3. **tools**: Agent-as-is; fits into the existing LiveCodeBench harness shape.
4. **data**: 584 olympiad-grade problems with hidden tests + algorithmic category tags + expert failure analyses.
5. **tasks**: Competitive-programming, olympiad-level. Function-level generation with heavy reasoning load.
6. **evaluation**: Deterministic, stdin/stdout. pass@1 / pass@k per difficulty tier.
7. **playground**: Dataset + inline exec (no Docker). Expert annotations give rich failure-mode diagnostics for post-hoc analysis.
8. **category**: coding — olympiad-level competitive programming. Use alongside LiveCodeBench for a harder, expert-annotated tier.

### BIRD-SQL (+ LiveSQLBench / BIRD-Critic)
1. **description**: BIRD is the canonical text-to-SQL benchmark (NeurIPS 2023, actively extended through 2026). 12,751 question-SQL pairs across 95 databases (33 GB) spanning 37+ professional domains, evaluated against SQLite / MySQL / PostgreSQL. The 2025 family extensions are the agent-relevant parts: `LiveSQLBench` (live DB-level tasks, best model 47.78% success), `BIRD-Critic / SWE-SQL` (issue-resolution style SQL debugging), `BIRD-Interact` (conversational + agentic SQL). Fills a major gap — the anthology has no SQL/DSL bench.
2. **github**: https://github.com/bird-bench (main), https://github.com/bird-bench/BIRD-CRITIC-1, https://github.com/bird-bench/mini_dev. Site: https://bird-bench.github.io/.
3. **tools**: Agent-as-is for base BIRD; BIRD-Interact is explicitly agentic (tool calls against a live DB).
4. **data**: BIRD = 12,751 pairs, 95 DBs. LiveSQLBench-Base-Full-V1 = 600 tasks over 18 end-user DBs. BIRD-Critic = SQL-issue resolution.
5. **tasks**: SQL generation from NL, SQL debugging, multi-turn SQL conversation, agentic SQL (tool-augmented).
6. **evaluation**: Deterministic — execution-accuracy against hidden test queries. Soft-F1 + R-VES metrics included to reduce bias.
7. **playground**: DBs shipped; needs SQLite/MySQL/Postgres. No Docker required for base bench; agentic variants run against live DBs.
8. **category**: coding — SQL / database DSL (agentic variants available, covers a DSL the anthology is missing).

Notes on benchmarks deliberately demoted or skipped:

- **HumanEval / HumanEval+ / MBPP / MBPP+** — near-saturated (frontier >95% on HumanEval; EvalPlus drops are smaller each year). Keep only as sanity checks.
- **CodeContests** (DeepMind) — archived Dec 2024, so strictly read-only. Still usable as a dataset but not recommended as a primary harness target.
- **RepoBench** — no executable verification (exact-match / edit-similarity / CodeBLEU only). Fails our "deterministic unit tests" criterion.
- **ClassEval** — class-level Python with good tests, but 100 tasks is small and it is not agentic. Use only as a narrow sub-signal.
- **SWE-Bench / SWE-Bench Verified / SWE-Bench Multimodal** — already supported by this harness and, per OpenAI's own statement, Verified no longer measures frontier capability due to contamination.
