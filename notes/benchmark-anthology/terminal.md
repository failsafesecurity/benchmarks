# Terminal Benchmarks

### Terminal-Bench
1. **description**: Stanford x Laude Institute benchmark "to help agent makers quantify their agents' terminal mastery." Tasks span software engineering, ML, security, and data science — the kind of multi-step sysadmin/hacking/wrangling work a developer does in a shell. v2.0 ships "Terminal-Bench-Core" (89 hand-curated tasks) as the leaderboard target; v1.0 had 80.
2. **github**: https://github.com/laude-institute/terminal-bench — Apache-2.0, fully open source, very active (the `harbor-framework/terminal-bench` mirror is the same project).
3. **tools**: Prescribes tool scaffolding. The reference agent ("Terminus") has exactly one tool: send keystrokes into a tmux session. Other built-in adapters (Claude Code, Codex, etc.) keep their native tool sets. New frameworks plug in via an adapter in `/adapters`.
4. **data**: Per-task directory with `task.yaml`, `Dockerfile`, `tests/` (pytest scripts), `solution.sh` (oracle). Multi-container setups orchestrated via `DockerComposeManager`.
5. **tasks**: 89 tasks in Core v0.1.1 (v2.0), ~100+ in full beta. Mix of "build a kernel module", "fix a broken Python env", "exfiltrate a flag", "parse log files". Difficulty skews hard — SOTA is <60% solved.
6. **evaluation**: Deterministic. Each task has a pytest-based test script that runs inside the container after the agent yields and checks final filesystem / process / output state. Optional asciinema recording for replay.
7. **playground**: Per-task Docker container (usually Ubuntu-based, task-specific image) with one or more `TmuxSession` instances attached. Agents drive commands through tmux; harness captures panes.
8. **category**: terminal — general (sysadmin + devops + data wrangling + security, hand-curated).

### InterCode-Bash
1. **description**: Princeton NLP's framework for "standardizing and benchmarking interactive coding with execution feedback." The Bash environment measures whether an agent can drive a shell to accomplish file-manipulation goals over multiple turns, observing stdout/stderr between commands.
2. **github**: https://github.com/princeton-nlp/intercode — MIT, open source. Paper: NeurIPS 2023.
3. **tools**: Prescribes a minimal tool surface: one action = a bash command string passed to `BashEnv.step()`. Observation is the command's stdout/stderr; a `submit` action ends the episode. Agents can be ReAct, Plan-and-Solve, etc. — InterCode only defines the environment, not the policy.
4. **data**: A Docker image per environment (`intercode-bash` based on Ubuntu) with seeded filesystem state (files, directories, permissions) per task. Test cases derived from NL2Bash with a manually curated test set.
5. **tasks**: 200 dev + ~24 test Bash instruction/command pairs (from NL2Bash), plus the CTF variant (100 PicoCTF tasks) and SQL/Python/SWE environments. Each task has an NL instruction ("find all files modified in the last 24 hours larger than 1MB") and an oracle command.
6. **evaluation**: Deterministic dual-execution: run the agent's final command and the gold command in identical fresh containers, then compare resulting filesystem state (file hashes, directory listings) and/or stdout. Reward is 1.0 on match.
7. **playground**: Docker container per task, reset between episodes. No tmux — raw `docker exec` of each command.
8. **category**: terminal — shell scripting / file manipulation (short single-turn to few-turn commands).

### Cybench
1. **description**: Stanford/Cornell/Berkeley benchmark of "professional-level Capture the Flag tasks" — the agent sits in a Kali Linux shell and has to solve real CTF challenges (crypto, pwn, web, rev, forensics) end-to-end. Measures offensive-security capability of frontier models.
2. **github**: https://github.com/andyzorigin/cybench — Apache-2.0, fully open source.
3. **tools**: Agent-as-is in unguided mode (just bash); subtask-guided mode feeds intermediary Q&A prompts as scaffolding. Baseline agent uses HELM to unify LLM providers and a simple bash-exec loop.
4. **data**: 40 tasks from 4 public CTF competitions (HackTheBox, SekaiCTF, Glacier, HKCert). Each task ships starter files + Docker images for any network services. 17 tasks also have human-authored subtasks.
5. **tasks**: 40 tasks, plus 56 subtasks across a subset. Difficulty calibrated by human solve time (11 minutes to 24+ hours). Flag-based final answers.
6. **evaluation**: Deterministic flag comparison. Evaluator checks the agent's submitted answer against the flag. Subtask mode scores intermediate answers sequentially.
7. **playground**: Kali Linux Docker container (the agent's execution env) + one or more challenge service containers reachable over a docker network. `docker exec` for command execution.
8. **category**: terminal — offensive security / CTF.

### NYU CTF Bench
1. **description**: "Scalable open-source benchmark for evaluating LLMs in offensive security." 200 CSAW CTF challenges dockerized for automated agent interaction. Covers web, pwn, rev, crypto, forensics, misc. Designed explicitly so an agent framework can spin up a challenge, interact, and submit a flag with no human in the loop.
2. **github**: https://github.com/NYU-LLM-CTF/NYU_CTF_Bench (benchmark) + https://github.com/NYU-LLM-CTF/nyuctf_agents (baseline agents) — MIT licensed, fully open.
3. **tools**: Baseline agents (D-CIPHER) prescribe function-calling tools (run_command, decompile, disassemble, submit_flag). Benchmark itself is tool-agnostic — you bring your own scaffolding.
4. **data**: 200 challenges with pre-built Docker Hub images and per-challenge `docker-compose.yaml`. Challenge files staged into a working directory that the agent mounts.
5. **tasks**: 200 tasks across 6 categories. Difficulty ranges from intro-level CSAW quals to finals-tier. Significantly harder on average than Cybench — published SOTA <10% on full set.
6. **evaluation**: Deterministic flag check against metadata. Each challenge has a known flag string; submission matching the flag = pass.
7. **playground**: Per-challenge Docker compose stack (challenge server containers) + a separate agent container with standard CTF tooling (gdb, pwntools, radare2, etc.).
8. **category**: terminal — offensive security / CTF (larger + more systematic than Cybench).

### AgentBench-OS
1. **description**: The OS subset of THUDM's AgentBench, "testing LLM agents in real interactive environments." OS tasks put the agent in a fresh Ubuntu container and ask it to perform sysadmin work (find files matching criteria, count users, manipulate permissions, check config). One of the earliest dockerized shell-agent benchmarks.
2. **github**: https://github.com/THUDM/AgentBench — Apache-2.0, open source.
3. **tools**: Prescribes a function-calling schema: the agent outputs `Act: bash ...` or `Act: finish ...` blocks. Framework parses these, runs commands via `docker exec`, feeds back stdout.
4. **data**: Three Docker images: `local-os/default`, `local-os/packages` (more utilities pre-installed), `local-os/ubuntu`. Per-task initialization scripts seed filesystem state.
5. **tasks**: ~144 OS tasks in the main release (dev+test). Tasks are short (1-5 turns typically). Example: "How many users are in the system whose login shell is not /bin/bash?"
6. **evaluation**: Deterministic. Each task has either an expected answer string (match) or a checker script that runs in the container post-hoc to verify state.
7. **playground**: Ubuntu Docker container per task, `docker exec` for commands, ~5s startup, <500MB RAM per worker. No tmux.
8. **category**: terminal — sysadmin Q&A / file and process inspection.

### NL2Bash-EABench (IBM)
1. **description**: Execution-based re-evaluation framework from IBM Research that fixes NL2Bash's original string-match scoring. Generates bash (and PowerShell) from NL prompts and actually runs them in a container, then verifies behavior against an oracle. Focuses on incident-remediation style commands.
2. **github**: https://github.com/IBM/nl2bash-eabench — open source (Apache-2.0 per IBM convention).
3. **tools**: Agent-as-is for generation; the harness itself does the execution. Not a multi-turn environment — agent produces a script, harness runs it once and checks.
4. **data**: 150 hand-crafted test cases across three benchmarks (bash_1, bash_2, bash_3, 50 each). Each case is a JSON record with NL prompt, expected behavior, and container setup. Podman and Docker build scripts included.
5. **tasks**: 150 bash tests + additional PowerShell tests. Incident-remediation focus (restart a service, rotate a log, clean a directory, diagnose a failure). Difficulty skews easy-to-medium.
6. **evaluation**: Deterministic execution-based. Run the generated script inside the container, compare return code + stdout/stderr + filesystem mutations to the expected-behavior verifier. Per-test JSON output.
7. **playground**: Podman or Docker container per test with a task-specific seeded FS. One-shot execution (no interactive shell).
8. **category**: terminal — incident remediation / shell scripting (one-shot).

### SWE-agent Bash Tasks (via SWE-ReX)
1. **description**: Not a benchmark per se but worth including: SWE-agent's execution backend `SWE-ReX` exposes a clean bash-in-Docker sandbox with stateful interactive shells (ipython, gdb). Any NL2Bash-style or sysadmin task set can be dropped in. Relevant because it's the most widely-adopted Docker bash execution shim in the agent-framework ecosystem.
2. **github**: https://github.com/SWE-agent/SWE-ReX (runtime) + https://github.com/SWE-agent/mini-swe-agent (reference agent) — MIT, open source.
3. **tools**: Prescribes one tool: `execute_bash` (plus optional `str_replace_editor`, `submit`). SWE-ReX handles prompt/return-code detection so the agent sees clean command outputs, including for interactive tools.
4. **data**: None shipped — bring-your-own task set. Common pairing is SWE-bench tasks, but the shell backend is generic.
5. **tasks**: Depends on the mounted suite. Mini-SWE-agent achieves >74% on SWE-bench Verified using only `execute_bash`, showing how much mileage pure-shell agents have.
6. **evaluation**: Delegated to the mounted suite. SWE-ReX itself only guarantees reliable command execution + exit-code capture.
7. **playground**: Docker environment by default (also supports Singularity, Modal cloud, local subprocess). Runs a small agent-server inside the container; command dispatch via HTTP.
8. **category**: terminal — execution runtime / harness building block (use as a substrate for adapting other shell benchmarks).

### InterCode-SQL
1. **description**: The SQL environment of Princeton's InterCode framework. Puts an agent in front of a live MySQL instance and asks it to answer NL questions by issuing SQL, with full execution feedback between turns (errors, empty results, partial rows). Co-released with InterCode-Bash; meaningfully different from pure text-to-SQL because the agent is *expected* to explore the schema (SHOW TABLES, DESC) across turns before committing.
2. **github**: https://github.com/princeton-nlp/intercode — MIT, same repo as InterCode-Bash. Paper: NeurIPS 2023.
3. **tools**: One action = a SQL string passed to `SqlEnv.step()`. Observation is the MySQL response (rows or error). A `submit` action ends the episode. Agent policy is BYO (ReAct / Plan-and-Solve / vanilla).
4. **data**: Adapted from the Spider dev set — 1034 NL/SQL pairs across 20 databases. Each episode spins up a Dockerized MySQL pre-seeded with the relevant schema + data dump. Gold SQL is the oracle.
5. **tasks**: ~1034 SQL task instances in the main release. Difficulty follows Spider's easy/medium/hard/extra split. Baseline authors report ReAct reaches ~83% Pass@1 vs. ~47% zero-shot, showing how much multi-turn schema exploration helps.
6. **evaluation**: Deterministic execution-based. Run agent's final SELECT and the gold query against the fresh MySQL; compare result rows (value + record-order similarity). Reward 1.0 on match.
7. **playground**: Dockerized MySQL per task, reset between episodes. `docker exec`/driver-based command execution; no tmux.
8. **category**: terminal — database / SQL agent (multi-turn, execution-driven).

### BIRD-Interact
1. **description**: Interactive-agent reimagining of the BIRD-SQL leaderboard (June 2025). Pairs each database with a hierarchical knowledge base, metadata files, and a function-driven user simulator so the agent has to ask clarifying questions, recover from SQL errors, and handle follow-ups. Covers full CRUD (not just SELECT) on operational + analytical domains. Agents driving via shell or SQL client can be evaluated end-to-end — hence it fits the "DB terminal" slot.
2. **github**: https://github.com/bird-bench/BIRD-Interact (plus https://bird-bench.github.io/ for BIRD-SQL base) — open source.
3. **tools**: Two modes. `c-Interact` (conversational): turn-taking with the simulator + DB. `a-Interact` (agentic): tool-calling API — the simulator, knowledge base, and DB are each exposed as tools the agent can call in any order.
4. **data**: FULL (600 tasks, up to 11,796 interactions) and LITE (300 tasks with simplified DBs). Each task ships an ambiguous initial sub-task, clarification requirements, follow-up sub-tasks, and environmental uncertainties, all backed by executable test cases.
5. **tasks**: 600 full / 300 lite. CRUD operations on both analytical and operational schemas, with graded sub-goals per episode.
6. **evaluation**: Deterministic execution-based — test cases run against the resulting DB state after the agent signals completion. Multi-part scoring over sub-tasks. Reported SOTA: GPT-5 finishes 8.67% of c-Interact tasks, 17.00% of a-Interact.
7. **playground**: Containerized DB per task + simulated user process. Agent interacts through the provided tool API (or shell wrapper calling same tools).
8. **category**: terminal — database / SQL agent (multi-turn, clarification-heavy, CRUD).

### Spider 2.0 (+ Spider2-DBT)
1. **description**: ICLR 2025 Oral. The "enterprise" re-do of Spider: real-world text-to-SQL *workflows* on warehouse-scale schemas (>3000 columns), multiple dialects (BigQuery, Snowflake, DuckDB, PostgreSQL, ClickHouse), and operational tasks (transformation, analytics, ELT). The DBT variant (`Spider2-DBT`, 68 tasks) is the interesting one for agent-framework benchmarking: the agent is given a dbt repository and must edit/author dbt models and tests, executing them against a live warehouse.
2. **github**: https://github.com/xlang-ai/Spider2 — Apache-2.0, open source. Paper: ICLR 2025.
3. **tools**: Reference "spider-agent" uses a function-calling surface: `exec_sql`, `write_file`, `read_file`, `list_dir`, `dbt_run`, `dbt_test`, `submit`. Snow/Lite variants are text-in/text-out so any tool surface works; DBT variant requires shell + filesystem + warehouse connector.
4. **data**: 632 real-world enterprise problems in total. Splits: Spider 2.0 (full agentic), Spider 2.0-Snow (547 tasks, self-contained on Snowflake), Spider 2.0-Lite (text-in/text-out), Spider2-DBT (68 tasks, repository-level dbt workflow). Each ships schema dumps, BI documentation, and per-task expected output CSV / DB mutation.
5. **tasks**: 632 across all settings. Difficulty very high: GPT-4o solves ~10.1% on full Spider 2.0 (vs. 86.6% on Spider 1.0); GPT-4 ~6.0%.
6. **evaluation**: Deterministic. For SELECT tasks: result-set comparison with order/schema tolerance. For DBT/repo tasks: post-run warehouse state + dbt test pass/fail. Per-task grading script.
7. **playground**: Managed cloud DBs (Snowflake, BigQuery) provisioned by the benchmark, plus a local Python/dbt workspace for the agent. Snow variant needs Snowflake creds; Lite variant is fully offline.
8. **category**: terminal — enterprise data engineering / dbt + SQL agent.

### CRAB (Cross-environment Agent Benchmark)
1. **description**: CAMEL-AI's benchmark for multimodal agents acting across *multiple* environments simultaneously — the terminal slice is Ubuntu (plus Android phone as a second env). Unlike OS-only benchmarks, CRAB tasks require coordinating actions across the two environments (e.g. receive an SMS on Android -> act in Ubuntu shell). Introduces a graph-based fine-grained evaluation so partial credit is possible.
2. **github**: https://github.com/camel-ai/crab — Apache-2.0, open source. Paper: arXiv 2407.01511.
3. **tools**: Python-callable environment API per env. Ubuntu env exposes shell + GUI actions via pyautogui; agents can compose tool calls across envs in a single policy. Supports single-agent and multi-agent communication settings.
4. **data**: CRAB Benchmark-v0 — 120 tasks across Ubuntu + Android. Task generator included for extending the set. Graph-of-subgoals per task defines intermediate checks.
5. **tasks**: 120 cross-environment tasks. Mix of sysadmin-ish Ubuntu work, Android ops, and coordinated workflows. Reported best: GPT-4o single agent 38.01% completion; high "reach step limit" rates across models.
6. **evaluation**: Graph evaluator — each subgoal node is a deterministic check (file exists, process running, screen state). Final score = weighted subgoal completion + completion ratio + cost metrics.
7. **playground**: Ubuntu VM (KVM-backed) + Android emulator, both driven by CRAB's Python env API. Docker-optional. Heavier than pure Docker setups but still open-source.
8. **category**: terminal — cross-environment sysadmin + mobile ops (graph-scored).

### AIOpsLab
1. **description**: Microsoft Research framework for evaluating agents on cloud SRE / AIOps work. Deploys a real microservice app (DeathStarBench's HotelReservation or SocialNetwork) on Kubernetes, injects faults via ChaosMesh, exports telemetry (Prometheus + Jaeger + Filebeat), and asks the agent to detect, localize, diagnose, and mitigate. Closest thing to a pure "SRE in a terminal" benchmark.
2. **github**: https://github.com/microsoft/AIOpsLab — MIT, open source. Paper: arXiv 2501.06706 (SoCC 2024 / MLSys 2025).
3. **tools**: Provides a unified agent interface with tools for `kubectl`, `shell exec`, `read_logs`, `query_metrics`, `query_traces`, and `submit_diagnosis`. Tool-agnostic on the backend — you can wire in your own agent framework.
4. **data**: Extensible fault library covering ~48 distinct fault types (pod kill, CPU stress, network partition, config corruption, scale-down, etc.) crossed with two microservice apps -> hundreds of scenarios. Task = (app, fault, telemetry window).
5. **tasks**: Four canonical task types per scenario: detection, localization, root-cause analysis, mitigation. Detection + localization are auto-graded; mitigation verified by re-running workload and checking SLOs.
6. **evaluation**: Deterministic per task type. Detection: binary + time-to-detect. Localization: top-K component match. RCA: category/label match against ground truth. Mitigation: post-action health check against defined SLO thresholds.
7. **playground**: Kubernetes cluster (kind/minikube or full k8s) with the microservice app deployed; ChaosMesh injects faults on a schedule. Agent interacts through the provided shell/kubectl tools. Heavier setup than Docker-only benches but entirely open source (no proprietary VMs).
8. **category**: terminal — SRE / cloud operations on Kubernetes (fault diagnosis + remediation).

### ITBench (IBM)
1. **description**: IBM Research's open benchmark for AI agents on real-world IT automation. Ships three scenario families: SRE (Kubernetes/OpenShift incident response), CISO (compliance + security ops), FinOps (cloud cost optimization). Extends AIOpsLab's general shape with compliance + finops tracks and a standardized leaderboard. ICML 2025.
2. **github**: https://github.com/itbench-hub/ITBench (framework), https://github.com/IBM/ITBench-SRE-Agent (baseline agent), https://github.com/ibm/ITBench-Leaderboard — Apache-2.0, open source.
3. **tools**: Baseline SRE agent uses `kubectl`, shell exec, Prometheus query, Jaeger query, Clickhouse query, file read/write, and a final `submit_answer`. Framework is tool-agnostic — agents plug in through a standardized HTTP interface.
4. **data**: Scenarios span SRE (microservice incidents on real k8s), CISO (CIS-benchmark compliance drift), FinOps (cost anomaly resolution on cloud accounts). Each scenario ships a deployment manifest + fault/violation definition + expected resolution. Hugging Face release: `ibm-research/ITBench-Lite` + `ITBench-Trajectories`.
5. **tasks**: ~60 SRE + CISO + FinOps scenarios in ITBench-Lite; full set larger. Reported baselines: SOTA agents resolve 11.4% SRE / 25.2% CISO / 25.8% FinOps scenarios.
6. **evaluation**: Deterministic per scenario. SRE: post-action cluster/workload state. CISO: compliance scanner re-run. FinOps: cost-anomaly closure + projected savings check. Standardized JSON trajectory + score.
7. **playground**: Kubernetes (self-hosted or managed), plus cloud-provider emulators for FinOps. Scenarios encapsulated so you can run a slice locally in Docker + kind.
8. **category**: terminal — IT automation / SRE + compliance + finops (enterprise-shaped).

### DevOps-Gym
1. **description**: ICLR 2026 benchmark positioned as "the first end-to-end DevOps benchmark for agents." Covers four pipeline stages: (1) build & configuration, (2) monitoring, (3) issue resolution, (4) test generation. Tasks are sourced from real GitHub issues plus synthesized variants, and the agent drives actual DevOps tooling (Docker, CI configs, log parsers) inside a sandbox.
2. **github**: Referenced in ICLR 2026 paper (arXiv 2601.20882); public repo under `devops-gym` / accompanying the paper. Open source.
3. **tools**: Reference agent exposes `shell`, `file_read`, `file_write`, `git`, `docker`, `run_tests`, `read_logs`. Framework allows BYO tool scaffolds.
4. **data**: Real-world repos mirrored as task environments, paired with GitHub-derived issues + synthesized-but-executable tasks. Each stage defines its own grading script (build-pass, monitor-alert-resolved, issue-fix-passes-tests, generated-test-catches-known-bug).
5. **tasks**: Four tracks (build/config, monitoring, issue-resolve, test-gen). Reported SOTA: 51.85% / 20.56% / 23.87% / 13.87% across the four.
6. **evaluation**: Deterministic. Build/config: docker build + container health. Monitoring: alert acknowledged + correct root cause labeled. Issue resolution: tests pass post-patch. Test generation: generated test fails on buggy revision + passes on fixed revision.
7. **playground**: Per-task Docker environment with the target repo + DevOps tooling pre-installed. Shell-driven; no GUI requirement.
8. **category**: terminal — DevOps pipeline agent (build + monitor + fix + test, end-to-end).

### MLE-bench
1. **description**: OpenAI's benchmark for "ML engineering" agents. Drops the agent into a containerized Ubuntu workspace with a Kaggle competition's data + description and 24h to produce a submission. While framed as ML-engineering, in practice the agent lives in a shell — running pip, editing code, launching training, inspecting logs — so it's a strong terminal-skill proxy for long-horizon data-wrangling work.
2. **github**: https://github.com/openai/mle-bench — MIT, open source. Paper: arXiv 2410.07095.
3. **tools**: Tool-agnostic. OpenAI's reference runs use AIDE scaffolding (shell + file edit + run_python). Any framework exposing `execute_bash` + file I/O can adapt. Framework dispatch is via the `mlebench-env` Docker image.
4. **data**: 75 hand-selected Kaggle competitions: tabular, image, NLP, multimodal, sequence. Each ships Kaggle data, description, and a grading script that evaluates the agent's submission against the Kaggle leaderboard.
5. **tasks**: 75. Graded against human bronze/silver/gold medal thresholds. Reported SOTA: o1-preview + AIDE achieves bronze-or-better on 16.9% of competitions; frontier models still well below human median.
6. **evaluation**: Deterministic. Agent writes `submission.csv`; grader scores it against the official Kaggle metric for that competition and assigns a medal tier based on historical leaderboard percentiles.
7. **playground**: Docker container (`mlebench-env`, Ubuntu 20.04) with ~36 CPU / 440GB RAM / single A10 GPU, 24h wall-clock budget. Network access restricted; all data pre-mounted.
8. **category**: terminal — long-horizon ML engineering in shell (data wrangling + training + submission).
