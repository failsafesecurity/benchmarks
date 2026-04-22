# Office Work Benchmarks

### OfficeQA
1. **description**: OfficeQA evaluates how well AI systems can reason over real-world documents to answer complex questions. Uses U.S. Treasury Bulletin PDFs (1939-2025) with dense financial tables, charts, text.
2. **github**: https://github.com/databricks/officeqa — fully open source.
3. **tools**: Agent-as-is. No harness prescribed.
4. **data**: 696 PDFs + parsed JSON + LLM-friendly text + officeqa.csv mapping questions to source_files.
5. **tasks**: 246 single-turn grounded QA items with easy/hard difficulty labels.
6. **evaluation**: Deterministic. reward.py:score_answer(gt, pred, tolerance) → 1.0/0.0 with configurable numerical tolerance.
7. **playground**: None. Dataset + scoring helper.
8. **category**: office work / document QA.

### SpreadsheetBench
1. **description**: "Towards Challenging Real World Spreadsheet Manipulation" (NeurIPS D&B 2024 spotlight). 912 real-world questions scraped from online Excel forums — multiple tables per file, non-standard relational layouts, non-textual elements. Authors emphasize fidelity to real user frustrations; SOTA LLMs score ~17%, Copilot ~20%. A v2 has since extended it to multi-step workflow-level goals across multi-sheet workbooks.
2. **github**: https://github.com/RUCKBReasoning/SpreadsheetBench — fully open source (code + dataset on HF: KAKA22/SpreadsheetBench).
3. **tools**: Agent interacts with spreadsheet files. Each instruction ships with multiple input-output spreadsheet test cases; the agent must emit a solution (code or edited file) that passes all test cases — online-judge-style. No mandated tool stack, but a spreadsheet manipulation harness (openpyxl/pywin32 or similar) is effectively required.
4. **data**: 912 input Excel workbooks + corresponding output test-case workbooks (each task has several input/output pairs sharing structure, differing in data).
5. **tasks**: 912 real-user spreadsheet manipulation instructions. Domain: formulas, formatting, aggregation, data cleaning, cross-sheet reasoning.
6. **evaluation**: Deterministic. Output workbook must match expected workbook across all test-case pairs (online-judge verification).
7. **playground**: Pure dataset + evaluation harness. No live spreadsheet app simulated — the agent produces edited files or code.
8. **category**: office work / spreadsheet editing.

### SheetCopilot (benchmark component)
1. **description**: NeurIPS 2023 paper that ships both an agent framework and a benchmark of 221 spreadsheet control tasks across Google Sheets / Excel semantics. Explicitly tests closed-loop software control via a small set of atomic actions. SheetCopilot's own agent solves ~44% one-shot.
2. **github**: https://github.com/BraveGroup/SheetCopilot — fully open source.
3. **tools**: Prescribes a specific atomic-action API (Write, SetFormat, Sort, Filter, CreatePivotTable, etc.) implemented via pywin32 on Windows Excel. Agent must operate through this API rather than free-form file editing.
4. **data**: 221 tasks, each with a seed .xlsx workbook and a natural-language instruction; evaluation sheets for comparison.
5. **tasks**: 221 single-turn / short-horizon spreadsheet control tasks (formatting, pivots, charts, formulas, filtering).
6. **evaluation**: Deterministic. Evaluates final workbook state via property checks (value, format, chart presence) against reference sheets — set-level matching rather than strict cell diff.
7. **playground**: Live Excel via pywin32 (Windows) — closed-loop execution, not pure dataset.
8. **category**: office work / spreadsheet editing (with app-control scaffolding).

### OfficeBench
1. **description**: "Benchmarking Language Agents across Multiple Applications for Office Automation" (2024). Explicitly positioned as one of the first multi-app office-automation benchmarks; deploys agents in simulated office workflows that require switching between Word, Excel, email, and calendar tools. GPT-4o tops out at 47% pass rate.
2. **github**: https://github.com/zlwang-cs/OfficeBench — fully open source.
3. **tools**: Prescribed tool scaffolding. Ships a Docker image with Word, Excel, a calendar app, and an email client; agents must drive them via provided action APIs.
4. **data**: Task configs each seed the Docker sandbox with starter documents, spreadsheets, mailboxes, and calendars.
5. **tasks**: ~300 multi-app workflow tasks spanning document editing, spreadsheet edits, sending/replying to emails, and scheduling events. Core surface is cross-app (e.g., read email → extract data → update spreadsheet → send reply).
6. **evaluation**: Mixed — Exact Matching, Fuzzy Matching, and Execution-based Evaluation depending on task (file state diff, sent-email contents, calendar entries).
7. **playground**: Docker-sandboxed simulated office suite.
8. **category**: office work / multi-app workflow.

### OdysseyBench
1. **description**: Microsoft Research (2025). Extends OfficeBench-style multi-app evaluation with **long-horizon** workflows that require agents to pull context from extended interaction histories and reason across Word, Excel, PDF, Email, and Calendar over many steps. Targets agent memory and long-context planning specifically.
2. **github**: https://github.com/microsoft/OdysseyBench — fully open source.
3. **tools**: Multi-app tool scaffolding (Word/Excel/PDF/Email/Calendar interfaces) plus a long-context interaction-history mechanism. Pairs with a HomerAgents generation framework for synthesizing new tasks.
4. **data**: Seeded documents, emails, calendar events, and dialogue histories for each scenario.
5. **tasks**: 602 total — OdysseyBench+ (300 real-world-derived) + OdysseyBench-Neo (302 synthesized). Each task requires multi-step reasoning over long prior context.
6. **evaluation**: Execution-based checks against expected final state of each application plus task-specific assertions.
7. **playground**: Simulated office suite environment (similar envelope to OfficeBench) with extended interaction histories.
8. **category**: office work / multi-app workflow (long-horizon + memory).

### WorkArena / WorkArena++
1. **description**: ServiceNow (ICML 2024 / NeurIPS D&B 2024). Browser-based benchmark for enterprise knowledge-work tasks on a real ServiceNow instance — filtering lists, filling forms, searching knowledge bases, using service catalogs, reading dashboards. WorkArena++ composes the atomic tasks into 682 planning-heavy compositions. Best agents hit ~55% on L1.
2. **github**: https://github.com/ServiceNow/WorkArena (tasks) and https://github.com/ServiceNow/BrowserGym (environment) — fully open source, remote-hosted ServiceNow demo instance required (free developer tier).
3. **tools**: Prescribes a browser-agent stack. Uses BrowserGym action space (DOM queries, click/type, screenshots, accessibility tree). Not tool-agnostic — the evaluation assumes a web agent.
4. **data**: Live ServiceNow instance with seeded records (incidents, users, catalog items) re-initialized per task.
5. **tasks**: WorkArena L1: 33 atomic task templates × 19,912 instances. WorkArena++: 682 compositional tasks.
6. **evaluation**: Deterministic execution-based — checks final platform state (record created/updated, correct record opened, dashboard values read back).
7. **playground**: Live ServiceNow UI in a real browser via BrowserGym. VM-style but web only.
8. **category**: office work / multi-app workflow (enterprise SaaS, form-heavy knowledge work).

### CRMArena / CRMArena-Pro
1. **description**: Salesforce AI Research (NAACL 2025). Evaluates agents on professional CRM work in a real Salesforce org populated with 16 interconnected objects (accounts, cases, orders, knowledge articles, etc.) and realistic latent-variable distributions. Personas: Service Manager, Service Agent, Service Analyst. Best ReAct/GPT-4o scores 38.2% (54.4% with hand-crafted tools). CRMArena-Pro adds B2B/B2C and multi-turn scenarios (19 tasks).
2. **github**: https://github.com/SalesforceAIResearch/CRMArena — fully open source; dataset also on HF: Salesforce/CRMArena. Requires a free Salesforce developer org to run end-to-end.
3. **tools**: Prescribed CRM tool scaffolding — a tool catalog exposing Salesforce object operations (query, update, create). Agent must work through these APIs rather than a generic browser.
4. **data**: Populated Salesforce org schema + synthetic-but-realistic records, plus per-task configs.
5. **tasks**: CRMArena: 9 task types across 3 personas. CRMArena-Pro: 19 expert-reviewed tasks across sales, service, and pricing operations; multi-turn interactions supported.
6. **evaluation**: Deterministic execution-based — final Salesforce org state compared against expected diffs; for multi-turn, turn-level assertions.
7. **playground**: Live Salesforce developer org.
8. **category**: office work / multi-app workflow (CRM / enterprise SaaS).

### FinanceBench
1. **description**: Patronus AI (2023). First benchmark for LLM open-book financial QA over real SEC filings (10-Ks, 10-Qs, 8-Ks, earnings reports, earnings calls). Authors report GPT-4-Turbo + retrieval gets 81% of questions wrong or refused — motivating better retrieval and grounding for finance agents.
2. **github**: https://github.com/patronus-ai/financebench — open source under permissive license; the 150-case human-annotated evaluation subset is public. Full 10k dataset is gated.
3. **tools**: Agent-as-is. Explicitly "open book" — agents need retrieval over the bundled PDFs; choice of retriever/RAG stack is up to the implementer. Pairs naturally with a file-reader tool.
4. **data**: Bundled public SEC filing PDFs + 150 annotated QA pairs (question, answer, evidence passage, source doc).
5. **tasks**: 150 public single-turn grounded QA items (domain: financial statement analysis, MD&A, key metrics) — numeric, extractive, and reasoning subtypes.
6. **evaluation**: LLM-as-judge with rubric (provided judge prompts), graded against the annotated reference answer and evidence. Not purely deterministic due to answer-phrasing variance on numeric questions.
7. **playground**: None. Pure dataset + judge script.
8. **category**: office work / document QA (financial / RAG).

### AssistantBench
1. **description**: "Can Web Agents Solve Realistic and Time-Consuming Tasks?" (Tel Aviv U. / AI2 / UW / Penn / Princeton, 2024). Real-world, time-consuming information-seeking tasks typical of a personal assistant — questions requiring browsing many pages, cross-site aggregation, and sometimes arithmetic (e.g., "Which gyms near me have weekend classes before 7AM?"). Even SeePlanAct reaches only 11 points; closed-book LLMs hallucinate heavily.
2. **github**: https://github.com/oriyor/assistantbench — fully open source.
3. **tools**: Agent-as-is, but designed with a browsing harness in mind. Ships SeePlanAct as a reference web agent; compatible with generic browser/search tools.
4. **data**: 214 tasks sourced across 258 websites / 525+ pages, with reference answers and domain labels (shopping, travel, research, health, etc.).
5. **tasks**: 214 realistic personal-assistant queries requiring multi-step web research.
6. **evaluation**: Deterministic + partial credit — answers are structured (number, list, JSON) and compared with task-specific metrics (exact match, F1 over list items, numeric tolerance).
7. **playground**: Live web (requires a browsing agent or search tool). No seeded VM.
8. **category**: office work / multi-app workflow (knowledge-work web research).

### TableBench
1. **description**: "A Comprehensive and Complex Benchmark for Table Question Answering" (AAAI 2025). 886 curated tests across 18 subcategories under 4 major families: Fact Checking, Numerical Reasoning, Data Analysis, and Visualization. Designed to expose reasoning gaps that simpler TableQA benchmarks (WikiTQ, TabFact) miss — even GPT-4 leaves significant headroom.
2. **github**: https://github.com/TableBench/TableBench — fully open source (data on HF).
3. **tools**: Agent-as-is on text serialization of tables by default. Visualization subset benefits from a Python/code-execution tool. Does not prescribe a spreadsheet app.
4. **data**: Curated tables in structured form + 886 questions with labeled reasoning type and answer format (short answer, Python code for visualization, analytical narrative).
5. **tasks**: 886 QA items; categories include numerical reasoning, multi-step data analysis, and chart generation.
6. **evaluation**: Mixed. Deterministic exact/tolerance match for factual/numeric; ROUGE-style for analytical answers; execution + image check for visualization tasks.
7. **playground**: Pure dataset + scoring scripts. Code-execution sandbox recommended for visualization tasks.
8. **category**: office work / spreadsheet editing + document QA (table reasoning).

### MMLongBench-Doc
1. **description**: "Benchmarking Long-context Document Understanding with Visualizations" (2024). Long, visually rich PDFs (avg 47.5 pages, 21k tokens, 7 domains) requiring reasoning across text, tables, charts, and images. 33% of questions span multiple pages; 22.5% are intentionally unanswerable to catch hallucination. GPT-4o scores 44.9% F1 — comfortably unsaturated.
2. **github**: https://github.com/mayubo2333/MMLongBench-Doc — fully open source; integrated into VLMEvalKit.
3. **tools**: Agent-as-is for a multimodal model. Effective solutions typically add a PDF reader / page-selector tool or RAG over rendered pages. Does not prescribe a specific harness.
4. **data**: 135 multimodal PDFs + 1091 annotated QA pairs with evidence locations and modality tags.
5. **tasks**: 1091 grounded QA items over long PDFs; text-only, chart-only, table-only, image-only, and cross-page subsets.
6. **evaluation**: Deterministic reference-answer matching (F1 / exact on short answers) plus a special "unanswerable" label to penalize hallucinations.
7. **playground**: Pure dataset + scoring. VLMEvalKit provides a runner.
8. **category**: office work / document QA (long multimodal).

### PPTC / PPTC-R
1. **description**: "PPTC Benchmark: Evaluating LLMs for PowerPoint Task Completion" (ACL 2024 Findings). Multi-turn PowerPoint control benchmark — the agent must translate user instructions into API-call sequences against a PPTX model and arrive at a target slide deck. Deliberately stresses multi-turn accumulation: GPT-4 hits 75.1% single-turn accuracy but only 6% session-level accuracy. PPTC-R adds adversarial/robustness perturbations over PPT versions and paraphrased instructions.
2. **github**: https://github.com/gydpku/PPTC (main) and https://github.com/ZekaiGalaxy/PPTC-R (robustness extension) — fully open source.
3. **tools**: Prescribed PPT API scaffolding. Agent operates through a fixed set of python-pptx-style actions (add_slide, insert_textbox, set_font, insert_picture, set_shape_fill, etc.); evaluation runs the predicted API sequence against a PPTX canvas.
4. **data**: Seeded starter decks per session; expected deck states at each turn; reference API traces.
5. **tasks**: 279 multi-turn sessions with hundreds of instructions — mix of "create from scratch" and "edit existing template" scenarios, involving text, layout, formatting, images, and charts.
6. **evaluation**: Deterministic PPTX-Match Evaluation System — compares the *resulting* pptx state (property-level: text presence, font, shape geometry, color, etc.) against the reference rather than matching API call traces, so alternative correct solutions pass.
7. **playground**: Pure PPTX file manipulation harness — no live PowerPoint app; python-pptx-based sandbox.
8. **category**: office work / slides (PowerPoint control).

### PPTAgent + Zenodo10K / PPTEval
1. **description**: "PPTAgent: Generating and Evaluating Presentations Beyond Text-to-Slides" (EMNLP 2025). Two contributions: (i) an agentic framework for reflective PPT generation via edit-based actions over reference slides, and (ii) **PPTEval**, a rubric-based evaluation across Content, Design, and Coherence, plus the Zenodo10K dataset (10,448 presentations) as a reference pool. The benchmark component is the evaluation protocol + Zenodo10K-derived test set used to compare presentation generators; useful independently of the agent.
2. **github**: https://github.com/icip-cas/PPTAgent — fully open source (v2 adds deep-research integration and a 20-tool sandbox environment).
3. **tools**: Prescribes an edit-over-reference action space (select template slide → apply structured edits) rather than free-form PPTX authoring. V2 ships an agent sandbox with asset-creation, text-to-image, and visual design tools.
4. **data**: Zenodo10K — 10,448 real presentations across domains; held-out subset serves as references/targets for evaluation.
5. **tasks**: Presentation generation from a topic/outline instruction; evaluated holistically rather than by API trace match.
6. **evaluation**: PPTEval — rubric-based LLM-as-judge over Content (factual/contextual accuracy), Design (layout, color, readability), Coherence (narrative flow). Not deterministic; reproducibility depends on the judge model being pinned.
7. **playground**: Python-pptx-based sandbox; v2 adds a reflective agent environment with tool-use.
8. **category**: office work / slides (PowerPoint generation + design evaluation).

### TheAgentCompany
1. **description**: CMU (2024). Self-hosted simulation of a small software company — agents work across GitLab, Plane (project management), RocketChat (chat), and ownCloud (web Office suite) alongside LLM-powered "colleague" NPCs. Tasks are scoped to full workplace duties: HR, finance, admin, SWE, data analysis. Best agent completes only ~30% autonomously; ownCloud (Office docs) and RocketChat (communication) are the hardest surfaces — directly relevant to office-work evaluation despite the SWE framing. A follow-up `TheMCPCompany` (Oct 2025) reuses the same environment but swaps browser control for 18k MCP tools, isolating tool-use vs. UI-control performance.
2. **github**: https://github.com/TheAgentCompany/TheAgentCompany — fully open source (Docker images for every service). TheMCPCompany: https://github.com/Reza-esfandiarpoor/the-mcp-company.
3. **tools**: Prescribed — a browser-based agent stack by default (OpenHands), with tasks that require Office document edits via ownCloud's web UI, instant messaging via RocketChat, ticketing via Plane, and git ops via GitLab. TheMCPCompany variant exposes REST APIs as MCP tools instead.
4. **data**: Seeded intranet state — pre-populated repos, tickets, chat channels, document folders, simulated employees with backstories and scripted responses.
5. **tasks**: 175 diverse workplace tasks (file management, data gathering, report writing, cross-service orchestration, collaborating with simulated colleagues), with partial-credit checkpoints per task.
6. **evaluation**: Execution-based, multi-criterion — each task ships a checker script that verifies platform state (file contents, ticket fields, chat messages posted) plus intermediate milestones for partial scoring.
7. **playground**: Full Dockerized enterprise stack (GitLab + Plane + RocketChat + ownCloud + NPC agents). The richest open office-work playground currently available.
8. **category**: office work / multi-app workflow (simulated workplace, including Office documents and chat).

### EnterpriseBench
1. **description**: "Can LLMs Help You at Work?" (2025). Enterprise-environment sandbox covering software engineering, HR, finance, and administrative workflows with data-source fragmentation, access-control hierarchies, and cross-functional tasks explicitly baked in. Strongest agents hit only 21.5% completion — the benchmark specifically targets realistic enterprise frictions (permissions, data silos) that OfficeBench/WorkArena don't model.
2. **github**: Project page https://ast-fri.github.io/EnterpriseBench/ (OpenReview submission Vw5uftspxQ). Open data generation pipeline; Surge's commercial "CoreCraft" instance is separate.
3. **tools**: Prescribed enterprise tool catalog per task. Data-source fragmentation (HR system, finance system, docs) is part of the challenge — agents must pick the right system and respect role-based access.
4. **data**: Synthesized but internally consistent enterprise datasets generated from organizational metadata (employees, departments, permissions, documents, tickets).
5. **tasks**: 550 tasks across software-engineering, HR, finance, and admin domains. Cross-functional composition is explicit (e.g., HR request → finance approval → SWE ticket).
6. **evaluation**: Execution-based state checks per task; respects access-control correctness (wrong-role actions count as failures).
7. **playground**: Enterprise simulation sandbox with permission model.
8. **category**: office work / multi-app workflow (enterprise with access control).

### LOFT
1. **description**: "Can Long-Context Language Models Subsume Retrieval, RAG, SQL, and More?" — Google DeepMind (NAACL 2025 Findings). Long Context Frontiers benchmark: 35 datasets × 6 task categories × 4 modalities (text, vision, audio, code), at 32k/128k/1M token context lengths. Explicitly tests whether a long-context LCLM can replace a RAG/SQL/retrieval pipeline — highly relevant for "can I just stuff the whole corporate doc corpus in context?" agent design decisions. Multi-hop compositional reasoning is where LCLMs still lag dramatically.
2. **github**: https://github.com/google-deepmind/loft — fully open source.
3. **tools**: Agent-as-is. Designed to compare monolithic LCLM-only solutions to pipeline-style retrieval/RAG/SQL systems; ships "corpus-in-prompt" (CiP) templates that serialize full corpora into the context window.
4. **data**: 35 bundled datasets spanning retrieval, RAG, multi-hop QA, SQL-over-tables, long-doc ICL, and compositional reasoning; context lengths auto-scale with the target length.
5. **tasks**: Thousands of items across the 35 subsets. Task mix covers document QA, structured-data QA (SQL), and multi-hop reasoning — the closest thing to a comprehensive "office-RAG" evaluation suite.
6. **evaluation**: Deterministic per-dataset metrics (EM, F1, execution accuracy for SQL) combined with LOFT's length-scaling protocol.
7. **playground**: Pure dataset + eval scripts; agent frameworks supply the retrieval/RAG backbone to compare against.
8. **category**: office work / document QA (long-context RAG alternatives).

### M-LongDoc
1. **description**: "M-LongDoc: A Benchmark for Multimodal Super-Long Document Understanding" (EMNLP 2025). 851 open-ended QA samples over documents averaging **200+ pages** each — academic papers, company reports, product manuals — requiring reasoning over interleaved text, tables, and figures. Unlike MMLongBench-Doc (avg 48 pages, short answers), M-LongDoc demands *explanatory* answers that synthesize text and images, and includes a retrieval-aware tuning framework as a companion baseline.
2. **github**: Project page https://multimodal-documents.github.io/ — fully open source; paper arxiv:2411.06176.
3. **tools**: Agent-as-is with an encouraged RAG / page-selector scaffold; the authors ship a retrieval-aware training recipe but the benchmark itself only scores the generated answer.
4. **data**: Multimodal super-long documents (avg >200 pages) + 851 open-ended QA samples with reference answers and modality tags.
5. **tasks**: 851 explanatory QA items — each answer should integrate textual and visual evidence from hundreds of pages.
6. **evaluation**: Automated framework that scores correctness against reference answers; uses LLM-judged rubric scoring for open-ended outputs rather than EM/F1.
7. **playground**: Pure dataset + scoring. RAG/retriever is BYO.
8. **category**: office work / document QA (multimodal, super-long reports and manuals).

### CRAG (Comprehensive RAG Benchmark)
1. **description**: Meta AI (NeurIPS 2024 / KDD Cup 2024). 4,409 QA pairs across 5 domains (finance, sports, music, movies, open) and 8 question types — popular vs. long-tail, static vs. fast-changing — paired with mock web-search and mock Knowledge-Graph APIs. Meta reported top LLMs ≤34% accuracy; naive RAG lifts to 44%; SOTA industrial RAG hits 63% hallucination-free. Office-relevant because it's the most rigorous open benchmark for the retrieval-and-ground step that every office QA agent needs.
2. **github**: https://github.com/facebookresearch/CRAG — fully open source (data + mock APIs). KDD Cup 2024 evaluation harness also public via AIcrowd.
3. **tools**: Prescribed mock web-search API (5 pages/question in Task 1, 50 in Task 3) and mock KG query API (Task 2+3). Agents must exercise retrieval, grounding, and hallucination control.
4. **data**: 4,409 annotated QA pairs + pre-crawled web snippets + synthetic KG endpoints.
5. **tasks**: Three progressive tracks — web-retrieval summarization, KG-augmented QA, end-to-end RAG with 50 pages + KG mix. Includes adversarial question types: false premise, temporal, aggregation, multi-hop.
6. **evaluation**: Deterministic scoring with explicit penalties for hallucinations (wrong answers score worse than "I don't know"). Standardized metric used across KDD Cup submissions.
7. **playground**: Mock retrieval services packaged with the dataset.
8. **category**: office work / document QA (RAG benchmark with hallucination accounting).

### AstaBench
1. **description**: Allen Institute for AI (Oct 2025). A holistic benchmarking suite for scientific research agents — 2,400+ problems across 11 sub-benchmarks in 4 areas: literature understanding, code & execution, data analysis, end-to-end discovery. Includes LitQA2, PaperFindingBench, ScholarQA-style tasks. Critically, AstaBench measures **cost** alongside quality and ships baseline agents for controlled comparison — making it the best-engineered "research-assistant" eval currently available.
2. **github**: https://github.com/allenai/asta-bench — fully open source; project page https://allenai.org/asta/bench.
3. **tools**: Prescribes standard production-grade research tools (paper search, full-text retrieval, code sandbox, data analysis env) so agents are compared on reasoning, not tool access. Baseline ReAct / Asta Paper Finder agents included.
4. **data**: Aggregated across the 11 sub-benchmarks; literature tasks use recent-paper pools, code/data tasks ship bundled corpora.
5. **tasks**: 2,400+ across literature QA (LitQA2), paper-finding, code execution, data analysis, and end-to-end research discovery.
6. **evaluation**: Per-benchmark deterministic + LLM-judge metrics; **cost-normalized** leaderboard reports $/task alongside accuracy.
7. **playground**: Asta tool ecosystem (Paper Finder, code sandbox) provided as a reference stack.
8. **category**: office work / document QA (research-assistant / scientific knowledge work).

### LegalBench
1. **description**: Stanford HazyResearch (NeurIPS 2023). 162 legal-reasoning tasks collaboratively built by 40 contributors (lawyers, paralegals, law professors) across six legal reasoning types and many document types (statutes, contracts, opinions). The go-to "office work for legal teams" benchmark: contract analysis, hearsay classification, rule-based reasoning, holding extraction. Still actively used as a baseline for legal-ops agents.
2. **github**: https://github.com/HazyResearch/legalbench — fully open source; data mirrored on HF (`nguha/legalbench`).
3. **tools**: Agent-as-is. Tasks are prompt-level classification/extraction/generation; agents can layer retrieval over statute corpora but the core benchmark doesn't prescribe tools.
4. **data**: 162 tasks with train/test splits, plus per-task prompt templates and rubrics contributed by legal practitioners.
5. **tasks**: Binary classification, multi-class classification, extraction, entailment, generation — across contracts (CUAD-derived tasks), civil procedure, evidence rules, tax, and corporate law.
6. **evaluation**: Deterministic per-task metrics (accuracy / F1 / exact match depending on task type). No LLM-as-judge dependence for most tasks.
7. **playground**: Pure dataset + scoring. Legal document corpus bundled.
8. **category**: office work / document QA (legal / domain-specific).

### Hybrid Financial Table QA (TAT-QA + FinQA + MultiHiertt)
1. **description**: A cluster of closely-related open benchmarks for **hybrid table-and-text reasoning over financial filings** — the canonical skill missing from pure spreadsheet benches like SpreadsheetBench and pure text QA like FinanceBench. TAT-QA (ACL 2021): 16,552 QA pairs over real financial report excerpts mixing prose + tables, with arithmetic and span answers. FinQA (EMNLP 2021): 8,281 expert-annotated numeric-reasoning questions with DSL-style reasoning programs. MultiHiertt (ACL 2022): 10,440 questions over **multiple hierarchical tables plus long text** per document — the hardest of the three. Worth carding as a unit because office-work agents typically need coverage across all three.
2. **github**: TAT-QA: https://github.com/NExTplusplus/TAT-QA; FinQA: https://github.com/czyssrs/FinQA; MultiHiertt: https://github.com/psunlpgroup/MultiHiertt — all fully open source.
3. **tools**: Agent-as-is with a code-execution tool recommended. FinQA and MultiHiertt explicitly evaluate generated reasoning programs; TAT-QA supports both direct-answer and program modes.
4. **data**: Real-world financial report excerpts (hybrid table+text) with annotated answers, arithmetic derivations, and supporting-evidence spans.
5. **tasks**: ~35k QA items combined; numeric reasoning, span extraction, multi-hop across table + narrative, and (in MultiHiertt) cross-table reasoning over hierarchical tables.
6. **evaluation**: Deterministic per-dataset — EM / numeric-tolerance match on answers; program execution accuracy on FinQA/MultiHiertt.
7. **playground**: Pure datasets + scoring. Python sandbox optional for program-mode eval.
8. **category**: office work / spreadsheet editing + document QA (financial hybrid table reasoning).

## Also considered (noted, not carded)

- **GAIA** — already supported as a suite in this harness. Covers generalist assistant tasks including document QA, web, and tool use; overlaps with several cards above on the "realistic assistant" axis.
- **DocVQA / ChartQA** — classic single-doc QA, no tool use / workflow dimension; better covered by MMLongBench-Doc in the long-context setting.
- **HotpotQA / MuSiQue** — multi-doc reading comprehension; not agentic (no tool use, no write-side actions). Deprioritized.
- **SUPER (AI2)** — strong benchmark but scoped to ML/NLP research-repo setup & execution; more coding-agent than office-work. See https://github.com/allenai/super-benchmark if you want a research-document / code-agent hybrid.
- **Enron-based email evals** — existing public usage is dominated by privacy and classification work (e.g., LLM-PBE). No mature, open, agentic "email triage" benchmark surfaced; would likely need to be built in-house.
- **ColBench (facebookresearch/sweet_rl)** — collaborative artifact creation (code, web pages, slides) with a simulated human partner. Interesting for multi-turn RL eval but the "office" surface is limited to slide/webpage generation; scored via win-rate against human preferences, not deterministic office state.

- **PPTBench (arxiv:2512.02624)** — 4,439 VLM samples over 958 PPTX files across Detection/Understanding/Modification/Generation. Strong VLM evaluation angle but VQA-shaped (no agent loop, no tool scaffolding) — PPTC/PPTAgent cover the agentic slide surface better.
- **SlidesBench / AutoPresent (CVPR 2025)** — 7k train / 585 test slide-generation tasks from 310 decks. Solid dataset but overlaps heavily with the PPTAgent card; the evaluation is reference-similarity + design-quality rubric. Consider if you specifically want reference-based slide scoring.
- **OmniACT (ECCV 2024)** — 9.8K desktop+web GUI automation tasks via PyAutoGUI. Office-suite slices exist but the benchmark is UI-control-centric and script-accuracy-scored rather than office-state-scored; better cataloged under browser/GUI benches.
- **ToolQA (NeurIPS 2023)** — 13 tools × 8 domains. Sensible tool-use eval but domains (flights, coffee, DBLP) aren't office-work-shaped; better placed in a generic tool-use anthology.
- **InfiniteBench / HELMET** — long-context evals (100k+ tokens). Useful for picking a long-context base model, but task mix is synthetic-heavy (NIAH, passkey) and novel-oriented rather than office-document-oriented. LOFT and M-LongDoc are stronger office proxies.
- **DocBench** — 229 docs + 1,102 QA across academia/finance/government/law/news. Solid but format-overlaps with FinanceBench + MMLongBench-Doc + LegalBench already carded.
- **BizBench / BizFinBench / EnterpriseArena** — financial/enterprise quantitative reasoning. Useful but narrower than FinanceBench + FinQA-cluster + EnterpriseBench already carded.
- **AgentClinic** — simulated clinical diagnosis with patient + doctor + measurement agents. Genuinely agentic, but medical-office-specific; card in a healthcare anthology instead of general office work.
- **AmbigDocs** — 36k multi-doc disambiguation QA pairs from Wikipedia. Interesting failure mode (entity-name collisions) but no workflow dimension; single-QA classification-style.
- **WritingBench / LongGenBench** — long-form writing evals. WritingBench uses LLM-as-judge with query-dependent rubrics (1,239 queries, 6 domains); LongGenBench tests 16K/32K generation with instruction constraints. Worth adding if you add an explicit "long-form writing" cell to your matrix; they're not agentic by themselves.
- **LawBench** — Chinese-law analog of LegalBench. Skipped to avoid legal-bench duplication; card if you need Chinese-jurisdiction coverage.
