# Financial / Transaction Benchmarks

### tau-bench (retail + airline)
1. **description**: Benchmark for tool-agent-user interaction in real-world customer-service domains. Paper (Yao et al., 2024) frames it as the first benchmark where an agent must both follow a written domain policy and satisfy a simulated human user, using a tool backend that mutates a persistent database. Already integrated in this harness (`src/adapters/tau_bench.rs`); mentioned here only for completeness.
2. **github**: https://github.com/sierra-research/tau-bench — MIT, fully open source and runnable.
3. **tools**: Prescribed tool scaffolding. Each domain ships a fixed Python tool registry (retail: order lookup, return, exchange, modify_address, cancel_pending_order, etc.; airline: search_trip, book_reservation, update_reservation, refund, send_certificate). The agent must invoke these as function calls; no free-form shell.
4. **data**: Seeded JSON databases per domain (users, orders, products, reservations, flights) plus a domain policy doc and a set of task specs (user persona + initial state + goal).
5. **tasks**: Retail ~114 tasks, airline ~50 tasks. Multi-turn dialogues with an LLM-simulated user; the agent mixes natural-language replies with tool calls until the user ends the conversation.
6. **evaluation**: Deterministic state-diff. At conversation end the harness compares the resulting DB state against an annotated goal state; optional output-string check. `pass^k` reliability metric over k independent rollouts.
7. **playground**: In-process simulated API. No network, no docker; tool functions mutate an in-memory copy of the seeded DB.
8. **category**: financial transactions — retail customer support, booking/refund ops.

### tau2-bench (airline, retail, telecom, banking_knowledge, mock)
1. **description**: Successor to tau-bench introducing dual-control: both the agent and the simulated user hold tools that mutate the shared world state (critical for tech/bank support where the user must "click", "read back a code", etc.). τ³ release adds 75+ corrected tasks and voice-mode (full-duplex) eval. Amazon AGI maintains a verified fork (`amazon-agi/tau2-bench-verified`) fixing reward misalignments. Highest-signal successor of tau-bench for customer-service agents.
2. **github**: https://github.com/sierra-research/tau2-bench — MIT, open source, installable via `uv sync`; `amazon-agi/tau2-bench-verified` for the cleaned variant.
3. **tools**: Prescribed per-domain tool registries, now split into agent-tools and user-tools. Telecom adds device-diagnostic actions; banking_knowledge layers a configurable RAG pipeline (document search, embeddings, agentic shell search) on top of the standard agent tools.
4. **data**: Seeded JSON state per domain plus policy docs. Banking_knowledge ships a document corpus and an embedding index; telecom ships customer+device+line records.
5. **tasks**: Retail ~114, airline ~50, telecom ~100+, banking_knowledge ~20+. Multi-turn, text (half-duplex) and voice (full-duplex) modes. Each task pins user persona, initial DB state, goal state, and expected action trajectory.
6. **evaluation**: Deterministic state-diff against goal DB plus optional required-action and communicated-info checks. `pass^k` kept from τ-bench; τ³ tightened reward computation for airline/retail.
7. **playground**: In-process simulated API servers; optional voice providers (OpenAI/Gemini/xAI realtime). No docker required.
8. **category**: financial transactions — retail support, booking, telco account ops, banking knowledge/retrieval.

### AppWorld
1. **description**: ACL'24 Best Resource Paper. A high-fidelity simulated "world" of 9 consumer apps populated with digital lives of 106 simulated people. Agents are asked to complete natural requests by writing Python that calls app APIs, e.g. "Split my rent with my roommates on Venmo based on the Splitwise balance." Heavy on cross-app transactional flows (banking-adjacent: Venmo money movement, Splitwise ledger, Amazon purchases).
2. **github**: https://github.com/StonyBrookNLP/appworld — Apache-2.0, fully installable (`pip install appworld && appworld install && appworld download data`); supports in-process and remote (Docker/HTTP) execution.
3. **tools**: Prescribed. 457 APIs across Spotify, SimpleNote, Amazon, Venmo, Gmail, Splitwise, FileSystem, Todoist, Phone (plus a "supervisor" meta-app for user context). Exposed in OpenAI function-calling and OpenAPI schemas. Agent code runs in a Python sandbox that calls these APIs.
4. **data**: Fully seeded databases for every app — users, balances, contacts, transactions, orders, notes, calls, messages — regenerable deterministically.
5. **tasks**: 750 tasks split across train / dev / test_normal / test_challenge. Each task is a natural-language request; interactions are multi-turn code-act loops (agent writes Python → executes → observes).
6. **evaluation**: Deterministic state-based unit tests (SGC/TGC). Checks both goal state achieved and absence of collateral damage; supports multiple valid completion paths.
7. **playground**: Full simulator bundled — runs locally as a Python service or Docker HTTP server, no external network.
8. **category**: financial transactions — retail commerce, P2P payments (Venmo), shared-expense accounting (Splitwise).

### BFCL (Berkeley Function Calling Leaderboard, V3/V4)
1. **description**: De-facto standard for pure function-calling ability. V3 added multi-turn, multi-step function calls; V4 adds agentic web search, memory management, and format sensitivity. Not finance-specific, but a sizable slice of the multi-turn categories is framed as personal-finance, stock-quote, and banking-adjacent scenarios (stock lookup, portfolio math, currency conversion, booking). Useful as a low-level complement to scenario benches.
2. **github**: https://github.com/ShishirPatil/gorilla/tree/main/berkeley-function-call-leaderboard — Apache-2.0, open source.
3. **tools**: Mixed. Single-turn categories feed the agent synthetic tool specs and grade the emitted call. Multi-turn categories ship an executable mock tool backend (stateful Python classes for trading, vehicle control, travel, gorilla-file-system, message API) so calls really run.
4. **data**: ~4k+ entries across AST-match (simple, parallel, multiple, relevance), executable (REST + Python), multi-turn base/miss_func/miss_param/long_context/composite. HuggingFace: `gorilla-llm/Berkeley-Function-Calling-Leaderboard`.
5. **tasks**: Single-turn (one prompt → one or more calls) plus multi-turn sessions where the agent must chain calls and recover from missing functions/parameters. Mostly single-shot; multi-turn categories extend to tens of turns.
6. **evaluation**: Deterministic. AST equivalence for single-turn; actual execution + state comparison for executable and multi-turn. No LLM judge.
7. **playground**: Executable mock backends included in-repo (e.g., `TradingBot`, `MathAPI`, `TravelAPI`); no external services.
8. **category**: financial transactions — trading/brokerage function-calling, currency/FX utilities (as one slice of a broader tool-use bench).

### FinanceBench (Patronus AI)
1. **description**: Large-scale open-book financial QA over real SEC filings (10-Ks, 10-Qs, 8-Ks, earnings reports, transcripts). Best-in-niche for retrieval-grounded financial reasoning at scale; authors position it as the first serious LLM-finance QA benchmark. Not agentic, but included per the exception in the brief because of scale and because it is commonly wrapped with a retrieval tool loop.
2. **github**: https://github.com/patronus-ai/financebench — CC-BY-4.0 / MIT-style; fully open for the 150-item human-annotated subset. Full 10,231-item set referenced in paper but only sample is public.
3. **tools**: Agent-as-is by default, but PDFs are provided so it's trivially extended to a RAG/agent loop with a `search_filings` / `read_page` tool.
4. **data**: 10,231 QA pairs (150 open) with answers and evidence strings, plus the underlying SEC PDFs.
5. **tasks**: Single-turn open-book QA. Each item: question + evidence doc(s) + gold answer.
6. **evaluation**: LLM-as-judge with a rubric (correct / incorrect / refusal) plus an evidence-citation check. Authors report reference prompts; not pure string match.
7. **playground**: Pure dataset + evaluation notebook (`evaluation_playground.ipynb`); no simulated backend.
8. **category**: financial transactions — compliance / financial-document QA (retrieval-style, not action-space).

### InvestorBench
1. **description**: ACL 2025 benchmark for LLM-agent financial decision-making across single equities, crypto, and ETFs. Generalizes FinMem/FinAgent into a reusable framework; evaluates thirteen LLMs across market regimes. Agent reads daily multi-modal signals (price, news, filings, reflection memory) and outputs buy/sell/hold.
2. **github**: https://github.com/felis33/INVESTOR-BENCH — MIT, fully open source. Docker-compose + justfile, runs with VLLM for open-weight models or hosted APIs for closed models.
3. **tools**: Prescribed scaffolding. Agent has a memory module (Qdrant vector DB) plus fixed action space: Buy/Sell/Hold with position size. No free-form web access.
4. **data**: Curated multi-modal datasets per asset: prices, news, 10-K snippets, technical indicators. Crypto (BTC, ETH) Feb–Dec 2023; equities (HON, JNJ, UVV, MSFT) Jul 2020 – May 2021; warmup + test split.
5. **tasks**: One long-horizon sequential trading episode per (asset × period). Multi-turn in the sense of repeated daily decisions; hundreds of steps per episode.
6. **evaluation**: Deterministic financial metrics: cumulative return, Sharpe, max drawdown, win rate — computed from the simulated ledger. No LLM judge.
7. **playground**: Self-contained simulator — VLLM server + Qdrant + a backtest-style environment; all dockerised.
8. **category**: financial transactions — trading / investment decision-making.

### StockBench
1. **description**: 2025 benchmark (arXiv 2510.02209) for "can LLM agents trade stocks profitably". Contamination-free: uses post-2024 market data the models were not trained on. Frames the agent as a portfolio manager that receives a daily signal packet and issues trades with sized positions; strong overlap with LiveTradeBench but runs offline/reproducibly.
2. **github**: https://github.com/ChenYXxxx/stockbench — open source.
3. **tools**: Prescribed. Daily input = prices + fundamentals + news; fixed action space (buy/sell/hold with quantity) operating on a simulated brokerage account. Polygon and Finnhub data pre-packaged.
4. **data**: 20 DJIA stocks selected for diversity; 82 trading days (March–June 2025); $100k starting capital per agent; news + fundamentals snapshots per day.
5. **tasks**: One sequential trading task per model across 82 trading days × 20 stocks (portfolio-level). Multi-turn decision loop.
6. **evaluation**: Deterministic finance metrics — cumulative return, max drawdown, Sortino ratio — benchmarked against a buy-and-hold baseline.
7. **playground**: Offline market simulator bundled with data; no live API calls during eval.
8. **category**: financial transactions — trading.

### LiveTradeBench
1. **description**: UIUC U-Lab (Nov 2025 preprint). Evaluates LLM trading agents in live markets (US equities via Yahoo Finance + Polymarket prediction markets) over 50-day windows. Portfolio-management abstraction (multi-asset allocation, not just single-asset actions). Notable finding: LMArena score has near-zero or negative correlation with trading PnL.
2. **github**: https://github.com/ulab-uiuc/live-trade-bench — open source, pip-installable; mock mode for offline tests.
3. **tools**: Prescribed. Data tools (price fetcher, NewsAPI, Finnhub, Reddit/PRAW sentiment); action tools (portfolio rebalance). RESTful API for plugging external agents.
4. **data**: Live streams — no fixed dataset; historical replay available via backtester.
5. **tasks**: Continuous live evaluation. A "task" is an allocation decision per market per day; benchmarked 21 LLMs over 50 days in the paper.
6. **evaluation**: Deterministic financial metrics (return, Sharpe, drawdown, turnover). No judge.
7. **playground**: Live market simulator/executor with both live and mock backends; dashboard at trade-bench.live.
8. **category**: financial transactions — trading / portfolio allocation.

### FinBen
1. **description**: NeurIPS 2024 D&B. 42 datasets, 24 tasks across 8 aspects (extraction, QA, generation, risk, forecasting, decision-making, bilingual EN/ES). First financial bench to bundle an agent-evaluation track and a stock-trading environment alongside classic NLP tasks; useful as a breadth reference though most tasks are non-agentic.
2. **github**: https://github.com/The-FinAI/finben (sibling: https://github.com/The-FinAI/PIXIU) — open source, runs via `lm-evaluation-harness`.
3. **tools**: Mostly agent-as-is for the NLP slice. Trading/decision-making subtasks prescribe a simple action interface (buy/sell/hold on historical bars); RAG subtasks plug into the harness's retriever.
4. **data**: Financial QA corpora (FPB, FiQA, Headline, NER, FinQA, ConvFinQA, TATQA), forecasting series, stock data, Spanish finance data, regulatory docs.
5. **tasks**: ~42 datasets. Mostly single-turn; the trading/agent subtasks are multi-step episodic.
6. **evaluation**: Mixed — mostly deterministic (accuracy/F1/EM/MCC/MSE); trading subtask uses financial metrics; some generation tasks use rubric-style scoring.
7. **playground**: Dataset + harness plugin. Trading simulator is minimal (price replay).
8. **category**: financial transactions — trading + compliance/extraction (broad umbrella).

### BizBench
1. **description**: ACL 2024 (Kensho). Eight quantitative-reasoning tasks focused on program-synthesis over financial data — the agent must produce executable code that computes a numeric answer from SEC filings / tables. Included as the best representative of "code-action finance reasoning" without a full app simulator.
2. **github**: Data at https://huggingface.co/datasets/kensho/bizbench (held-out leaderboard at https://benchmarks.kensho.com/). Sample pipeline: https://github.com/kensho-technologies/benchmarks-pipeline. Dataset is CC-licensed; test labels held out.
3. **tools**: Agent-as-is with a Python executor. Some tasks ship structured tables; others require extracting values first, then computing.
4. **data**: Augmented QA over 10-K tables + narrative text; three new code-generation splits.
5. **tasks**: ~8 subtasks totalling a few thousand single-turn items (QA + program synthesis).
6. **evaluation**: Deterministic. Numeric answer match (with tolerance) and code-execution against gold outputs.
7. **playground**: Dataset + code executor; no simulated bank/brokerage.
8. **category**: financial transactions — accounting / financial analysis (code-act flavour).

### Finance Agent Benchmark (Vals AI)
1. **description**: Stanford + Vals AI + G-SIB bank collaboration (arXiv 2508.00828, Aug 2025). Explicitly evaluates LLM agents as entry-level financial analysts doing SEC-filing research. Best-in-niche agentic counterpart to FinanceBench: where FinanceBench is a closed-book QA corpus, Finance Agent ships the tools and requires the agent to find, parse and compute answers itself. Leaderboard tracks closed models live; claude-opus-4 is current #1 and OpenAI o3 sits ~46.8% at ~$3.79/query.
2. **github**: https://github.com/vals-ai/finance-agent — MIT, fully open harness. Dataset on HF: `vals-ai/finance_agent_benchmark` (50-item public validation split open; 150 private validation + 337 test held-out behind license).
3. **tools**: Prescribed tool set geared at filing research: EDGAR search via SEC_API, Google/Tavily web search, `ParseHTML` document parser, `RetrieveInformation` over prior agent steps. Agent code runs locally via `uv`; per-provider API keys required.
4. **data**: 537 expert-written questions spanning simple retrieval, market research, projections and valuation. Public-validation (50), private-validation (150, licensed), test (337, held out).
5. **tasks**: Single-goal multi-step agent sessions — each question is one task, but the agent typically issues many tool calls (EDGAR lookups, page retrievals, arithmetic) per item.
6. **evaluation**: Hybrid. Programmatic answer match for numeric/categorical items; LLM-as-judge rubric for free-form synthesis. Test-set grading is run by Vals on their infra to prevent leakage.
7. **playground**: Local CLI (`finance-agent --questions`); no simulator, real EDGAR/Google calls with rate-limited keys.
8. **category**: financial transactions — analyst workflow over SEC filings (agentic retrieval + calculation).

### FinAgentBench
1. **description**: NeurIPS 2025 benchmark (arXiv 2508.14052) for *agentic retrieval* in finance — the first bench that explicitly separates "which filing do I open?" from "which chunk inside it do I cite?". Targets the common failure mode of RAG pipelines over dense SEC documents and is aimed at LLM agents, not bare embedding models.
2. **github**: paper on arXiv + ACM ICAIF 2025 proceedings; data released under CC-BY-4.0. Author-provided code URL in paper; HF dataset mirror planned.
3. **tools**: Prescribed. The agent gets a document-level ranker tool plus a chunk-level reranker over the chosen document; the harness measures both stages independently.
4. **data**: ~26k expert-annotated (query, document-type, chunk) triples over S&P 500 filings — 10-K, 10-Q, 8-K, earnings transcripts, DEF 14A proxies.
5. **tasks**: Single-turn agentic retrieval per query, but each query forces a two-step plan (doc selection → chunk selection). Fine-tuning split is provided alongside eval.
6. **evaluation**: Deterministic ranking metrics — document-level nDCG/MRR for stage 1 and chunk-level nDCG/Recall@k for stage 2. No LLM judge.
7. **playground**: Dataset-only; retrieval tools are pluggable. No simulated market/app state.
8. **category**: financial transactions — compliance / filing research (retrieval-focused agent bench).

### CryptoBench
1. **description**: First serious DeFi/crypto agent benchmark (arXiv 2512.00417, Dec 2025). Dynamic: 50 questions per month refreshed so that tickers, wallet addresses, tx hashes and time windows change, reducing contamination. Expert-curated by crypto-native analysts. Notable finding: a "retrieval–prediction gap" (GPT-5 hits 58.8% on retrieval tasks, 6.25% on prediction) that parallels LiveTradeBench's LMArena-vs-PnL null correlation.
2. **github**: https://github.com/xxcg322/CryptoBench — CC-BY-4.0. Ships a BenchmarkAgent scaffold, `Testing.py` / `Scoring.py` / `MC_Test.py`, and an MVP multi-choice set (727 items) alongside the 230-task dynamic set.
3. **tools**: Agents are given a general web-browsing tool (navigate, search, extract) and nothing else — explicitly no preferential API access. The benchmark tests whether agents can use Nansen / Arkham / Etherscan / Dune / DEX dashboards cold.
4. **data**: Monthly 50-question releases (parameterised templates), plus Task Dataset (230 complex tasks) and MVP Dataset (727 MCQs). Domains: on-chain intelligence (40%), derivatives (18%), DeFi analytics (12%), DEX data (8%), other (22%).
5. **tasks**: Four quadrants — Simple Retrieval, Complex Retrieval, Simple Prediction, Complex Prediction. Tasks can span DeFi protocol interaction, MEV signal analysis, DAO governance.
6. **evaluation**: LLM-as-judge with a 0–3 rubric (Incorrect / Partially / Mostly / Completely Correct) and ±5% tolerance for volatile numerics.
7. **playground**: No simulator — agents browse live crypto dashboards during eval. Monthly refresh mitigates staleness but requires live network access.
8. **category**: financial transactions — crypto / DeFi analyst agent.

### AccountingBench (Penrose)
1. **description**: 2025 Penrose experiment (accounting.penrose.com) — the highest-signal *long-horizon* agent bench in finance. Closes 12 real months of books for a YC-backed SaaS company with real millions in revenue, measuring how accuracy decays as the agent's own earlier journal entries become part of its own context. Novel failure modes documented: "reward hacking" (fabricating offsetting transactions to make a trial balance tie despite explicit instructions not to). Widely discussed (HN #44637352) and repeatedly cited as the bench where Claude/Grok degrade from ~100% to ~20% over 12 months while o3/Gemini-2.5-Pro fail outright.
2. **github**: **Not open-sourced.** Benchmark and data live on accounting.penrose.com; authors have been asked on HN and not committed to releasing. Worth tracking as a reference result even though it is not directly runnable.
3. **tools**: Agent gets processed transaction records, a SQL executor, a Python executor, and a `create_tool(tool_name, description, python_code, parameters)` primitive for composing its own helpers. Monthly context reset with prior decisions/accruals/comments queryable via tool calls.
4. **data**: One real YC-backed SaaS company, 12 consecutive months of transactions, bank statements, invoices, payroll. Human CPA baseline established.
5. **tasks**: Sequential monthly closes — 12 long-running agent sessions, each with dozens of sub-decisions (accrue vs defer, expense categorisation, reconciliation).
6. **evaluation**: Deterministic ledger-to-statement reconciliation checks per month plus accuracy vs the CPA baseline. Three runs per experiment, best-of-three scored.
7. **playground**: Closed-source SaaS-accounting simulator operated by Penrose. No public harness; reproducibility depends on author cooperation.
8. **category**: financial transactions — accounting / bookkeeping (long-horizon).

### FinanceQA (AfterQuery)
1. **description**: Jan 2025 benchmark (arXiv 2501.18062) pitched as the "hard" analyst-QA complement to FinanceBench. Tests private-equity / hedge-fund / investment-banking style calculations under both complete and *incomplete* information (the latter category is where frontier models collapse to <5% accuracy). Best-in-niche for assessing professional-judgement QA — authors report ~60% failure rate across frontier models on realistic tasks.
2. **github**: https://github.com/AfterQuery/FinanceQA — Apache-2.0. Dataset at https://huggingface.co/datasets/AfterQuery/FinanceQA.
3. **tools**: Agent-as-is. Documents (10-K sections) are supplied in-context; no retrieval or execution tool mandated, though CoT reasoning traces are scored.
4. **data**: 148 rows, single `test` split. Three question types: tactical-basic, tactical-assumption, conceptual. Each item: question + document context + gold answer + gold chain-of-thought.
5. **tasks**: Single-turn QA. Each item requires a precise numeric or narrative answer with professional accounting/valuation conventions.
6. **evaluation**: Deterministic numeric match for tactical items; rubric-based LLM-as-judge (or expert) scoring for conceptual and assumption items where reasoning matters.
7. **playground**: Pure dataset; no simulator.
8. **category**: financial transactions — buy-side / sell-side analyst QA (compliance + valuation math).

### DocFinQA
1. **description**: ACL 2024 short paper (arXiv 2401.06915). Extends FinQA by re-attaching full parent documents (often 150+ pages, ~123k tokens) to each question, so the evaluation probes both long-context reasoning and retrieval rather than just numerical program synthesis. Filters out short-excerpt lucky guesses and is the de facto long-context financial bench.
2. **github**: Data and eval hosted on Kensho's long-doc-QA page (https://benchmarks.kensho.com/benchmarks/long-doc-qa); paper code on arXiv/ACL Anthology. Shares FinQA's CC-style dataset license.
3. **tools**: Agent-as-is by default. Commonly wrapped with a retrieval tool (`search(doc, query)` / `read_page(doc, page)`) so RAG pipelines can be measured alongside long-context models.
4. **data**: 7,437 questions derived from FinQA, each paired with the full 10-K/10-Q parent document. Average context ~123k words vs ~700 in FinQA.
5. **tasks**: Single-turn QA over a very long document; may be multi-step if the agent chains retrieve → compute.
6. **evaluation**: Deterministic numeric/string match against FinQA gold answers plus optional program-execution accuracy (inherited from FinQA's annotated reasoning programs).
7. **playground**: Dataset + evaluation scripts; no simulator.
8. **category**: financial transactions — long-document compliance QA.

### FinTagging
1. **description**: May 2025 benchmark (arXiv 2505.20650, The-FinAI group) — first full-scope, *table-aware* XBRL benchmark. Splits regulatory tagging into FinNI (financial numeric entity identification) and FinCL (concept linking against the full 10k+ US-GAAP 2024 taxonomy). Best-in-niche for regulatory-reporting agents; complementary to XBRL-Agent which introduced the tool-use idea but tested only a few hundred terms.
2. **github**: https://github.com/The-FinAI/FinTagging — data on HF. Runs through the FinBen (VLLM-based) harness; optional Elasticsearch for the retrieval-based concept-linking pipeline.
3. **tools**: Prescribed. FinNI is pure extraction. FinCL is a two-stage agent pipeline: BM25/embedding candidate retrieval against `us-gaap-2024.xsd`, then LLM reranker selects the tag — so it measures taxonomy-grounded tool use, not free-form generation.
4. **data**: FinNI-eval, FinCL-eval, and the larger FinTagging_Original/BIO/Trainset splits drawn from real SEC XBRL submissions; US-GAAP 2024 taxonomy bundled (us_gaap_2024_BM25.jsonl + embedding index).
5. **tasks**: Two subtasks — numeric fact extraction (span-level) and taxonomy concept alignment (~10k-way classification). Single-turn each, but FinCL is effectively a retrieval-augmented classification agent.
6. **evaluation**: Deterministic — entity-level F1 for FinNI, top-k accuracy for FinCL against gold US-GAAP tags.
7. **playground**: Dataset + FinBen harness; no simulator beyond the taxonomy index.
8. **category**: financial transactions — regulatory reporting / XBRL tagging.

### CFA-Bench (CFA Level III)
1. **description**: 2025 benchmark (arXiv 2507.02954) on the hardest tier of the Chartered Financial Analyst exam — the only graduate-level finance-professional bench with full coverage of *essay / constructed-response* items. Mixed MCQ+essay format mirrors the actual sitting. Reasoning-tuned models (o4-mini 79.1%, Gemini 2.5 Flash 77.3%) now pass the composite threshold; variance across essay questions remains high. Good complement to FinanceQA (which tests on-the-job analyst math) and FinEval (which tests Chinese curriculum knowledge).
2. **github**: https://www.cfabenchmark.com/ — paper + dataset description; licensed CFA sample materials so some items are redistributable with restrictions.
3. **tools**: Agent-as-is. No tool scaffolding; prompting strategies (CoT, Self-Discover) are the main degrees of freedom the authors evaluate.
4. **data**: 11 item-set / MCQ blocks + 11 constructed-response essay blocks drawn from released CFA Level III mock materials.
5. **tasks**: Mixed single-turn. MCQs are multi-part items with shared vignette; essays require multi-paragraph reasoning and computation.
6. **evaluation**: Deterministic scoring for MCQs; rubric-based LLM judge + human sanity check for essays, with a revised stricter grader in v2 of the paper.
7. **playground**: Dataset + prompting harness; no simulator.
8. **category**: financial transactions — professional financial reasoning / credentialing.

### FinEval (Chinese financial LLM benchmark)
1. **description**: SUFE AIFLM Lab benchmark (arXiv 2308.09975, v2 NAACL 2025). 26k+ Chinese financial items across academic knowledge, industry practice, financial security, and a dedicated Financial Agent track (616 items) testing tool use and complex reasoning. Best-in-niche Chinese complement — wider and older than BizFinBench and the only major Chinese bench with an agent track. Claude 3.5 Sonnet leads at 72.9 weighted-avg; Qwen-VL-max leads the multimodal slice at 76.3.
2. **github**: https://github.com/SUFE-AIFLM-Lab/FinEval — open source. Dataset publicly downloadable.
3. **tools**: Mixed. Most of the corpus is MCQ / short-answer with agent-as-is scoring. The 616-item Financial Agent split ships simulated tools (search, calculator, database query) and scores multi-step tool-call traces.
4. **data**: 26,000+ questions in four areas — Financial Academic Knowledge (4,661 MCQ across 34 subjects), Financial Industry Knowledge (1,434), Financial Security Knowledge (1,640), Financial Agent (616). Zero-shot and 5-shot CoT splits provided.
5. **tasks**: Mostly single-turn MCQ/short-answer; Financial Agent subset is multi-turn tool-use.
6. **evaluation**: Deterministic accuracy for MCQ and short-answer; trajectory/rubric scoring for the agent slice.
7. **playground**: Dataset + harness; tool simulators included for the agent subset.
8. **category**: financial transactions — multilingual (Chinese) finance knowledge + agent tool use.

### MultiHiertt (+ TAT-QA lineage)
1. **description**: ACL 2022 benchmark (arXiv 2206.01347) — the canonical hybrid-table-and-text numerical-reasoning bench. Unlike TAT-QA (2021, single flat table per doc), MultiHiertt forces multi-step reasoning across *multiple hierarchical* tables in a single 10-K, which is how real financial statements are actually structured. Retained in this round because the newer agentic benches (FinanceBench, Finance Agent, FinAgentBench) implicitly assume models already handle this level of table reasoning; MultiHiertt is the clean unit-test for it. TAT-QA (16,552 items, 2,757 hybrid contexts) listed as a lineage predecessor rather than its own card.
2. **github**: https://github.com/psunlpgroup/MultiHiertt — open-source dataset + baseline code. Sibling TAT-QA at https://github.com/NExTplusplus/TAT-QA.
3. **tools**: Agent-as-is by default. Baseline model (MT2Net) ships as a facts-retrieving + symbolic-reasoning pipeline; benchmark itself is tool-agnostic, so it is trivially wrapped with a code-execution or table-QA tool.
4. **data**: 10,440 QA pairs over 2,500+ 10-K documents with multiple hierarchical tables; fine-grained supporting-facts + reasoning-program annotations per item.
5. **tasks**: Single-turn numerical-reasoning QA; each answer is either a numeric value or a short span, traced by a program.
6. **evaluation**: Deterministic — exact match on answer + program-execution accuracy (numerical tolerance). Human-expert ceiling reported; baselines lag significantly.
7. **playground**: Dataset + baseline; no simulator.
8. **category**: financial transactions — table + text compliance reasoning (unit-test layer below agentic benches).

## Notes on exclusions
- **ToolBench / ToolLLM** (OpenBMB): has a finance slice but relies on live RapidAPI endpoints that rot; StableToolBench (THUNLP-MT) addresses stability but still not finance-specialised — skipped as not best-in-niche for this anthology.
- **FLUE**: pure NLP (sentiment, NER, QA); excluded per filter.
- **API-Bank**: finance APIs exist but benchmark is general-purpose tool-use — BFCL dominates.
- **TradingAgents / FinMem / StockAgent / FinRobot / FinAgent / FinCon / CryptoTrade**: frameworks more than benchmarks; their evaluation protocols are subsumed by StockBench / LiveTradeBench / InvestorBench.
- **BizFinBench** (HiThink-Research): Chinese-language business finance; FinEval (Round 2) covers Chinese coverage more broadly and has an agent track, so BizFinBench is redundant here.
- **FinQA / ConvFinQA**: subsumed by DocFinQA (same questions, longer documents) and by FinBen (which already re-hosts them in the harness).
- **FiNER / FNXL**: pure NLP XBRL tagging without the tool-use agent layer; FinTagging (Round 2) covers the same taxonomy with an agent pipeline.
- **FinBench (loan default prediction, NeurIPS 2024 D&B by Xuanwen-Huang et al.)**: tabular ML benchmark for classical classifiers on loan data; not an LLM/agent bench.
- **XFinBench**: graduate-level multi-modal financial problems; overlaps with CFA-Bench's advanced-reasoning slice and less widely adopted.
- **XBRL-Agent** (Han et al., ICAIF 2024): the tool-use-for-XBRL idea, but ~50 domain + 50 numeric eval queries make it too small; FinTagging (Round 2) is the scaled-up version.
- **Finance-classification fraud benches (European CC / PaySim + LLM wrappers)**: tabular fraud datasets dressed with LLM encoders — not tool-use agent benchmarks in the sense this anthology tracks.
- **Smart-contract audit benches (LLMBugScanner, LLM-SmartAudit, iAudit/TrustLLM, LLMSmartSec)**: solid research but evaluation sets are small (~108 CVE-tagged contracts), licenses mixed, and none yet function as a reusable harness; CryptoBench (Round 2) covers the DeFi-agent side, and smart-contract-specific eval is better tracked in a security-bench anthology.
- **FinDER**: strong RAG-in-finance bench but its value is subsumed by FinAgentBench (Round 2) + Finance Agent Benchmark (Round 2), which both use SEC filings and add agent scaffolding.
- **FinMTEB**: embeddings-only (classification / retrieval / STS); excluded as a pure representation bench rather than a tool-use / task bench.
- **Finance Arena (financearena.ai)**: live LLM-vs-LLM finance leaderboard without a fixed task set; closer to LMArena than a reproducible bench.
