# Security Benchmarks

### Cybench
1. **description**: Framework for evaluating cybersecurity capabilities and risks of language models via professional CTF challenges. Authors position it as the first CTF benchmark "recent, meaningful, and spanning a wide range of difficulties," adopted by US AISI, UK AISI, and Anthropic system cards.
2. **github**: https://github.com/andyzorigin/cybench — fully open source (MIT). Also integrated in Inspect Evals.
3. **tools**: Prescribes a scaffolded agent. Runs inside a Kali Linux sandbox (Docker or Kubernetes) with shell + pentest tooling; `run_task.sh` is the primary entrypoint. Iteration/token caps configurable.
4. **data**: 40 tasks (39 in Inspect version post-GPL removal) sourced from HackTheBox cyber-apocalypse-2024, SekaiCTF 2022-23, Glacier, HKCert. Each task ships challenge files, Dockerfile, and a known flag. Subtasks provided for fractional scoring.
5. **tasks**: 40 CTF tasks across crypto, web, pwn, reversing, forensics, misc. Domain: offensive security / CTF.
6. **evaluation**: Deterministic flag-string match. Two metrics: unguided success (binary) and subtask score (fraction of subtasks solved). No LLM judge.
7. **playground**: Full harness — Docker/K8s sandbox per task, CLI driver, agent loop with configurable iteration budgets.
8. **category**: security / CTF (offensive). Broad coverage across crypto, web, pwn, forensics makes it the de facto open-source CTF benchmark for frontier-model evaluation.

### NYU CTF Bench
1. **description**: Scalable open-source CTF benchmark for evaluating LLMs in offensive security, built from CSAW CTF archives. Authors emphasize scale and standardized docker packaging over Cybench's curated difficulty.
2. **github**: https://github.com/NYU-LLM-CTF/NYU_CTF_Bench (dataset) + https://github.com/NYU-LLM-CTF/llmctfautomation (agent framework). NeurIPS 2024 D&B. Fully open source.
3. **tools**: Ships a reference agent framework (llmctfautomation) with shell / file tools; tasks can be consumed agent-as-is since challenges are plain Dockerized services. Agent scaffolding is optional.
4. **data**: 200-challenge main set + 55-challenge dev split. Each challenge dockerized with writeup, flag, and any required binaries/services.
5. **tasks**: 200 challenges across 6 categories — web, pwn, forensics, rev, crypto, misc. Domain: CTF / offensive security.
6. **evaluation**: Deterministic flag match against ground-truth string per challenge.
7. **playground**: Full harness — docker-compose per challenge, automation framework with agent loop.
8. **category**: security / CTF. Complements Cybench with larger scale and explicit train/test split useful for fine-tuning experiments.

### CyberSecEval 3 (PurpleLlama)
1. **description**: Meta's cyber-risk benchmark suite covering both defensive (insecure-code generation, prompt injection compliance) and offensive capabilities (automated spear phishing, autonomous cyber operations). Authors frame it as "eight risks across two categories: third-party risk and developer/user risk," aligned to CWE and MITRE ATT&CK.
2. **github**: https://github.com/meta-llama/PurpleLlama/tree/main/CybersecurityBenchmarks — MIT, fully open. CyberSecEval 4 is the current main branch; v3 tests still included.
3. **tools**: Mostly agent-as-is for static tests (code-gen, prompt injection, MITRE TTP generation). Autonomous-cyber-ops track uses a multi-turn shell-executor harness bundled in the repo.
4. **data**: ICD-based prompts for insecure code (Python/JS/C++), visual prompt-injection image set, spear-phishing persona corpus, FRR/MITRE TTP prompt sets. All JSON/JSONL.
5. **tasks**: ~thousands of prompts total across suites (insecure-code ~1k, MITRE ~1k, prompt-injection ~250, spear phishing hundreds). Domain: cyber risk surface area.
6. **evaluation**: Mixed. Insecure code = static analysis (CodeShield / weggli / regex CWE detectors) — deterministic. Spear-phishing + autonomous ops = LLM-as-judge with rubric prompts. Prompt injection = string-match refusal + judge.
7. **playground**: Runner scripts (`run.py`) per suite; autonomous-ops ships a shell sandbox. Not a unified harness.
8. **category**: security / multi (code-security + prompt-injection + social-engineering + autonomous-cyber-ops). Breadth is the value; depth per suite is modest.

### InterCode-CTF
1. **description**: The CTF slice of InterCode (Princeton NLP, NeurIPS 2023), framing interactive coding as an RL environment with code-as-actions and execution-as-observation. Authors: "standardizing and benchmarking interactive coding with execution feedback."
2. **github**: https://github.com/princeton-nlp/intercode — MIT, fully open. Palisade Research maintains an active fork at https://github.com/palisaderesearch/intercode used in "Hacking CTFs with Plain Agents."
3. **tools**: Prescribes a bash-shell tool over a dockerized Ubuntu. Clean ReAct-style loop; adapter is minimal.
4. **data**: 100 picoCTF tasks; each task has query, gold solution, digital assets, tags. `ic_ctf.json` is the manifest.
5. **tasks**: 100 beginner-to-intermediate CTF tasks (mostly rev/crypto/forensics/web). Domain: CTF.
6. **evaluation**: Deterministic flag match. Gold flag per task; exact-string comparison on the agent's final answer.
7. **playground**: Full harness — per-task Docker container, gym-like `step()` interface, reward on correct flag.
8. **category**: security / CTF (beginner). Good smoke-test tier below Cybench; the gym-style API makes adapter integration trivial.

### AgentDojo
1. **description**: Dynamic environment evaluating prompt-injection attacks and defenses against tool-using LLM agents. Authors (ETH SPY lab + Invariant Labs): "not a static test suite, but an extensible environment for designing and evaluating new agent tasks, defenses, and adaptive attacks."
2. **github**: https://github.com/ethz-spylab/agentdojo — AGPL, fully open; `pip install agentdojo`. NeurIPS 2024 D&B. Integrated in Inspect Evals.
3. **tools**: Prescribes a rich simulated tool environment — email, cloud drive, banking, travel, Slack, workspace — all mocked in Python. Agents must use provided tool schemas.
4. **data**: 4 suites (workspace, slack, travel, banking) with simulated backing state; injection payloads embedded in tool outputs (emails, docs, messages). Data is code, not a static JSONL.
5. **tasks**: 97 realistic user tasks × 629 injection test cases. Domain: indirect prompt injection in tool-augmented agents.
6. **evaluation**: Deterministic state-based. Each task defines a `utility` check (did the legit task succeed?) and a `security` check (did the injection's target state change occur?). No LLM judge.
7. **playground**: Full harness — Python runtime simulating tool backends, benchmark CLI (`agentdojo.scripts.benchmark`), attack/defense plug-in interfaces.
8. **category**: security / indirect prompt injection (tool-integrated agents). Best-in-class for evaluating real tool-agent injection robustness with deterministic scoring.

### InjecAgent
1. **description**: Benchmark for indirect prompt injection in tool-integrated LLM agents. Authors (UIUC, Daniel Kang): "malicious instructions embedded within content processed by LLMs, aiming to manipulate these agents into executing detrimental actions against users." Focus on direct-harm and data-stealing attack classes.
2. **github**: https://github.com/uiuc-kang-lab/InjecAgent — MIT, fully open. ACL 2024 Findings.
3. **tools**: Agent-as-is using a ReAct-style prompt with provided tool specs. Tools are simulated via pre-written fake responses, not executed — so no real environment needed.
4. **data**: 1,054 test cases across 17 user tools + 62 attacker tools. `attacker_cases_dh.jsonl` (direct harm), `attacker_cases_ds.jsonl` (data stealing), `user_cases.jsonl`. Two settings: base and enhanced (with hacking prompt).
5. **tasks**: 1,054 single-turn agent prompts with injected tool responses. Domain: indirect prompt injection.
6. **evaluation**: Deterministic keyword/regex check on the agent's next action — does it call the attacker's requested tool with attacker's args? Reported as ASR-valid and ASR-all. No LLM judge.
7. **playground**: Dataset + evaluation scripts. No live tool runtime — the benchmark substitutes canned tool responses. Adapter is simple.
8. **category**: security / indirect prompt injection. Lighter-weight and simpler to integrate than AgentDojo; complementary since it stresses many tool schemas rather than deep state.

### AgentHarm
1. **description**: Benchmark from UK AI Safety Institute measuring whether agents refuse overtly malicious tasks and, if jailbroken, whether they retain the capability to complete them. Authors: "scoring well requires jailbroken agents to maintain their capabilities following an attack to complete a multi-step task." 11 harm categories including cybercrime, fraud, harassment.
2. **github**: Integrated in Inspect Evals (https://github.com/UKGovernmentBEIS/inspect_evals) with dataset on HuggingFace `ai-safety-institute/AgentHarm`. ICLR 2025. Fully open.
3. **tools**: Prescribes synthetic tool set (web browsing, email, social, code exec, etc.) wired through Inspect AI. Tool calls are stubbed with canned responses; no live execution.
4. **data**: 176 public augmented harmful tasks (44 base behaviors × augmentations) + 32 validation; plus benign-twin tasks to measure over-refusal.
5. **tasks**: Multi-step harmful agent requests across 11 categories. Domain: harmful tool use / jailbreak robustness.
6. **evaluation**: LLM-as-judge with structured rubric. Two judges: a refusal judge (did it refuse?) and a semantic judge (did it complete the harm?), both default to GPT-4o. Rubric is task-specific and ships with the dataset.
7. **playground**: Runs via Inspect AI harness. CLI: `inspect eval inspect_evals/agentharm`.
8. **category**: security / harmful-tool-use + jailbreak. Distinct from prompt-injection benchmarks: the user is the adversary, not the environment.

### RedCode
1. **description**: Safety benchmark for code-executing agents. Authors (AI-secure, NeurIPS 2024): "multi-dimensional safety benchmark" covering both recognizing/handling unsafe code (Exec) and resisting instructions to generate harmful code (Gen). Observes that ReAct agents are middle-safety, CodeAct agents are least safe.
2. **github**: https://github.com/AI-secure/RedCode — MIT, fully open.
3. **tools**: Prescribes a Docker code-execution sandbox. Includes adapters for three agent types: CodeAct, OpenCodeInterpreter, ReAct. Each has its own run script.
4. **data**: RedCode-Exec: 4,050 prompts (risky code in Python, Bash, natural language) across 25 OWASP-aligned risk scenarios. RedCode-Gen: 160 function-signature prompts targeting malware families.
5. **tasks**: 4,210 total. Domain: risky code execution + harmful code generation.
6. **evaluation**: Hybrid. Exec: deterministic rule checks + runtime indicators (did the agent execute the risky op in the sandbox?). Gen: LLM judge scoring malware similarity against targets, plus static detectors.
7. **playground**: Full Dockerized harness per agent type. Heavier to integrate than dataset-only benchmarks.
8. **category**: security / tool-misuse (code execution). Complements CTF benchmarks by testing whether capable code-agents refuse *dangerous* code, not whether they can solve hard puzzles.

### AutoPenBench
1. **description**: Benchmark for generative agents on penetration-testing tasks. Authors present fully-autonomous and assisted modes; reported 21% autonomous vs 64% assisted success rate in the paper.
2. **github**: https://github.com/lucagioacchini/auto-pen-bench — fully open source.
3. **tools**: Prescribes a Kali-Linux agent workstation container with pentest tooling + a `FinalAnswer` tool for flag submission. Example wiring uses the `instructor` library for structured JSON output; other agents must conform to the tool schema.
4. **data**: 36 tasks total — 24 in-vitro (access control, web, network, crypto) + 12 real-world CVE-based. Each ships Dockerfile + docker-compose + solution walkthrough + command/stage milestones.
5. **tasks**: 36 CTF-style pentest scenarios with milestones for partial credit. Domain: penetration testing.
6. **evaluation**: Deterministic flag match via `FinalAnswer` tool comparing against ground-truth flag. Stage milestones enable fractional/process scoring.
7. **playground**: Full harness — attacker Kali container + victim container(s) networked via docker-compose. CLI runner.
8. **category**: security / pentest. Smaller than Cybench but milestone-based scoring gives finer-grained signal than pure flag-match.

### CyberGym
1. **description**: UC Berkeley RDI benchmark for AI agents on real-world vulnerability analysis at scale. Authors: "rigorously assess the capabilities of AI agents on real-world vulnerability analysis tasks." Agents reproduce CVEs in large codebases; the benchmark has uncovered 10 zero-days during development.
2. **github**: https://github.com/sunblaze-ucb/cybergym — Apache 2.0, fully open. arXiv 2506.02548 (June 2025).
3. **tools**: Prescribes a container-per-task harness with shell + file tools + build system. Agents need to navigate large codebases, so a file/search toolkit is provided.
4. **data**: 1,507 instances drawn from OSS-Fuzz across 188 widely-used projects. Each instance ships the unpatched codebase, the vulnerability description, and a fuzzer harness to validate the PoC.
5. **tasks**: 1,507 real-CVE reproduction tasks. Three difficulty levels (Level 1 = description + code → PoC; harder levels strip hints). Domain: vulnerability reproduction / exploitation.
6. **evaluation**: Deterministic. Agent's generated PoC is fed through the project's fuzz harness; task scores 1 if it triggers the target crash/sanitizer signature, else 0. No judge.
7. **playground**: Full harness — per-task Docker with build toolchain + fuzzer runner + leaderboard infrastructure.
8. **category**: security / vulnerability-discovery (defensive-offensive overlap). Far more realistic codebase scale than CTF benchmarks; heavy but the most faithful proxy for real security-engineering capability.

### WMDP
1. **description**: Weapons of Mass Destruction Proxy benchmark — a malicious-knowledge probe across bio, cyber, and chem domains. Authors (Center for AI Safety): "proxy information which correlates with, is neighboring to, or is a component of actual hazardous knowledge," deliberately filtered to avoid publishing uplift material.
2. **github**: https://github.com/centerforaisafety/wmdp — MIT, fully open. Dataset on HuggingFace `cais/wmdp`. Integrated in lm-evaluation-harness.
3. **tools**: Agent-as-is. Pure multiple-choice; no tools, no environment.
4. **data**: 3,668 MCQs: 1,273 bio, 1,987 cyber, 408 chem. Each question has 4 choices with one correct answer.
5. **tasks**: 3,668 4-way multiple-choice items. Domain: hazardous-knowledge probing. Cyber split covers pen-testing, malware, exploit dev knowledge questions.
6. **evaluation**: Deterministic accuracy on MCQ. Zero-shot or few-shot via lm-eval-harness.
7. **playground**: None — pure dataset. Works via lm-evaluation-harness out of the box.
8. **category**: security / hazardous-knowledge (non-agent). Included because cyber split is a standard reference for any security-capability claim; adapter is trivial (MCQ scoring) but it doesn't exercise an action space — best used as a companion metric rather than a primary agent eval.

### BIPIA
1. **description**: Microsoft's benchmark for indirect prompt injection attacks. Authors: "first benchmark for indirect prompt injection attacks, focusing on scenarios where external content processed by the LLM contains malicious instructions." Broad attack-pattern sweep rather than AgentDojo-style deep tool simulation.
2. **github**: https://github.com/microsoft/BIPIA — MIT. Dataset mirrored on HuggingFace. KDD 2025.
3. **tools**: Agent-as-is. Tool outputs are simulated via pre-crafted contexts; no live tool runtime. Demo notebook shows how to plug a chat model into the evaluator.
4. **data**: Attack-pattern corpus across 5 domains (Email QA, Web QA, Table QA, Summarization, Code QA) × 5 text-only attack categories (task automation, info dissemination, phishing, ad promotion, fraud) + 3 code-attack categories. ~86k test prompts, ~626k train prompts.
5. **tasks**: Single-turn QA prompts with injected adversarial instructions embedded in the reference context. Domain: indirect prompt injection via document content.
6. **evaluation**: Deterministic. Per-attack regex/keyword detectors check whether the model's output complied with the injected instruction (ASR). Two defense evals also defined (boundary awareness, explicit reminder).
7. **playground**: Dataset + evaluator scripts. No live tool environment — adapter only needs a chat completion hook.
8. **category**: security / indirect prompt injection (content-based). Complements AgentDojo: BIPIA is breadth-first across attack patterns and document types; AgentDojo is depth-first in tool-integrated state.

### AIRTBench
1. **description**: Dreadnode's benchmark for autonomous AI red teaming — LLM agents attacking *other* AI/ML systems. Authors: "measure autonomous AI red teaming capabilities in language models" via black-box CTF challenges spanning prompt injection, model inversion, adversarial inputs, and system exploitation of AI services.
2. **github**: https://github.com/dreadnode/AIRTBench-Code — Apache 2.0, fully open. arXiv 2506.14682 (June 2025). Challenges hosted on Dreadnode's Crucible platform.
3. **tools**: Prescribes a Python-code execution sandbox with HTTP/network access so the agent can interact with the target AI service's API. Reference harness in the repo.
4. **data**: 70 black-box CTF challenges from the Crucible environment. Each exposes an AI service endpoint plus a task description; flag is retrieved by a successful attack.
5. **tasks**: 70 AI/ML attack scenarios — adversarial inputs, prompt injection, model inversion, system exploitation. Domain: attacking AI systems (AI-red-team CTF).
6. **evaluation**: Deterministic flag match. Agents submit a flag string to the challenge endpoint; correct flag = solve. Leaderboard tracks per-category success.
7. **playground**: Full harness — agent loop + Crucible challenge APIs. Can run against remote Crucible or reproduce locally.
8. **category**: security / AI-red-team (offensive, AI-on-AI). Unique niche — no other open benchmark systematically measures an agent's ability to break *other* ML systems.

### HarmBench
1. **description**: Center for AI Safety's standardized framework for automated red teaming and robust refusal. Authors: "lacking a standardized evaluation framework to rigorously assess [red-teaming] methods" — HarmBench fixes that with a fixed behavior set and a pipeline for attack × defense × judge.
2. **github**: https://github.com/centerforaisafety/HarmBench — MIT. Website https://www.harmbench.org. ICML 2024.
3. **tools**: Agent-as-is for most behaviors; multimodal and contextual subsets ship templates. Ships a classifier-based judge (HarmBench Llama-2-13B classifier) to replace GPT-4.
4. **data**: 400 harmful behaviors across 7 semantic categories + 4 functional categories (standard, copyright, contextual, multimodal). Attack artifacts from 18+ red-team methods (GCG, PAIR, TAP, AutoDAN, etc.) included.
5. **tasks**: 400 behaviors × attack method. Domain: automated jailbreak generation + refusal robustness. Single-turn.
6. **evaluation**: Classifier judge scores each (behavior, completion) pair as harmful/not. Attack Success Rate reported per method × model. Companion eval StrongREJECT (https://github.com/dsbowen/strong_reject) is commonly paired for harmfulness rubric scoring.
7. **playground**: Full pipeline: test-case gen → completions → eval. Config-driven; supports vLLM, HF transformers, closed-API models.
8. **category**: security / jailbreak + refusal. The de facto standard for comparing red-team attacks head-to-head; use StrongREJECT as a complementary scorer when comparing model-level refusal quality.

### DoomArena
1. **description**: ServiceNow Research framework for testing AI agents against evolving security threats. Authors frame it as a plug-in layer rather than a standalone benchmark: decouples attacks from environments so the same threat model can be applied across BrowserGym, τ-Bench, OSWorld, TapeAgents.
2. **github**: https://github.com/ServiceNow/DoomArena — Apache 2.0. Website https://servicenow.github.io/DoomArena. arXiv 2504.14064 (April 2025), ICML 2025.
3. **tools**: Plug-in adapters for existing agent environments — instruments tool outputs, browser DOM, and observations with configurable attack injectors. Agent itself uses the host environment's toolset.
4. **data**: Attack configurations (threat-model JSON) + host environment's task set. Ships reference configs for τ-Bench-airline/retail + BrowserGym-WebArena + OSWorld. No single static dataset.
5. **tasks**: Variable — inherits from host environment. Threat models cover indirect prompt injection, malicious pop-ups/banners, compromised tool outputs, data-exfiltration attempts.
6. **evaluation**: Environment-native utility score + attack-success flag. Attack success defined per threat model (did the attacker's target state change occur?). Deterministic.
7. **playground**: Framework + examples. Designed for extension — users declare a threat model and DoomArena injects attacks into whichever environment they choose.
8. **category**: security / agent-red-team framework. Valuable because it lets an agent harness reuse existing capability benchmarks while adding a security axis; closest analog to how nearai-bench might layer injections on top of a non-security suite.

### OpenAgentSafety
1. **description**: Comprehensive framework for evaluating real-world agent safety in realistic multi-turn environments. Authors (CMU/Stanford, ICLR 2026): built on top of TheAgentCompany with real tools (shell, browser, files, messaging). Finds unsafe actions in 49-73% of vulnerability-probing tasks.
2. **github**: https://github.com/sani903/OpenAgentSafety (primary) and https://github.com/kenhuangus/OpenAgentSafety (mirror). arXiv 2507.06134. Fully open.
3. **tools**: Real containerized sandbox inherited from OpenHands — Unix shell, Python interpreter, file system, real web browser, messaging clients. Not simulated.
4. **data**: 350+ multi-turn tasks with both benign and adversarial user intents, spanning 8 risk categories (privacy, legal, security policy, financial, etc.). Each task ships an environment seed + evaluation rubric.
5. **tasks**: 350+ multi-turn scenarios where the user or environment may try to coerce the agent into unsafe actions. Domain: realistic agent safety with extended interaction.
6. **evaluation**: Rule-based deterministic checks on final state (file contents, API calls made, messages sent) plus LLM-as-judge for ambiguous behavioral checks. Per-category safety score.
7. **playground**: Full harness built on OpenHands — Dockerized, runs the target agent end-to-end against scripted user simulators.
8. **category**: security / realistic-agent-safety (multi-turn, real tools). Distinct from AgentHarm (stubbed tools, single-turn harmful request) — OpenAgentSafety stresses compounding unsafe steps over extended sessions with real side effects.

### AgentPoison
1. **description**: NeurIPS 2024 red-team benchmark for poisoning the memory / RAG knowledge base of LLM agents. Authors: "first backdoor attack targeting generic and RAG-based LLM agents" via optimized triggers that require no model retraining. Complementary to injection-based attacks — targets the retrieval layer.
2. **github**: https://github.com/BillChan226/AgentPoison (primary) and https://github.com/AI-secure/AgentPoison. MIT. NeurIPS 2024.
3. **tools**: Targets three concrete agent stacks with their native tools: an RAG autonomous-driving agent, a knowledge-intensive QA agent (ReAct + retrieval), and EHRAgent (healthcare tools). Attack framework is trigger-optimization code; evaluation uses the host agents.
4. **data**: Poison sets generated by the iterative trigger optimizer; three target agents ship with their native datasets (nuScenes, HotpotQA variant, MIMIC-III-derived). Poisoning rates < 0.1%.
5. **tasks**: Hundreds of per-agent queries — some clean (measuring benign-utility degradation), some trigger-bearing (measuring ASR). Domain: agent memory / RAG poisoning.
6. **evaluation**: Deterministic. ASR = trigger queries that routed to poisoned memory and executed the attacker goal. Benign-utility delta bounded < 1%. Per-agent task-specific correctness metrics.
7. **playground**: Three end-to-end agent harnesses + attack generator. Heavier to integrate than single-turn benchmarks but the only open benchmark for this attack vector.
8. **category**: security / supply-chain-adjacent (memory + RAG poisoning). Unique niche; belongs in any comprehensive agent-security battery alongside AgentDojo (prompt-level) and HarmBench (jailbreak-level).

### 3CB (Catastrophic Cyber Capabilities Benchmark)
1. **description**: Apart Research benchmark for robustly evaluating LLM agent *cyber offense* capabilities. Authors: addresses "legibility, coverage, and generalization" failings of CTF-only benchmarks — challenges mapped to all MITRE ATT&CK categories, not just crypto/pwn/web.
2. **github**: https://github.com/apartresearch/3cb — open source. Integrated in Inspect Evals as `threecb`. arXiv 2410.09114.
3. **tools**: Prescribes a bash-shell sandbox harness (3CB Harness) with reproducible configs. Agent executes shell commands in a Docker environment; flag returned via stdout.
4. **data**: 15 original challenges designed to span the MITRE ATT&CK enterprise matrix (reconnaissance, initial access, execution, persistence, privilege escalation, defense evasion, credential access, discovery, lateral movement, collection, exfiltration, C2, impact).
5. **tasks**: 15 offensive-cyber scenarios. Smaller than Cybench/NYU CTF but deliberately curated for ATT&CK coverage — each challenge exercises a different technique category. Domain: offensive cyber capability.
6. **evaluation**: Deterministic flag match; the 3CB Harness logs full trajectories for legibility audits.
7. **playground**: Full harness + challenge set. Runs via Inspect AI (`inspect eval inspect_evals/threecb`) or directly.
8. **category**: security / offensive-cyber (capability-mapped). Useful as a structured complement to Cybench when you care about *which* ATT&CK category the agent handles, not just raw flag count.

### SEC-bench
1. **description**: First fully automated benchmarking framework for LLM agents on real-world software security engineering tasks. Authors: "significant performance gaps" — SOTA code agents reach 18% on PoC generation and 34% on vuln patching. Complements CyberGym by including the patching side in addition to reproduction.
2. **github**: https://github.com/sec-bench/sec-bench (referenced from https://sec-bench.github.io/). arXiv 2506.11791 (June 2025).
3. **tools**: Prescribes a container-per-instance sandbox with full build toolchain, a fuzzer harness, and file/search tools for navigating repos. Includes a multi-agent scaffold for dataset construction — instance creation is itself automated at ~$0.87/instance.
4. **data**: Real-world software vulnerabilities from OSS projects, each with a reproducible environment, vulnerability metadata, and a gold patch. Dataset grows over time via the automated pipeline.
5. **tasks**: Two task types per instance — (a) PoC generation (given vulnerability description + code, produce a triggering input) and (b) vulnerability patching (produce a fix that passes tests + neutralizes the PoC). Domain: defensive + offensive security engineering.
6. **evaluation**: Deterministic. PoC evaluated by whether it triggers the target sanitizer/crash in the sandbox; patch evaluated by test-suite pass + PoC neutralization. No LLM judge.
7. **playground**: Full automated harness — instance builder, sandbox runner, leaderboard.
8. **category**: security / software-security-engineering (defensive + offensive). Closest analog to SWE-bench but security-flavored; heavier than CTF benches but far more faithful to real security-engineering work.
