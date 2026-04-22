# Benchmark Anthology

Unfiltered candidate list of benchmarks that could be adapted into the ironclaw/openclaw harness. Full cards (8-field format — description, github, tools, data, tasks, evaluation, playground, category) live in each per-category file; this README is the index with quick-scan summary tables.

**Totals: 122 cards across 6 categories** — security (19), coding (23), terminal (15), office work (21), financial (20), browser (24).

**Legend**

- **Eval**: `DET` = deterministic (state diff / unit tests / string match), `JUDGE` = LLM-as-judge rubric, `MIX` = mixed.
- **Env**: `Docker` = per-task container, `Sim` = in-process simulator, `Dataset` = no runtime, `Live` = live web / live markets, `VM` = sandboxed instance (ServiceNow PDI etc.).
- **Fit**: rough integration cost relative to the existing adapter pattern in `src/adapters/`.
  - ★★★ = slots in directly (dataset + scoring, or SWE-Bench-style Docker per instance)
  - ★★ = moderate (needs a new playground class or simulator wiring)
  - ★ = heavy (multi-container orchestration, live web, or bespoke runtime)

Already supported in-harness: SWE-Bench/Verified, τ-bench (retail + airline), GAIA, trajectory, spot, custom, zclaw-security, pinchbench.

---

## Security — [security.md](security.md)

19 cards. Subcategories: CTF / offensive, prompt-injection, harmful-tool-use, vuln-discovery.

| Benchmark | # Tasks | Eval | Env | Fit | One-liner |
|---|---|---|---|---|---|
| [Cybench](security.md#cybench) | 40 | DET (flag) | Docker/K8s (Kali) | ★★ | Curated pro CTF; de-facto frontier CTF bench, Inspect-integrated. |
| [NYU CTF Bench](security.md#nyu-ctf-bench) | 200 | DET (flag) | Docker-compose | ★★ | Larger-scale CSAW-derived CTF; train/test split. |
| [InterCode-CTF](security.md#intercode-ctf) | 100 | DET (flag) | Docker (gym API) | ★★★ | picoCTF tasks; gym-style step() → easiest CTF adapter. |
| [AutoPenBench](security.md#autopenbench) | 36 | DET + milestones | Docker-compose (Kali + victim) | ★★ | Pentest tasks with milestone partial credit. |
| [3CB](security.md#3cb-catastrophic-cyber-capabilities-benchmark) | 15 | DET (flag) | Docker (Inspect AI) | ★★ | MITRE ATT&CK-mapped offensive CTFs; structured coverage. |
| [AIRTBench](security.md#airtbench) | 70 | DET (flag) | Docker (Crucible) | ★★ | AI-on-AI red-team CTFs — attacking ML systems; unique niche. |
| [AgentDojo](security.md#agentdojo) | 97 × 629 inj. | DET (state) | Sim (Python tools) | ★★★ | Indirect prompt injection, best-in-class; Inspect-integrated. |
| [InjecAgent](security.md#injecagent) | 1,054 | DET (action match) | Dataset (canned tools) | ★★★ | Prompt injection with stubbed tool responses; trivial integration. |
| [BIPIA](security.md#bipia) | ~86k prompts | DET (ASR regex) | Dataset | ★★★ | Microsoft breadth-first indirect prompt injection sweep. |
| [AgentHarm](security.md#agentharm) | 176 + 32 | JUDGE (refusal + semantic) | Sim (stubbed tools) | ★★ | Harmful tool-use jailbreak bench; Inspect-integrated. |
| [HarmBench](security.md#harmbench) | 400 × attacks | JUDGE (classifier) | Scripts | ★★ | De-facto automated red-teaming / refusal-robustness pipeline. |
| [OpenAgentSafety](security.md#openagentsafety) | 350+ | MIX (rules + judge) | Docker (OpenHands) | ★★ | Multi-turn real-tool agent safety; compounding unsafe steps. |
| [RedCode](security.md#redcode) | 4,210 | MIX (rules + judge) | Docker (code exec) | ★★ | Risky-code exec + malware-gen; per-agent-type runners. |
| [AgentPoison](security.md#agentpoison) | hundreds × 3 stacks | DET (ASR + utility delta) | 3 agent harnesses | ★ | Memory/RAG backdoor poisoning — only open bench for this vector. |
| [DoomArena](security.md#doomarena) | variable (inherits) | DET (state + attack flag) | Plug-in layer | ★★ | Attack-injection layer over BrowserGym / τ-Bench / OSWorld. |
| [CyberSecEval 3](security.md#cybersec-eval-3-purplellama) | ~thousands | MIX (static-analysis + judge) | Scripts + shell sandbox | ★★ | Meta's breadth bench: insecure code + phishing + autonomous ops. |
| [CyberGym](security.md#cybergym) | 1,507 | DET (PoC triggers crash) | Docker per CVE | ★ | Real OSS-Fuzz CVE reproduction; uncovered 10 0-days in dev. |
| [SEC-bench](security.md#sec-bench) | dataset grows | DET (PoC + patch) | Docker per instance | ★ | PoC gen + vuln patching on real OSS; SWE-Bench shape for security. |
| [WMDP](security.md#wmdp) | 3,668 MCQ | DET (accuracy) | Dataset (lm-eval) | ★★★ | Hazardous-knowledge MCQ probe (non-agent companion metric). |

---

## Coding — [coding.md](coding.md)

23 cards. Prioritised agentic / repo-scale. Note: SWE-Bench Verified officially deprecated by OpenAI due to training-data contamination.

| Benchmark | # Tasks | Languages | Eval | Env | Fit | One-liner |
|---|---|---|---|---|---|---|
| [SWE-Bench Pro](coding.md#swe-bench-pro) | Public + private | Python-heavy | DET (hidden tests) | Docker/instance | ★★★ | Verified's contamination-resistant successor; top models ~23%. |
| [SWE-Gym](coding.md#swe-gym) | 2,438 | Python | DET (tests) | Docker/instance | ★★★ | Training-oriented gym; drop-in with OpenHands/Moatless. |
| [SWE-Smith](coding.md#swe-smith) | 50,000 | Python | DET (tests) | Docker/instance | ★★★ | Training-scale SWE-Bench-shape data (128 repos, 10× SWE-Gym). |
| [R2E-Gym](coding.md#r2e-gym) | 8,100 | Python | DET (synth tests) | Docker/instance | ★★★ | SWE-GEN-synthesized tests, 13 repos. |
| [Multi-SWE-Bench](coding.md#multi-swe-bench-bytedance) | 1,632 | Java, TS, JS, Go, Rust, C, C++ | DET (native tests) | Docker/instance | ★★★ | Fills SWE-Bench's Python monoculture. |
| [SWE-PolyBench](coding.md#swe-polybench) | 2,110 (384 verified) | Java, JS, TS, Python | DET (tests + retrieval) | Docker/instance | ★★★ | Multilingual SWE-Bench; includes CST-node retrieval metrics. |
| [SWE-Lancer](coding.md#swe-lancer) | 198 | TS/Python/React Native | DET (Playwright) + oracle | Docker (isolated net) | ★★ | OpenAI's freelance-SWE bench with payouts; 14 GB images. |
| [Commit0](coding.md#commit0) | 57 | Python | DET (lib tests + typecheck + lint) | venv / Modal | ★★ | Rebuild libraries from scratch; long-horizon authoring. |
| [Nemotron-CORTEXA](coding.md#nemotron-cortexa-as-eval-scaffold) | SWE-Bench-shape | Python | DET (delegated) | Docker/instance | — | Not a fresh set — NVIDIA's 68% SOTA scaffold as baseline. |
| [MLE-Bench](coding.md#mle-bench) | 75 | Python (ML stack) | DET (Kaggle → medal) | Docker (24h budget) | ★★ | OpenAI's Kaggle-ML-engineering bench; long-horizon. |
| [PaperBench](coding.md#paperbench) | 20 (8,316 leaves) | Python (ML stack) | JUDGE (calibrated rubric) | Docker, no net | ★ | Replicate ICML Spotlights from scratch; long-horizon research. |
| [Aider Polyglot](coding.md#aider-polyglot) | 225 | C++, Go, Java, JS, Python, Rust | DET (Exercism tests) | Docker | ★★★ | Tests edit-format fidelity across languages. |
| [LiveCodeBench](coding.md#livecodebench) | 1,055 | Python | DET (APPS checker) | Dataset | ★★★ | Contamination-free competitive programming, time-windowed. |
| [LiveCodeBench Pro](coding.md#livecodebench-pro) | 584 | Python | DET (stdin/stdout) | Dataset | ★★★ | Olympiad-grade Codeforces/ICPC/IOI; 0% pass on hard. |
| [USACO](coding.md#usaco) | 307 | Python | DET (I/O) | Dataset | ★★★ | USA Olympiad, tier-stratified; creativity-heavy. |
| [APPS](coding.md#apps) | 10,000 | Python | DET (I/O match) | Dataset | ★★★ | Competitive programming, saturated on easy tier. |
| [BigCodeBench](coding.md#bigcodebench) | 1,140 (148 Hard) | Python | DET (unit tests) | Docker / e2b | ★★★ | Library-heavy function tasks; unsaturated. |
| [CRUXEval](coding.md#cruxeval) | 800 | Python | DET (exec match) | Dataset | ★★★ | Code-execution reasoning (not generation); cheap smoke test. |
| [SciCode](coding.md#scicode) | 80 (338 sub) | Python + scipy | DET (scientist tests) | Dataset | ★★★ | 16 science sub-fields; <5% solved by Claude 3.5. |
| [Long Code Arena](coding.md#long-code-arena-lca) | 6 subtasks | Python, Java, Kotlin | MIX | Per-task | ★★ | CI-repair + bug-localization are agent-relevant. |
| [KernelBench](coding.md#kernelbench) | 250 | CUDA (from PyTorch) | DET (correct + `fast_p`) | Local CUDA GPU | ★★ | GPU kernel synthesis — <20% frontier success. |
| [TritonBench](coding.md#tritonbench) | 350 | Triton DSL | DET (correct + throughput) | Local CUDA GPU | ★★ | Triton kernel gen; complementary to KernelBench. |
| [BIRD-SQL family](coding.md#bird-sql-livesqlbench-bird-critic) | 12,751+ (600 live) | SQL | DET (exec accuracy) | DB backend | ★★ | Fills SQL/DSL gap; agentic variants (BIRD-Interact) available. |

Demoted: HumanEval/MBPP (near-saturated), CodeContests (archived Dec 2024), RepoBench (no exec verification), ClassEval (too small).

---

## Terminal — [terminal.md](terminal.md)

15 cards. Strong overlap with security/CTF (flag-based benches double as shell tasks) — grouped here where the execution model is the terminal itself.

| Benchmark | # Tasks | Eval | Env | Fit | One-liner |
|---|---|---|---|---|---|
| [Terminal-Bench](terminal.md#terminal-bench) | 89 (Core v2.0) | DET (pytest) | Docker + tmux | ★★ | Stanford/Laude bench; sysadmin + ML + security + data wrangling. |
| [InterCode-Bash](terminal.md#intercode-bash) | ~224 | DET (dual-exec state diff) | Docker (gym) | ★★★ | Clean gym API, cleanest execution model for shell agents. |
| [InterCode-SQL](terminal.md#intercode-sql) | ~1,034 | DET (dual-exec row match) | Docker (MySQL) | ★★★ | SQL multi-turn with schema exploration; same gym API as Bash. |
| [BIRD-Interact](terminal.md#bird-interact) | 600 / 300 | DET (DB-state tests) | DB container + user sim | ★★ | Agentic SQL with clarification + CRUD + user simulator. |
| [Spider 2.0 / Spider2-DBT](terminal.md#spider-20-spider2-dbt) | 632 (68 DBT) | DET (rows + dbt tests) | Cloud DB (Snowflake/BQ) + local | ★ | Enterprise SQL workflows incl. dbt repo-level editing. |
| [Cybench](terminal.md#cybench) | 40 | DET (flag) | Docker (Kali) | ★★ | Also listed in security; shell-driven execution. |
| [NYU CTF Bench](terminal.md#nyu-ctf-bench) | 200 | DET (flag) | Docker-compose | ★★ | Also listed in security. |
| [AgentBench-OS](terminal.md#agentbench-os) | ~144 | DET (answer/checker) | Docker (Ubuntu) | ★★★ | Sysadmin Q&A; lightweight, fast smoke test. |
| [NL2Bash-EABench (IBM)](terminal.md#nl2bash-eabench-ibm) | 150 | DET (exec behaviour) | Podman/Docker | ★★★ | Incident-remediation one-shot execution. |
| [CRAB](terminal.md#crab-cross-environment-agent-benchmark) | 120 | DET (graph subgoals) | Ubuntu VM + Android emu | ★ | Cross-env sysadmin + mobile; partial-credit graph scoring. |
| [AIOpsLab](terminal.md#aiopslab) | ~hundreds | DET (per-task-type) | Kubernetes + ChaosMesh | ★ | Cloud SRE fault triage on real microservices. |
| [ITBench](terminal.md#itbench-ibm) | ~60 lite | DET (per scenario) | Kubernetes + cloud | ★ | IBM's SRE + CISO + FinOps enterprise IT bench. |
| [DevOps-Gym](terminal.md#devops-gym) | 4 tracks | DET (per-stage check) | Docker | ★★ | End-to-end DevOps pipeline: build + monitor + fix + test-gen. |
| [MLE-bench](terminal.md#mle-bench) | 75 | DET (medal tier) | Docker (Ubuntu) | ★★ | Long-horizon ML eng in shell (also listed in coding). |
| [SWE-ReX runtime](terminal.md#swe-agent-bash-tasks-via-swe-rex) | substrate | delegated | Docker / Modal / subproc | — | Not a bench; reference bash-in-Docker execution shim. |

Excluded: OSWorld (VMware-heavy + GUI-coupled), raw NL2Bash (string match), OverTheWire Bandit (no open harness), BashBench (not released).

---

## Office work — [office-work.md](office-work.md)

21 cards. Document QA, spreadsheet editing, slides, multi-app workflow, plus long-context / RAG.

| Benchmark | # Tasks | Eval | Env | Fit | One-liner |
|---|---|---|---|---|---|
| [OfficeQA](office-work.md#officeqa) | 246 | DET (numeric tolerance) | Dataset | ★★★ | Reference card; Treasury Bulletin PDF QA. |
| [FinanceBench](office-work.md#financebench) | 150 (public) | JUDGE (rubric) | Dataset | ★★★ | SEC-filings open-book QA. |
| [MMLongBench-Doc](office-work.md#mmlongbench-doc) | 1,091 | DET (F1 + unanswerable) | Dataset (PDFs) | ★★★ | Long multimodal PDFs (avg 47.5 pages). |
| [M-LongDoc](office-work.md#m-longdoc) | 851 | JUDGE (rubric) | Dataset (200+ page docs) | ★★★ | Multimodal super-long doc explanatory QA. |
| [LegalBench](office-work.md#legalbench) | 162 tasks | DET (accuracy/F1/EM) | Dataset | ★★★ | Legal reasoning across contracts, statutes, opinions. |
| [Hybrid Financial Table QA](office-work.md#hybrid-financial-table-qa-tat-qa-finqa-multihiertt) | ~35k combined | DET (numeric + program exec) | Dataset | ★★★ | TAT-QA + FinQA + MultiHiertt cluster — hybrid table+text. |
| [LOFT](office-work.md#loft) | 35 datasets × 4 modalities | DET (EM/F1/exec) | Dataset | ★★★ | Long-context vs RAG — can LCLMs replace retrieval pipelines? |
| [CRAG](office-work.md#crag-comprehensive-rag-benchmark) | 4,409 | DET (hallucination-penalised) | Mock web + KG APIs | ★★★ | Meta's canonical RAG bench with hallucination accounting. |
| [SpreadsheetBench](office-work.md#spreadsheetbench) | 912 | DET (online-judge, file-diff) | Dataset | ★★★ | Real-world Excel forum questions; SOTA ~17%. |
| [SheetCopilot](office-work.md#sheetcopilot-benchmark-component) | 221 | DET (sheet state) | Live Excel (pywin32) | ★ | Atomic spreadsheet action API; Windows-only. |
| [TableBench](office-work.md#tablebench) | 886 | MIX (num + ROUGE + exec) | Dataset | ★★★ | Table QA with visualization subset (needs code exec). |
| [PPTC / PPTC-R](office-work.md#pptc-pptc-r) | 279 sessions | DET (PPTX-Match) | Python-pptx sandbox | ★★ | Multi-turn PowerPoint API control; 6% session-level accuracy. |
| [PPTAgent + Zenodo10K](office-work.md#pptagent-zenodo10k-ppteval) | Zenodo-derived | JUDGE (content+design+coherence) | Python-pptx sandbox | ★★ | Agentic PPT generation with rubric-based evaluation. |
| [OfficeBench](office-work.md#officebench) | ~300 | MIX (exact + fuzzy + exec) | Docker (Office suite) | ★★ | Cross-app: Word + Excel + email + calendar. |
| [OdysseyBench](office-work.md#odysseybench) | 602 | DET (exec state) | Docker (Office suite) | ★★ | Long-horizon multi-app with memory. |
| [TheAgentCompany](office-work.md#theagentcompany) | 175 | MIX (checkers + partials) | Docker (GitLab + Plane + RocketChat + ownCloud) | ★★ | Simulated software company; also listed in browser. |
| [EnterpriseBench](office-work.md#enterprisebench) | 550 | DET (state + ACL) | Enterprise sim sandbox | ★★ | Access-control-aware enterprise workflows across HR/finance/SWE. |
| [WorkArena / ++](office-work.md#workarena-workarena) | 33 × 19,912 / 682 | DET (REST state) | VM (ServiceNow PDI) + BrowserGym | ★ | Enterprise SaaS knowledge work; also listed in browser. |
| [CRMArena / Pro](office-work.md#crmarena-crmarena-pro) | 9 types / 19 | DET (Salesforce state) | VM (Salesforce dev org) | ★ | Professional CRM work; Salesforce populated with 16 objects. |
| [AssistantBench](office-work.md#assistantbench) | 214 | DET + partial | Live web | ★★ | Realistic personal-assistant research tasks; also listed in browser. |
| [AstaBench](office-work.md#astabench) | 2,400+ | MIX (per-bench + judge) | Asta tool stack | ★ | Research-assistant bench; cost-normalised leaderboard. |

Noted, not carded: GAIA (already supported), DocVQA/ChartQA (single-doc, no tools), HotpotQA/MuSiQue (not agentic), SUPER (research-code agent), Enron emails (no mature agentic variant), ColBench (win-rate vs preferences), PPTBench (VLM-only), SlidesBench/AutoPresent (subsumed by PPTAgent), OmniACT (UI-script-scored), ToolQA (not office-shaped), InfiniteBench/HELMET (synthetic long-context), DocBench (format-overlap), WritingBench/LongGenBench (not agentic), LawBench (Chinese LegalBench dup).

---

## Financial transactions — [financial.md](financial.md)

20 cards. Support/transactional agents, function-calling, document QA, trading, accounting, compliance.

| Benchmark | # Tasks | Eval | Env | Fit | One-liner |
|---|---|---|---|---|---|
| [τ-bench](financial.md#tau-bench-retail-airline) | ~114 + ~50 | DET (DB diff) | Sim (in-process) | ✓ | Already integrated; retail + airline customer support. |
| [τ²-bench / τ³](financial.md#tau2-bench-airline-retail-telecom-banking_knowledge-mock) | ~284+ | DET (DB diff) | Sim | ★★★ | Next upgrade: dual-control, adds telecom + banking_knowledge + voice mode. |
| [AppWorld](financial.md#appworld) | 750 | DET (SGC/TGC tests) | Sim / Docker HTTP | ★★★ | Best banking-adjacent sim: Venmo + Splitwise + Amazon + 6 more apps. |
| [BFCL V3/V4](financial.md#bfcl-berkeley-function-calling-leaderboard-v3v4) | ~4k+ | DET (AST / exec) | Sim (mock backends) | ★★★ | Function-calling baseline; TradingBot + MathAPI + TravelAPI backends. |
| [Finance Agent (Vals AI)](financial.md#finance-agent-benchmark-vals-ai) | 537 (50 public) | MIX (match + judge) | Local CLI + EDGAR/web | ★★ | Agentic FinanceBench counterpart; analyst workflow with tools. |
| [FinAgentBench](financial.md#finagentbench) | ~26k triples | DET (nDCG/MRR) | Dataset | ★★★ | Two-stage agentic retrieval over SEC filings (doc → chunk). |
| [FinanceBench](financial.md#financebench-patronus-ai) | 150 (public) | JUDGE (rubric) | Dataset | ★★★ | SEC-filings QA; RAG-wrappable (also listed in office). |
| [FinanceQA (AfterQuery)](financial.md#financeqa-afterquery) | 148 | MIX (numeric + essay judge) | Dataset | ★★★ | Hard analyst QA under incomplete information. |
| [DocFinQA](financial.md#docfinqa) | 7,437 | DET (numeric match) | Dataset (~123k tokens avg) | ★★★ | Long-context FinQA extension; full parent documents. |
| [MultiHiertt (+ TAT-QA)](financial.md#multihiertt-tat-qa-lineage) | 10,440 | DET (EM + program exec) | Dataset | ★★★ | Hybrid hierarchical table + text reasoning. |
| [BizBench](financial.md#bizbench) | ~several k | DET (numeric + code exec) | Dataset + Python exec | ★★★ | Code-act over 10-K tables; Kensho. |
| [CFA-Bench (Level III)](financial.md#cfa-bench-cfa-level-iii) | 11 MCQ + 11 essay blocks | MIX (det MCQ + essay judge) | Dataset | ★★★ | CFA Level III professional credentialing; MCQ + essay. |
| [FinTagging](financial.md#fintagging) | FinNI + FinCL | DET (F1 + top-k) | FinBen harness + ES | ★★ | XBRL taxonomy-grounded tagging; US-GAAP 2024. |
| [FinBen](financial.md#finben) | 42 datasets / 24 tasks | MIX | lm-evaluation-harness | ★★ | Broad umbrella — NLP + forecasting + trading + bilingual. |
| [FinEval (Chinese)](financial.md#fineval-chinese-financial-llm-benchmark) | 26k+ (616 agent) | DET (MCQ) + trajectory | Dataset + sim tools | ★★ | Chinese finance bench with dedicated agent track. |
| [InvestorBench](financial.md#investorbench) | per-asset episodes | DET (finance metrics) | Docker-compose + Qdrant | ★★ | Trading agent across equities, crypto, ETFs; long-horizon daily loop. |
| [StockBench](financial.md#stockbench) | 20 DJIA × 82 days | DET (finance metrics) | Offline sim | ★★ | Contamination-free post-2024 portfolio trading. |
| [LiveTradeBench](financial.md#livetradebench) | continuous | DET (finance metrics) | Live markets + Polymarket | ★ | Live markets; LMArena rank ≠ PnL rank. |
| [CryptoBench](financial.md#cryptobench) | 50/mo + 230 + 727 MCQ | JUDGE (0-3 rubric) | Live crypto dashboards | ★ | DeFi agent bench; monthly refresh vs contamination. |
| [AccountingBench (Penrose)](financial.md#accountingbench-penrose) | 12 months | DET (ledger reconciliation) | **Closed-source** | — | Reference result only; long-horizon bookkeeping. |

Excluded: ToolBench/ToolLLM (RapidAPI rot), FLUE (pure NLP), API-Bank (dominated by BFCL), TradingAgents/FinMem/StockAgent (frameworks, subsumed), FinQA/ConvFinQA (subsumed by DocFinQA), FiNER/FNXL (subsumed by FinTagging), FinMTEB (embeddings-only), smart-contract audit benches (small + better under security).

---

## Browser use — [browser.md](browser.md)

24 cards. Self-hosted Docker prioritised; live-web benches included with caveats.

| Benchmark | # Tasks | Obs modality | Eval | Env | Fit | One-liner |
|---|---|---|---|---|---|---|
| [WebArena](browser.md#webarena) | 812 | DOM / a11y | DET (URL/DB/answer) | Docker (6 sites) | ★★ | The canonical self-hosted web bench; AMI available. |
| [WebArena-Verified](browser.md#webarena-verified) | 812 (+ 258 hard) | DOM / a11y | DET (audited evaluators) | Docker or offline replay | ★★★ | Cleaned WebArena with trace-replay mode for CI. |
| [VisualWebArena](browser.md#visualwebarena) | 910 | + Screenshot (SoM) | DET | Docker (3 new sites) | ★★ | Multimodal WebArena: classifieds + shopping + reddit. |
| [MiniWoB++](browser.md#miniwob) | 100+ | DOM + screenshot | DET (reward) | Static HTML | ★★★ | Canonical synthetic web widgets; Gym API, cheap RL smoke test. |
| [WebGames](browser.md#webgames) | ~150 | any | DET (ground-truth token) | Self-hosted static | ★★★ | Hermetic computer-use challenges; cleanest reproducibility. |
| [WebShop](browser.md#webshop) | 12,087 | HTML / simple | DET (attribute overlap) | Self-hosted Flask | ★★★ | E-commerce single-site; 1.18M real Amazon products. |
| [WorkArena / ++](browser.md#workarena-workarena) | 33 × 19,912 / 682 | DOM / a11y / screenshot | DET (REST state) | VM (ServiceNow PDI) | ★ | Enterprise SaaS; first-class BrowserGym integration. |
| [TheAgentCompany](browser.md#theagentcompany) | 175 | Screenshot + DOM | MIX (state + judge partials) | Docker (GitLab + Plane + ownCloud + RocketChat) | ★★ | Simulated software company; browser + terminal scaffold. |
| [BrowserGym](browser.md#browsergym) | 9 suites wrapped | Unified bundle | delegated | Mixed | ★★ | Infrastructure: 1 integration → 9 benchmarks. |
| [Mind2Web](browser.md#mind2web) | ~2,350 | HTML + optional screenshot | DET (oracle trajectory) | Dataset | ★★★ | Offline trajectories across 137 sites; 3 generalization splits. |
| [Online-Mind2Web](browser.md#online-mind2web) | 300 | Screenshot | JUDGE (WebJudge) | Live web | ★★ | Live-web refresh of Mind2Web; LLM-as-judge required. |
| [WebCanvas / Mind2Web-Live](browser.md#webcanvas-mind2web-live) | 542 | DOM + screenshot | DET (key-node milestones) | Live web | ★★ | Milestone-scored live web; robust to site drift. |
| [Mind2Web-2](browser.md#mind2web-2) | 130 | Any browsing | MIX (rubric + judge + citation) | Live web | ★ | Long-horizon research synthesis; Agent-as-a-Judge. |
| [WebLINX](browser.md#weblinx) | ~2,337 demos | DOM + screenshot + video | DET (oracle turn) | Dataset + BrowserGym static | ★★★ | Conversational browsing; turn-level action prediction. |
| [WebVoyager](browser.md#webvoyager) | 643 | Screenshot (SoM) | JUDGE (GPT-4V) | Live web | ★★ | Reference live-web recipe across 15 big-name sites. |
| [WebBench](browser.md#webbench-halluminate) | 2,454 (5,750 total) | Any browsing | MIX (programmatic + judge) | Live web | ★★ | Production-flavoured 452 real sites; READ + WRITE split. |
| [WebWalkerQA](browser.md#webwalkerqa) | 680 | Rendered HTML | DET (answer + hop) | Live web (seeded URLs) | ★★ | Multi-page subtree traversal; deep-within-site QA. |
| [BrowseComp](browser.md#browsecomp) | 1,266 | Any browsing | DET (answer match + grader) | Live web | ★★ | OpenAI's deep-research fact-seeking; near-zero for GPT-4o. |
| [AssistantBench](browser.md#assistantbench) | 214 | Screenshot | DET (partial-credit) | Live web | ★★ | Personal-assistant research; also listed in office. |
| [BEARCUBS](browser.md#bearcubs) | 111 | Any (computer-use) | DET (answer match) | Live web | ★★ | Deliberately brittle live-web + multimodal info-seeking. |
| [MMInA](browser.md#mmina) | 1,050 | Screenshot + HTML | DET (answer + hop success) | Live web | ★★ | Multi-hop multimodal cross-site tasks. |
| [VisualWebBench](browser.md#visualwebbench) | 1,500 | Screenshot | DET (acc/F1/IoU) | Dataset | ★★★ | Offline web-understanding diagnostics; prerequisite check. |
| [OSWorld (browser slice)](browser.md#osworld-browser-slice) | ~40 Chrome | Screenshot + a11y | DET (state scripts) | VM (multi-provider) | ★ | Computer-use VM; browser is one modality of broader bench. |
| [AndroidWorld](browser.md#androidworld) | 116 × params | Screenshot + a11y | DET (adb state) | Android emulator | ★ | **Mobile, not browser** — cross-reference for Chrome slice overlap. |

Excluded: GAIA (already supported), VisualAgentBench (overlaps WebArena), SeeClick (grounding only), "Online-Shopping-Bench" / "SimpleBrowserBench" (no canonical repos), ShowUI/AgentTrek (data pipelines not benches), OmniACT (PyAutoGUI-centric), ScreenSpot/ScreenSpot-Pro (grounding only), InSTA (training data).

---

## Cross-cutting observations

- **Inspect AI as a reference implementation source**: Cybench, AgentDojo, AgentHarm, WMDP, 3CB, HarmBench, BrowseComp and several others already have Inspect implementations you can read for evaluator logic before porting into Rust adapters.
- **BrowserGym as a unification layer**: if we want to be pragmatic, one BrowserGym adapter gets us WebArena (+ Verified), VWA, WorkArena, MiniWoB++, AssistantBench, WebLINX-static, and WebCanvas at once. Trade-off is a Python sidecar.
- **Docker-per-instance SWE-Bench pattern**: SWE-Bench Pro, SWE-Gym, R2E-Gym, Multi-SWE-Bench, SWE-Lancer, SWE-Smith, SWE-PolyBench (and differently, Commit0, MLE-Bench, PaperBench) all follow the same per-task-image shape. The existing `swe_bench.toml` adapter should carry over with only config-level changes.
- **Prompt-injection benches with stubbed tools** (InjecAgent, AgentHarm, AgentDojo, BIPIA) don't need a real execution sandbox — canned tool responses make them the lightest-weight security adapters to ship first.
- **τ²-bench is the obvious next transactional integration** since we already have τ-bench; same shape, cleaner reward computation, new domains (telecom + banking_knowledge).
- **Contamination-aware benches** to prioritise over saturated classics: SWE-Bench Pro > Verified; LiveCodeBench (Pro) > HumanEval; StockBench > FinBen-trading; Finance Agent (Vals AI) > FinanceBench; WebArena-Verified > WebArena; CryptoBench (monthly refresh) > static crypto Q&A.
- **SWE-Bench-shape benches for security and IT**: SEC-bench (security PoC + patching) and AIOpsLab/ITBench/DevOps-Gym (SRE/AIOps/DevOps) extend the per-instance-Docker pattern into new domains — same adapter mechanics, different tool surface.
- **Attack-layer benches sit above capability benches**: DoomArena is designed to plug into existing BrowserGym/τ-Bench/OSWorld installations and layer an attack axis on top, so an existing capability suite can double as a security suite without a separate adapter.
- **Cross-listed benchmarks** (count once): TheAgentCompany is listed under both browser and office work; MLE-Bench under coding and terminal; AssistantBench under browser and office work; WorkArena under office work and browser. Pick one "home" per category when building the adapter.
