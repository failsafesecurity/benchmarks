# Browser Use Benchmarks

### WebArena
1. **description**: A standalone, fully reproducible web environment for autonomous agents. Authors pitch it as "a realistic and reproducible web environment" that matches the diversity and functional complexity of real sites (shopping, forums, dev tools, CMS, maps). Established the modern self-hosted pattern that most later benchmarks copy.
2. **github**: https://github.com/web-arena-x/webarena — fully open source, Apache-2.0. Actively runnable; also wrapped inside BrowserGym and AgentLab.
3. **tools**: Agent-as-is (agent receives obs and returns an action string). The repo ships a Playwright-based browser harness; the default action space is keyboard/mouse over the accessibility tree. Observation/action mode is configurable.
4. **data**: Docker images for 6 stack sites — OneStopShop (Magento), Magento Admin CMS, Reddit (Postmill), GitLab, OpenStreetMap+routing, plus a Wikipedia snapshot. All seeded with fixed DB dumps so every run is deterministic.
5. **tasks**: 812 natural-language tasks across the 6 sites, covering info-seeking, site navigation, and content/config editing. Each task JSON carries start URL, intent, and a programmatic evaluator.
6. **evaluation**: Deterministic. Per-task evaluators check (a) URL match, (b) programmatic DOM/DB state (e.g., "is order #123 cancelled?"), or (c) string exact/must-include on the agent's final answer. No LLM judge by default.
7. **playground**: Self-hosted Docker sites (the canonical reproducible web-agent playground). Comes with a pre-built AWS AMI for one-click spin-up.
8. **category**: browser use — multi-site workflow (e-commerce + enterprise SaaS + forum + dev tools + maps).

### VisualWebArena
1. **description**: Visually-grounded extension of WebArena. Authors describe it as "a benchmark for evaluating multimodal autonomous agents on realistic visual web tasks" — every task requires understanding images on the page (product photos, thumbnails, classifieds listings).
2. **github**: https://github.com/web-arena-x/visualwebarena — fully open source, MIT.
3. **tools**: Agent-as-is. Adds Set-of-Mark (SoM) screenshot annotation as a first-class obs mode alongside the a11y tree; Playwright driver inherited from WebArena.
4. **data**: Three new Dockerized sites — Classifieds (Craigslist-like), a visually-rich Shopping site (OneStopMarket with product imagery), and a Reddit fork with image posts. Optionally reuses WebArena's CMS/GitLab/Map.
5. **tasks**: 910 multimodal tasks across Classifieds, Shopping, and Reddit. Each task has images embedded in the instruction or the page, and a programmatic evaluator.
6. **evaluation**: Deterministic, same scheme as WebArena (URL / DOM-state / answer match). Some tasks additionally check posted image content via URL match or alt-text.
7. **playground**: Self-hosted Docker sites; AWS AMI available.
8. **category**: browser use — e-commerce + classifieds + forum, multimodal.

### WorkArena / WorkArena++
1. **description**: ServiceNow-authored benchmark for enterprise SaaS work. Tests whether agents can do the mundane knowledge-worker tasks a ServiceNow user performs — filling forms, filtering lists, creating incidents, navigating dashboards. Authors frame it as "common knowledge work tasks."
2. **github**: https://github.com/ServiceNow/WorkArena — fully open source, Apache-2.0. Requires a ServiceNow Personal Developer Instance (free but gated via HuggingFace form).
3. **tools**: Agent-as-is, running through BrowserGym (Playwright + a11y tree + screenshot). Actions are standard browser primitives (click, fill, select, goto).
4. **data**: No local fixture — the "data" is a live ServiceNow PDI instance provisioned per user. WorkArena scripts seed and reset state inside the instance between tasks.
5. **tasks**: WorkArena-L1: 33 atomic task templates with ~19,912 parameterized instances. WorkArena++: 682 composite tasks assembled from atomics (planning, memory, long-horizon).
6. **evaluation**: Deterministic. Each task has a Python validator that inspects ServiceNow DB state via REST API and returns reward 0/1 plus a reason string. Oracle "cheat" functions are provided as reference trajectories.
7. **playground**: Live but sandboxed (your own PDI). Not Docker — ServiceNow hosts the instance. First-class BrowserGym integration.
8. **category**: browser use — enterprise SaaS (ITSM / forms / workflow).

### WebShop
1. **description**: Princeton's e-commerce simulator. 1.18M real Amazon products scraped into a locally-served shopping site; agents get a natural-language purchase intent and must search, filter, and buy a matching item. Authors call it "a simulated e-commerce website environment with real-world products and crowd-sourced text instructions."
2. **github**: https://github.com/princeton-nlp/WebShop — fully open source, MIT. Still the canonical "single-site deterministic web-agent task."
3. **tools**: Agent-as-is. Two obs modes: `html` (full rendered markup) and `simple` (stripped text). Optional ResNet image features. Action space is a discrete set: `search[query]`, `click[element]`.
4. **data**: Local Flask site served via `./run_dev.sh` at :3000; ships the full product corpus (1.18M) plus a 1k subset for fast iteration. Includes 1,765 human trajectories for imitation learning.
5. **tasks**: 12,087 crowd-sourced instructions split into train/dev/test. Every task comes with a gold product + gold attributes.
6. **evaluation**: Deterministic reward in [0,1] based on attribute overlap between the purchased product and the gold product (type match + attribute match + price constraint + option match). Exact-match task-success also reported.
7. **playground**: Self-hosted (single Flask app, no Docker needed but Docker-friendly). Integrated into BrowserGym as a legacy environment.
8. **category**: browser use — e-commerce (single site, long-tail product search).

### Mind2Web
1. **description**: Ohio State's "first dataset for developing and evaluating generalist agents for the web." Pure trajectory/snapshot corpus over 137 real websites — the goal is to learn an action predictor over raw HTML that generalizes to unseen domains.
2. **github**: https://github.com/OSU-NLP-Group/Mind2Web — fully open source (code MIT, data CC-BY-4.0). Also supported in BrowserGym (static mode).
3. **tools**: Agent-as-is on the offline split; the original pipeline is two-stage (DeBERTa element ranker + T5 action generator). No interactive browser needed for the canonical evaluation.
4. **data**: >2,000 tasks, each with a full HTML + DOM snapshot per step and a human-authored action sequence. Multimodal-Mind2Web adds aligned screenshots.
5. **tasks**: ~2,350 tasks across 137 sites in 31 domains (shopping, travel, service, entertainment, info). Three generalization splits: cross-task, cross-website, cross-domain.
6. **evaluation**: Deterministic. Element accuracy (did the agent pick the right DOM node?), operation F1 (CLICK/TYPE/SELECT + value), and full step/task success against the reference trajectory. Oracle-trajectory style.
7. **playground**: Pure trajectory dataset — no playground. Use Online-Mind2Web below if you want live browsing.
8. **category**: browser use — multi-site workflow (general web navigation, offline).

### Online-Mind2Web
1. **description**: 2025 refresh that takes Mind2Web onto the live web. Authors argue offline benchmarks overstate progress and re-author 300 tasks across 136 popular sites (Amazon, Reddit, Airbnb, GitHub, etc.) so agents must cope with real HTML, real CAPTCHAs, real site drift.
2. **github**: https://github.com/OSU-NLP-Group/Online-Mind2Web — fully open source, code MIT / data CC-BY-4.0.
3. **tools**: Agent-as-is. Reference pipeline uses Playwright + screenshot observations; tasks are framework-agnostic and shipped as a plain JSONL of instructions + start URLs.
4. **data**: 300 tasks + periodic task-set refreshes (to fight site-drift). Authors maintain a living leaderboard.
5. **tasks**: 300 diverse tasks over 136 live sites; single- and multi-page.
6. **evaluation**: **LLM-as-judge** — introduces *WebJudge*, which selects critical screenshots from the agent trajectory and scores outcome against the instruction using GPT-4-class judges. Authors recommend o4-mini for best alignment with human raters.
7. **playground**: Live web (not reproducible by construction). Include with the caveat that runs aren't byte-for-byte replayable.
8. **category**: browser use — multi-site workflow, live web.

### WebLINX
1. **description**: McGill's *conversational* web navigation benchmark. Human users and annotators work together via chat to complete real web tasks; agents must turn utterances into browser actions, mid-dialogue. Authors pitch it as "real-world website navigation with multi-turn dialogue."
2. **github**: https://github.com/McGill-NLP/weblinx — fully open source, Apache-2.0. Dataset on HuggingFace.
3. **tools**: Agent-as-is. Observations are DOM + screenshot; the paper's reference model (`DMR` + LLM) is HTML-first. Action space covers click, type, submit, scroll, say, and tab ops.
4. **data**: ~2,337 expert demonstrations of chat-driven browsing across ~155 real sites. Each demo includes utterances, DOM snapshots, screenshots, and video.
5. **tasks**: Demos divided across four generalization splits (seen/unseen sites, geography, category). Benchmark is "turn-level" — predict the next action given history.
6. **evaluation**: Deterministic against the oracle trajectory — action-type accuracy, element IoU/IoA, text F1 for typed content, and overall turn success. Also supports running live through BrowserGym (static task adapter).
7. **playground**: Trajectory dataset primarily; BrowserGym supplies a "static" replay env. No self-hosted live sites.
8. **category**: browser use — conversational / multi-site workflow.

### BrowserGym
1. **description**: ServiceNow's unification layer — a Gym-style environment that wraps multiple web benchmarks behind one API. Not a benchmark itself, but if you're building harness integrations it's the cheapest way to cover a dozen suites at once. Paired with AgentLab (parallel runner + leaderboard).
2. **github**: https://github.com/ServiceNow/BrowserGym — fully open source, Apache-2.0.
3. **tools**: **Prescribes tool scaffolding**. Playwright under the hood. Unified observation bundle: HTML, pruned accessibility tree, screenshot (raw + SoM-annotated), DOM object, focused element, chat messages. Unified action space with a bid-based element referencing scheme.
4. **data**: Ships task adapters; data comes from the underlying benchmark (WebArena fixtures, WorkArena PDI, MiniWoB HTML, etc.).
5. **tasks**: 9 first-party benchmarks integrated — MiniWoB (~100+), WebArena (812), WebArena-Verified, VisualWebArena (910), WorkArena (L1+L2+L3, 10k+), AssistantBench (214), WebLINX-static, OpenApps, TimeWarp.
6. **evaluation**: Delegates to the wrapped benchmark (deterministic for most; LLM-judge where the underlying suite uses one).
7. **playground**: Mixed — Docker for WebArena/VWA, ServiceNow PDI for WorkArena, local page for MiniWoB, live web for AssistantBench.
8. **category**: browser use — infrastructure / multi-benchmark aggregator.

### AssistantBench
1. **description**: "Can Web Agents Solve Realistic and Time-Consuming Tasks?" — 214 hand-authored tasks that mimic long, multi-site research a human would actually delegate (real-estate comps, business lookups, travel planning, academic fact-finding). Authors highlight that SOTA agents score near-zero.
2. **github**: https://github.com/oriyor/assistantbench — fully open source. Dataset on HuggingFace with hidden test answers (submission portal).
3. **tools**: Agent-as-is. Tasks are framework-agnostic prompts; the paper evaluates the SeePlanAct agent but any browser-equipped agent works. Integrated in BrowserGym.
4. **data**: 214 tasks averaging 5+ pages each, drawn from 258 distinct sites. Gold answers + gold reasoning paths provided on the dev set.
5. **tasks**: 33 dev / 181 test. Each instance has a short natural-language question and a structured gold answer (number, string, list, or JSON).
6. **evaluation**: Deterministic answer-match with tolerant scoring — numeric closeness, set-overlap F1 for lists, string match for atoms. No LLM judge required, though authors also report a judge-based variant.
7. **playground**: Live web. Reproducibility is approximate (site drift), but the answer-matching scoring insulates against small wording changes.
8. **category**: browser use — search / research / multi-site workflow.

### TheAgentCompany
1. **description**: CMU/All Hands AI's simulated software-company benchmark. Agents act as SWE/PM/data-scientist/HR staff inside a fake company, with genuine self-hosted services standing in for the real SaaS stack. Authors position it as a test of "consequential real-world professional work."
2. **github**: https://github.com/TheAgentCompany/TheAgentCompany — fully open source, MIT.
3. **tools**: Prescribes a browser + terminal scaffold. Default runner is OpenHands (Playwright + bash + file tools). Observation is screenshot + DOM; agent drives a real Chromium.
4. **data**: Four Dockerized services seeded with company data — GitLab (code + issues + MRs), Plane (project mgmt), ownCloud (files), RocketChat (chat). Plus simulated coworkers powered by an LLM.
5. **tasks**: 175 task images covering software engineering, product management, data analysis, HR, finance, and admin. Many tasks require hopping between services.
6. **evaluation**: Hybrid. Primary is deterministic result-based checks (API/DB state in GitLab/Plane/ownCloud). Secondary is "subcheckpoint" partial credit, sometimes scored by LLM evaluators for open-ended artifacts.
7. **playground**: Self-hosted Docker services (fully reproducible). One-command `docker compose` spin-up.
8. **category**: browser use — enterprise SaaS / multi-site workflow (with coding subtasks).

### BEARCUBS
1. **description**: Maryland's "small but mighty" benchmark for computer-using web agents. 111 information-seeking questions that deliberately require live-web access and multimodal interaction (watching a video, navigating a 3D map view) — things that can't be short-circuited with a text cache. Humans hit 84.7%; Operator hit 24.3%.
2. **github**: Project page: https://bear-cubs.github.io/ — dataset, eval script, and example trajectories are public. Paper: arXiv 2503.07919.
3. **tools**: Agent-as-is. Designed to be run by any computer-use agent (Operator, Claude Computer Use, browser-use, etc.); no prescribed scaffold.
4. **data**: 111 curated QA items with gold answers + metadata (text-only vs multimodal, domain tags).
5. **tasks**: 56 text-heavy + 55 multimodal (video, 3D, interactive maps, audio). Short-answer format.
6. **evaluation**: Deterministic answer match with automated grader; human-verified.
7. **playground**: Live web. No Docker snapshot — live-web dependency is intentional (the point is brittleness).
8. **category**: browser use — search / information-seeking, multimodal.

### MMInA
1. **description**: "Benchmarking Multihop Multimodal Internet Agents" (ACL 2025 Findings). Focuses on *compositional* tasks that chain info across multiple live sites (e.g., compare a product on Wikipedia then buy on Amazon). Agents must extract multimodal content from each hop.
2. **github**: https://github.com/shulin16/MMInA — fully open source.
3. **tools**: Agent-as-is; reference pipeline uses GPT-4V with screenshot+HTML obs. Playwright-based driver.
4. **data**: 1,050 human-written tasks spanning shopping, travel, general-info domains; each task annotated with the expected hop chain.
5. **tasks**: 1,050 tasks, each requiring 2+ hops across distinct websites. Heavy emphasis on cross-site reasoning.
6. **evaluation**: Deterministic task-success plus per-hop success metrics; answer match against gold. Human upper-bound 96.3%, GPT-4V 21.8%.
7. **playground**: Live web (with site-drift caveat).
8. **category**: browser use — multi-site workflow, multimodal.

### MiniWoB++
1. **description**: The canonical synthetic web benchmark. Extension of OpenAI's original MiniWoB (World of Bits), introduced in "Reinforcement Learning on Web Interfaces using Workflow-Guided Exploration." Single-page HTML scenarios with tight, well-scoped tasks (click button, drag-drop, fill date picker, compose email subset). Still a default smoke-test for any new web-agent paper despite being "toy" next to WebArena.
2. **github**: https://github.com/Farama-Foundation/miniwob-plusplus — fully open source, MIT. Docs: https://miniwob.farama.org/. Actively maintained by Farama Foundation.
3. **tools**: Agent-as-is. Tiny observation space (raw DOM of one page) + programmatic actions via JS bindings. First-class Gym/Gymnasium API; also wrapped in BrowserGym. Supports text obs and screenshot obs.
4. **data**: 125+ pre-built HTML environments served as a static site (no backend). Includes stochastic layout variation and seeded RNG for deterministic rollouts.
5. **tasks**: 100+ tasks with programmatic instruction + deterministic reward. Classic suite covers "click-link", "drag-items", "book-flight", "use-autocomplete", "email-inbox-nl", etc.
6. **evaluation**: Deterministic. Every task ships a reward function returning a real-valued score in [-1, 1] (typically binary success). Short episodes (seconds per rollout) — ideal for RL and for cheap regression tests.
7. **playground**: Self-hosted static HTML (serve with any web server). Fully offline, no Docker needed. Integrated in BrowserGym for unified obs/action.
8. **category**: browser use — single-page synthetic widgets (RL-friendly).

### WebVoyager
1. **description**: "Building an End-to-End Web Agent with Large Multimodal Models" (ACL 2024). The reference live-web benchmark for GPT-4V-era agents — 643 tasks scraped onto 15 well-known sites (Amazon, eBay, Google Maps, Booking.com, ArXiv, GitHub, Wikipedia, Reddit, Twitter/X, etc.). Pairs the benchmark with a GPT-4V *judge* for automatic scoring, which has become the most-copied eval recipe in later live-web benches.
2. **github**: https://github.com/MinorJerry/WebVoyager — fully open source. Data on the repo.
3. **tools**: Agent-as-is. Reference agent drives Chromium via Selenium with Set-of-Mark screenshot annotation; action space is `Click [n] / Type [n; text] / Scroll / Wait / GoBack / Google / Answer`. Widely reimplemented in browser-use, Steel, BrowserGym.
4. **data**: 643 hand-authored tasks across 15 live websites; each task has a natural-language intent + a reference answer snapshot for judging.
5. **tasks**: Mix of info-seeking (~"find the cheapest flight from X to Y") and action ("add N to cart"). Single-site per task.
6. **evaluation**: **LLM-as-judge** — GPT-4V consumes the final screenshot sequence + agent answer and emits success/failure. Authors report 85.3% agreement with human raters. No programmatic evaluators; reproducibility is approximate by construction.
7. **playground**: Live web. No self-hosted snapshot — results drift as sites change. Referenced sites are big/stable enough that the benchmark remains runnable.
8. **category**: browser use — multi-site workflow, live web, multimodal.

### WebCanvas / Mind2Web-Live
1. **description**: iMean AI's online evaluation framework. Argues that final-state scoring breaks under site drift, and instead scores *key intermediate nodes* (URL prefixes, DOM predicates) along the agent's path. Ships Mind2Web-Live — a 542-task refresh of Mind2Web re-authored for the live web with ~2,439 key-node annotations.
2. **github**: https://github.com/iMeanAI/WebCanvas — fully open source. Paper: arXiv 2406.12373 (ICML 2024 AgenticAI workshop).
3. **tools**: Agent-as-is. Reference pipeline uses Playwright + HTML/DOM + optional screenshots; plug-in architecture for custom agents.
4. **data**: Mind2Web-Live — 542 tasks on live sites, each with a list of *key nodes* (intermediate URL/DOM conditions) and a gold final state. Annotation tools included so community can keep the dataset fresh.
5. **tasks**: Task-level instructions identical in style to Mind2Web; every task decomposed into ordered milestones for partial-credit scoring.
6. **evaluation**: Deterministic *per-node* — at each step, the framework checks whether the agent has satisfied the next key-node predicate. Metrics: task success rate and task completion rate (fraction of nodes hit). Currently best agent ~23% SR / ~49% completion.
7. **playground**: Live web, but the key-node scoring makes it more robust to drift than pure final-state checks. No Docker — uses public sites directly.
8. **category**: browser use — multi-site workflow, live web, milestone-scored.

### WebGames
1. **description**: Convergence AI's "can your agent do things that are trivial for humans?" suite. 50+ (now ~150) small interactive challenges — dragging, password-typing, CAPTCHA-style puzzles, chart-reading, mini-games — each with a verifiable success signal. Deliberately exposes holes that realistic-site benchmarks hide.
2. **github**: https://github.com/convergence-ai/webgames — fully open source. Live site: https://webgames.convergence.ai/. Paper: arXiv 2502.18356 (Feb 2025).
3. **tools**: Agent-as-is. Hermetic client-side JS — no backend to configure. Agents interact via any browser driver (Playwright, Computer Use, etc.). JSONL task specs integrate cleanly with Inspect AI.
4. **data**: Self-contained static site; every challenge is a standalone HTML page with embedded success-check JS. No external APIs, no cookies, no drift.
5. **tasks**: Five progressive categories — fundamental browser actions, advanced input (drag, keyboard, scroll), cognitive (chart reading, memorization), workflow automation, interactive entertainment (mini-games).
6. **evaluation**: Deterministic — each challenge emits a verifiable ground-truth token on success. Best AI (Claude Computer Use / GPT-4o class) hit 43.1% vs. 95.7% human baseline at launch.
7. **playground**: Fully self-hosted / hermetic. The cleanest reproducibility story among live-browser-style benches.
8. **category**: browser use — synthetic interactive challenges, computer-use oriented.

### BrowseComp
1. **description**: OpenAI's "deep-research" benchmark (April 2025). 1,266 fact-seeking questions hand-authored by human trainers to be *short-answer but hard-to-find* — the trainer personally verifies that GPT-4o + browsing, o1, and early Deep Research all fail. Analogous in spirit to competitive-programming benches for coding agents: narrow-success, persistence-testing.
2. **github**: https://github.com/openai/simple-evals (eval harness) + dataset on OpenAI's CDN. Paper: arXiv 2504.12516. Fully open source under OpenAI's eval license; replicated in Inspect Evals and Kaggle leaderboards.
3. **tools**: Agent-as-is. Framework-agnostic — any browsing agent (Deep Research, browser-use, SearchR1, etc.) can be evaluated. No prescribed scaffold.
4. **data**: 1,266 Q&A items. Each question has a single, verifiable, time-stable short answer. Released fully (including answers) — no hidden test set.
5. **tasks**: Single-turn: question in, short factual answer out. Persistence-heavy (info typically buried on page 3+ of search results or needs cross-referencing 5+ sources).
6. **evaluation**: Deterministic answer-match with an LLM grader for surface-form normalization (numeric equivalence, alias matching). Near-zero for GPT-4o + browsing; ~51% for OpenAI Deep Research at launch.
7. **playground**: Live web. Reproducibility good in practice because answers are time-stable, but actual browsing paths drift.
8. **category**: browser use — search / deep research, single-answer QA.

### Mind2Web-2
1. **description**: OSU-NLP's sequel (NeurIPS 2025 D&B). Shifts focus from "predict next click" to *agentic search* — long-horizon information-synthesis tasks (e.g., "draft a comparison table of five EV SUVs under $60k including their current incentives"). Over 1,000 hours of expert annotation; explicitly tests sourcing/citation quality.
2. **github**: https://github.com/OSU-NLP-Group/Mind2Web-2 — fully open source. Paper: arXiv 2506.21506.
3. **tools**: Agent-as-is. Framework-agnostic — evaluates deep-research systems (Deep Research, Perplexity, GPT-Researcher, custom browser agents) from their final report output.
4. **data**: 130 realistic long-horizon tasks with tree-structured rubrics (~dozens of rubric nodes per task), covering research, shopping comparison, travel planning, technical due-diligence.
5. **tasks**: Each task requires real-time browsing + synthesis across many sources; answers are multi-paragraph with citation requirements.
6. **evaluation**: **Agent-as-a-Judge** — a task-specific judge agent walks the tree-structured rubric and checks answer correctness *plus* source attribution (every claim traced to a live URL). Hybrid deterministic (rubric match) + LLM-judge (semantic match). Best system (Deep Research) reaches ~50-70% of human performance.
7. **playground**: Live web. Time-varying answers are a feature — the judge recomputes rubrics per evaluation run.
8. **category**: browser use — deep research / agentic search, long-horizon synthesis.

### WebWalkerQA
1. **description**: Alibaba's "can agents walk a website's subpage tree?" benchmark (ACL 2025). Targets the failure mode where RAG returns surface-level hits but misses answers 3+ clicks deep. Pairs a specific Explorer/Critic reference agent (WebWalker) with the QA set, but the benchmark is agent-agnostic.
2. **github**: https://github.com/SevenPlusPlus/WebWalker (redirects to Alibaba-NLP/WebAgent) — fully open source. Paper: arXiv 2501.07572.
3. **tools**: Agent-as-is. Reference pipeline uses a headless-browser crawler + LLM reasoner; observation is rendered HTML per page. Task spec is seed URL + natural-language question.
4. **data**: 1,373 web pages harvested across 4 domains — education, organizations, academic conferences, games — annotated into 680 QA pairs. Each answer is only reachable by traversing 2+ subpages from the root.
5. **tasks**: 680 QA items split by domain; each task has a canonical seed URL and gold short-form answer.
6. **evaluation**: Deterministic answer-match (exact + normalized) plus per-hop traversal accuracy. Best reported ~37.5% (GPT-4o) — strong headroom.
7. **playground**: Live web, but seeded to specific site roots that are relatively static (edu/conference pages). Reproducibility is better than general live-web because target sites drift slowly.
8. **category**: browser use — search / deep traversal, QA.

### WebBench (Halluminate)
1. **description**: Halluminate's production-flavored benchmark (mid-2025). Argues prior benches (WebVoyager, WebArena) are too narrow in site count or too synthetic; builds a 5,750-task set across 452 real sites from the global top-1k by traffic, half of which (~2,454) are open-sourced. First-class split between READ (info-seek) and WRITE (fill forms, auth, downloads) tasks.
2. **github**: https://github.com/Halluminate/WebBench — open-source subset; full leaderboard at https://webbench.ai/.
3. **tools**: Agent-as-is. Framework-agnostic — leaderboard accepts any browsing agent (Anthropic CUA, Skyvern, browser-use, Operator, etc.).
4. **data**: 2,454 open-source tasks (of 5,750 total), each pinned to a specific real website with a gold answer or gold post-state. Sites sampled from top-1k global traffic.
5. **tasks**: READ (navigate + extract) and WRITE (input, auth, 2FA, file ops). WRITE tasks explicitly include solving 2FA challenges — unusual.
6. **evaluation**: Verified programmatic scoring where feasible; LLM-judge fallback for open-ended outputs. All leaderboard entries are "Verified" (human-audited). As of 2025-Q2 Claude Sonnet 3.7 CUA led at ~66%.
7. **playground**: Live web. Not reproducible byte-for-byte (452 real sites drift constantly); Halluminate's audit pipeline is the reproducibility story.
8. **category**: browser use — multi-site workflow, live web, production-focused.

### VisualWebBench
1. **description**: CMU/Ohio State's "how well do MLLMs actually *understand* web pages?" benchmark (ICLR 2025). Not a task-completion suite — it isolates seven sub-skills an agent needs *before* it can navigate (captioning, QA, OCR, grounding, action prediction). Useful as a fast diagnostic for a new VLM before plugging it into WebArena.
2. **github**: https://github.com/VisualWebBench/VisualWebBench — fully open source. Project page: https://visualwebbench.github.io/. Paper: arXiv 2404.05955.
3. **tools**: Prescribes evaluation protocol per sub-task (multiple-choice grounding, open-ended captioning, etc.). Models evaluated via direct image+prompt inference — no browser driver needed.
4. **data**: 1.5k human-curated instances from 139 real websites across 87 sub-domains; screenshots + structured labels.
5. **tasks**: Seven tasks — webpage captioning, webpage QA, heading OCR, element OCR, element grounding, action prediction, action grounding. Organized by three levels (website / element / action).
6. **evaluation**: Deterministic — accuracy / F1 / IoU depending on sub-task. No LLM judge. GPT-4V ~64.6%, Claude Sonnet ~65.8%, open-source MLLMs trail by 10-20 pts.
7. **playground**: Pure offline dataset — no browser loop. Use as a prerequisite check, not a replacement for WebArena-style eval.
8. **category**: browser use — visual web understanding / grounding (no task execution).

### WebArena-Verified
1. **description**: ServiceNow's cleaned-up fork of WebArena (2025). Manual audit of every task + evaluator in the original 812, plus a 258-task "hard subset" for fast iteration. Introduces *offline trace replay* — runs can be scored against captured network traces without re-spinning the Docker stack, which makes CI integration finally cheap.
2. **github**: https://github.com/ServiceNow/webarena-verified — fully open source, Apache-2.0. PyPI: `webarena-verified`. Docs: https://servicenow.github.io/webarena-verified/.
3. **tools**: Agent-as-is. Inherits WebArena's Playwright harness through BrowserGym; adds trace-replay runner for offline eval.
4. **data**: Same 6 Dockerized sites as WebArena (shopping, admin, Reddit, GitLab, maps, Wikipedia), but tasks and evaluators have been manually re-reviewed. Hard subset: 258 tasks (68% faster end-to-end).
5. **tasks**: Full 812 verified tasks + 258-task hard subset. Each task has audited gold answers / gold post-states and type-aware normalized evaluators.
6. **evaluation**: Deterministic — audited URL/DOM/DB predicates with type-aware structural comparison. No LLM judge. Supports both live evaluation (against the Docker stack) and offline trace replay (no stack needed).
7. **playground**: Self-hosted Docker (for live mode) or fully offline (for trace-replay mode). Cleanest reproducibility story of the WebArena family today.
8. **category**: browser use — multi-site workflow, e-commerce + SaaS + forum + dev + maps.

### OSWorld (browser slice)
1. **description**: XLang Lab's end-to-end computer-use benchmark (NeurIPS 2024 D&B). Superset of browser benches — agents drive a full Ubuntu/Windows/macOS VM via screenshot + pyautogui. Includes a sizeable Chrome slice (~40 tasks) that overlaps heavily with browser benchmarks, plus cross-app workflows that genuinely need the browser as *one* of several tools.
2. **github**: https://github.com/xlang-ai/OSWorld — fully open source, Apache-2.0. Paper: arXiv 2404.07972. 2025 refresh: OSWorld-Verified (XLang).
3. **tools**: **Prescribes computer-use scaffold**. Observation = screenshot + optional a11y tree; action = pyautogui + keyboard commands. Ships VM images (VMware/VirtualBox/AWS).
4. **data**: VM snapshots (Ubuntu + preloaded apps) with per-task reset scripts. Chrome preloaded with fixture bookmarks/cookies for the browser tasks.
5. **tasks**: 369 total; ~40 Chrome-specific tasks (web automation, cookie/history manipulation, form-filling, download management) + cross-app tasks that use Chrome as one hop (e.g., research in browser, paste into LibreOffice).
6. **evaluation**: Deterministic. Per-task Python scripts inspect VM state — file contents, app settings, cookies, clipboard, browser history — to verify functional completion. No LLM judge.
7. **playground**: Self-hosted VM (multiple providers). OSWorld-Verified (2025) fixes 300+ task/infra issues and is the recommended runnable.
8. **category**: computer use — browser slice is one modality of a broader OS-level bench.

### AndroidWorld
1. **description**: Google DeepMind's mobile counterpart to WebArena (ICLR 2025). **Not a browser benchmark** — agents drive a live Android emulator via screenshot + touch / keyboard. Included here because many research pipelines treat it as the sibling bench to WebArena and because mobile Chrome is one of its 20 apps. Note mobile vs. browser distinction when comparing numbers.
2. **github**: https://github.com/google-research/android_world — fully open source, Apache-2.0. Paper: arXiv 2405.14573.
3. **tools**: Prescribes a mobile computer-use scaffold. Observation = Android a11y tree + screenshot; action = tap / swipe / type / key events. Runs against a standard Android Studio emulator.
4. **data**: 116 hand-crafted task templates across 20 Android apps (Messages, Clock, Settings, Chrome, Maps, Markor, Camera, etc.), each dynamically parameterized — "millions of unique task instances" via randomized slot values.
5. **tasks**: 116 task classes × parameterization. Coverage: app-internal workflows, cross-app flows, settings manipulation, content creation. Mobile Chrome tasks overlap browser-bench themes (navigate, fill form, bookmark).
6. **evaluation**: Deterministic — ground-truth state inspection via Android Debug Bridge (adb). No LLM judge, no screenshot comparison. Best agent at launch ~30.6%.
7. **playground**: Self-hosted Android emulator + per-task init/teardown scripts. Fully reproducible; network-independent for most tasks.
8. **category**: **mobile app use** (not browser) — included as a cross-reference. Browser tasks exist as a minority slice via mobile Chrome.
