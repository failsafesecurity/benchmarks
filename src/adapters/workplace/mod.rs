pub mod assertions;
pub mod interceptor;
pub mod judge;
pub mod server;
pub mod state;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::error::BenchError;
use crate::suite::{BenchScore, BenchSuite, BenchTask, ConversationTurn, TaskSubmission};

use self::assertions::{evaluate_expected_state, WorkplaceAssertions};
use self::judge::{DimensionResult, ScoreBreakdown, ScoringConfig, ScoringHints};
use self::server::WorkplaceServer;
use self::state::{CompanyState, CompanyStateOverrides, ExpectedState, StateInjection};

/// Real service URLs used in skills. The HttpInterceptor matches these
/// and routes to the mock server — no actual HTTP calls leave the process.

// ---------------------------------------------------------------------------
// Scenario format
// ---------------------------------------------------------------------------

/// A workplace simulation scenario loaded from a JSON file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkplaceScenario {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub persona: String,
    #[serde(default)]
    pub setup: WorkplaceSetup,
    #[serde(default)]
    pub scoring: ScoringConfig,
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
    #[serde(default = "default_max_tool_iterations")]
    pub max_tool_iterations: usize,
    pub turns: Vec<WorkplaceTurn>,
    #[serde(default)]
    pub expected_state: Option<ExpectedState>,
}

fn default_timeout_secs() -> u64 {
    180
}

fn default_max_tool_iterations() -> usize {
    25
}

/// Setup for a workplace scenario.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkplaceSetup {
    /// Reference to a company template file (name without extension).
    #[serde(default)]
    pub template: Option<String>,
    /// Identity files injected into agent workspace.
    #[serde(default)]
    pub identity: HashMap<String, String>,
    /// Which skill files to load (e.g., ["slack", "email", "calendar"]).
    #[serde(default)]
    pub skills: Vec<String>,
    /// Scenario-specific state overrides, merged with template.
    #[serde(default)]
    pub company_state_overrides: Option<CompanyStateOverrides>,
}

/// A single turn in a workplace scenario.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkplaceTurn {
    pub user_input: String,
    #[serde(default)]
    pub user_input_file: Option<String>,
    #[serde(default)]
    pub assertions: WorkplaceAssertions,
    #[serde(default)]
    pub scoring_hints: ScoringHints,
    /// State to inject before this turn (e.g., new Slack messages arriving).
    #[serde(default)]
    pub state_injection: Option<StateInjection>,
}

// ---------------------------------------------------------------------------
// WorkplaceSuite
// ---------------------------------------------------------------------------

/// Workplace simulation benchmark suite.
///
/// Loads multi-turn scenarios from JSON files. Each scenario seeds a simulated
/// company state into a mock web server. The agent interacts via HTTP API calls
/// (using ironclaw's `http` tool). Scoring evaluates both the agent's responses
/// and the final state of the simulated company.
pub struct WorkplaceSuite {
    dataset_path: PathBuf,
    /// Base identity files (persona .md files).
    identity_files: HashMap<String, String>,
    /// Skill content loaded from SKILL.md files.
    skill_files: HashMap<String, String>,
    /// Company template states (template_name -> CompanyState).
    templates: HashMap<String, CompanyState>,
    /// Active server for the current task (sequential execution only).
    active_server: Arc<Mutex<Option<WorkplaceServer>>>,
    /// Active interceptor for the current task.
    active_interceptor:
        Arc<Mutex<Option<Arc<interceptor::WorkplaceInterceptor>>>>,
}

impl WorkplaceSuite {
    pub fn new(dataset_path: impl Into<PathBuf>) -> Self {
        let dataset_path = dataset_path.into();

        // Load identity files
        let identity_dir = dataset_path.join("identity");
        let identity_files = load_md_files(&identity_dir);

        // Load skill files
        let skills_dir = dataset_path.join("skills");
        let skill_files = load_md_files(&skills_dir);

        // Load templates
        let templates_dir = dataset_path.join("templates");
        let templates = load_templates(&templates_dir);

        if !identity_files.is_empty() {
            tracing::info!(
                "Workplace: loaded {} identity files, {} skills, {} templates",
                identity_files.len(),
                skill_files.len(),
                templates.len(),
            );
        }

        Self {
            dataset_path,
            identity_files,
            skill_files,
            templates,
            active_server: Arc::new(Mutex::new(None)),
            active_interceptor: Arc::new(Mutex::new(None)),
        }
    }

    /// Build the complete company state for a scenario by merging template + overrides.
    fn build_company_state(&self, setup: &WorkplaceSetup) -> Result<CompanyState, BenchError> {
        let mut state = if let Some(ref template_name) = setup.template {
            self.templates
                .get(template_name)
                .cloned()
                .ok_or_else(|| {
                    BenchError::Config(format!(
                        "unknown company template: \"{template_name}\". Available: {:?}",
                        self.templates.keys().collect::<Vec<_>>()
                    ))
                })?
        } else {
            return Err(BenchError::Config(
                "workplace scenario must specify setup.template".to_string(),
            ));
        };

        if let Some(ref overrides) = setup.company_state_overrides {
            state.merge_overrides(overrides.clone());
        }

        Ok(state)
    }

    /// Build identity context by merging base identity + scenario identity + skill content.
    fn build_identity(
        &self,
        setup: &WorkplaceSetup,
    ) -> HashMap<String, String> {
        let mut identity = HashMap::new();

        // Layer 1: persona identity files
        // Map persona to identity file (e.g., "CEO" -> load ceo.md content)
        for (filename, content) in &self.identity_files {
            // Inject as KNOWLEDGE files so the agent sees them
            let key = format!("KNOWLEDGE_{}", filename.replace(".md", "").to_uppercase());
            identity.insert(format!("{key}.md"), content.clone());
        }

        // Layer 2: skill files
        for skill_name in &setup.skills {
            let filename = format!("{skill_name}.md");
            if let Some(content) = self.skill_files.get(&filename) {
                identity.insert(
                    format!("SKILL_{}.md", skill_name.to_uppercase()),
                    content.clone(),
                );
            }
        }

        // Layer 3: scenario-specific identity (overrides all)
        for (key, value) in &setup.identity {
            identity.insert(key.clone(), value.clone());
        }

        identity
    }

    /// Recursively find all .json files under a directory.
    fn find_scenario_files(dir: &Path) -> Result<Vec<PathBuf>, BenchError> {
        let scenarios_dir = dir.join("scenarios");
        let search_dir = if scenarios_dir.is_dir() {
            &scenarios_dir
        } else {
            dir
        };

        let mut files = Vec::new();
        if search_dir.is_file() {
            files.push(search_dir.to_path_buf());
            return Ok(files);
        }
        if !search_dir.is_dir() {
            return Err(BenchError::Config(format!(
                "scenarios directory does not exist: {}",
                search_dir.display()
            )));
        }
        Self::find_json_recursive(search_dir, &mut files)?;
        files.sort();
        Ok(files)
    }

    fn find_json_recursive(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), BenchError> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                Self::find_json_recursive(&path, files)?;
            } else if path.extension().is_some_and(|ext| ext == "json") {
                files.push(path);
            }
        }
        Ok(())
    }

    fn load_scenario(path: &Path) -> Result<WorkplaceScenario, BenchError> {
        let content = std::fs::read_to_string(path)?;
        let mut scenario: WorkplaceScenario =
            serde_json::from_str(&content).map_err(|e| {
                BenchError::Config(format!("failed to parse {}: {}", path.display(), e))
            })?;

        // Resolve user_input_file references
        let base_dir = path.parent().unwrap_or(Path::new("."));
        for (i, turn) in scenario.turns.iter_mut().enumerate() {
            if turn.user_input.is_empty() {
                if let Some(ref file) = turn.user_input_file {
                    let file_path = base_dir.join(file);
                    turn.user_input = std::fs::read_to_string(&file_path).map_err(|e| {
                        BenchError::Config(format!(
                            "failed to read user_input_file {}: {}",
                            file_path.display(),
                            e
                        ))
                    })?;
                } else {
                    return Err(BenchError::Config(format!(
                        "turn {} in {} has neither user_input nor user_input_file",
                        i,
                        path.display()
                    )));
                }
            }
        }

        Ok(scenario)
    }
}

#[async_trait]
impl BenchSuite for WorkplaceSuite {
    fn name(&self) -> &str {
        "Workplace Simulation"
    }

    fn id(&self) -> &str {
        "workplace"
    }

    async fn load_tasks(&self) -> Result<Vec<BenchTask>, BenchError> {
        let files = Self::find_scenario_files(&self.dataset_path)?;
        let mut tasks = Vec::new();

        for path in files {
            let scenario = Self::load_scenario(&path)?;

            // Build merged identity for metadata
            let identity = self.build_identity(&scenario.setup);

            // Store scenario + computed identity in metadata
            let metadata = serde_json::json!({
                "scenario": serde_json::to_value(&scenario).map_err(|e| {
                    BenchError::Config(format!("failed to serialize scenario {}: {e}", scenario.name))
                })?,
                "identity": identity,
            });

            let first_prompt = scenario
                .turns
                .first()
                .map(|t| t.user_input.clone())
                .unwrap_or_default();

            tasks.push(BenchTask {
                id: scenario.name.clone(),
                prompt: first_prompt,
                context: if scenario.description.is_empty() {
                    None
                } else {
                    Some(scenario.description.clone())
                },
                resources: vec![],
                tags: scenario.tags.clone(),
                expected_turns: Some(scenario.turns.len()),
                timeout: Some(Duration::from_secs(scenario.timeout_secs)),
                metadata,
            });
        }

        Ok(tasks)
    }

    async fn setup_task(&self, task: &BenchTask) -> Result<(), BenchError> {
        let scenario: WorkplaceScenario =
            serde_json::from_value(task.metadata["scenario"].clone()).map_err(|e| {
                BenchError::Config(format!("failed to deserialize scenario: {e}"))
            })?;

        // Build and seed company state
        let company_state = self.build_company_state(&scenario.setup)?;

        // Start mock server
        let server = WorkplaceServer::start(company_state).await;
        tracing::info!(
            "Workplace server started for task '{}' at {}",
            task.id,
            server.base_url()
        );

        // Create interceptor pointing to mock server
        let wp_interceptor = Arc::new(interceptor::WorkplaceInterceptor::new(
            server.base_url(),
        ));
        *self.active_interceptor.lock().await = Some(wp_interceptor);
        *self.active_server.lock().await = Some(server);
        Ok(())
    }

    async fn teardown_task(&self, task: &BenchTask) -> Result<(), BenchError> {
        self.active_interceptor.lock().await.take();
        let server = self.active_server.lock().await.take();
        if let Some(_server) = server {
            tracing::info!("Workplace server stopped for task '{}'", task.id);
            // Server is dropped here, triggering graceful shutdown
        }
        Ok(())
    }

    async fn score(
        &self,
        task: &BenchTask,
        submission: &TaskSubmission,
    ) -> Result<BenchScore, BenchError> {
        let scenario: WorkplaceScenario =
            serde_json::from_value(task.metadata["scenario"].clone()).map_err(|e| {
                BenchError::Scoring {
                    task_id: task.id.clone(),
                    reason: format!("failed to deserialize scenario: {e}"),
                }
            })?;

        let num_turns = scenario.turns.len();
        if num_turns == 0 {
            return Ok(BenchScore::pass());
        }

        // Get HTTP request log from server (if still active)
        let http_log = if let Some(ref server) = *self.active_server.lock().await {
            server.request_log().await
        } else {
            vec![]
        };

        // Get state snapshot for state-based scoring
        let state_snapshot = if let Some(ref server) = *self.active_server.lock().await {
            Some(server.snapshot().await)
        } else {
            None
        };

        let mut all_dimensions: HashMap<String, DimensionResult> = HashMap::new();

        // --- Layer 1: Assertion-based scoring ---
        // Evaluate the last turn's assertions (for single-turn, this is the only turn)
        let last_turn = &scenario.turns[num_turns - 1];
        let (assertion_score, assertion_failures) = last_turn.assertions.evaluate(submission, &http_log);

        all_dimensions.insert(
            "assertion_compliance".to_string(),
            DimensionResult {
                score: assertion_score,
                method: "assertion".to_string(),
                details: if assertion_failures.is_empty() {
                    "all assertions passed".to_string()
                } else {
                    assertion_failures.join("; ")
                },
            },
        );

        // --- Layer 2: State-based scoring ---
        if let (Some(expected), Some(actual)) = (&scenario.expected_state, &state_snapshot)
        {
            let (state_score, state_failures) = evaluate_expected_state(actual, expected);
            all_dimensions.insert(
                "state_compliance".to_string(),
                DimensionResult {
                    score: state_score,
                    method: "state".to_string(),
                    details: if state_failures.is_empty() {
                        "all state assertions passed".to_string()
                    } else {
                        state_failures.join("; ")
                    },
                },
            );
        }

        // --- Layer 3: LLM-as-judge (placeholder — not yet wired) ---
        // When LLM judge is available, evaluate dimensions listed in
        // scenario.scoring.llm_judge_dimensions using build_judge_prompt().

        // --- Compute composite score ---
        let breakdown = if scenario.scoring.weights.is_empty() {
            // No custom weights: use assertion score directly
            ScoreBreakdown::compute(all_dimensions, &scenario.scoring.weights, &scenario.persona)
        } else {
            // Map assertion/state scores to the configured dimensions
            // For now, distribute the assertion score across assertion-based dimensions
            let mut weighted_dims = HashMap::new();
            for (dim_name, _weight) in &scenario.scoring.weights {
                if scenario.scoring.llm_judge_dimensions.contains(dim_name) {
                    // LLM-judged dimensions default to 0.5 (neutral) until judge is wired
                    weighted_dims.insert(
                        dim_name.clone(),
                        DimensionResult {
                            score: 0.5,
                            method: "llm_judge_placeholder".to_string(),
                            details: "LLM judge not yet implemented".to_string(),
                        },
                    );
                } else {
                    // Use assertion score for all other dimensions
                    weighted_dims.insert(
                        dim_name.clone(),
                        DimensionResult {
                            score: assertion_score,
                            method: "assertion".to_string(),
                            details: assertion_failures.join("; "),
                        },
                    );
                }
            }
            // Override with state score if available
            if let Some(state_dim) = all_dimensions.get("state_compliance") {
                // Blend state score into action-related dimensions
                for dim_name in &[
                    "delegation_ownership",
                    "tool_efficiency",
                    "information_synthesis",
                ] {
                    if let Some(d) = weighted_dims.get_mut(*dim_name) {
                        d.score = (d.score + state_dim.score) / 2.0;
                    }
                }
            }
            ScoreBreakdown::compute(weighted_dims, &scenario.scoring.weights, &scenario.persona)
        };

        let composite = breakdown.composite;
        let details_json = serde_json::to_string(&breakdown).unwrap_or_default();

        if composite >= 0.9 {
            Ok(BenchScore {
                value: composite,
                label: "pass".to_string(),
                details: Some(details_json),
            })
        } else if composite <= 0.0 {
            Ok(BenchScore::fail(details_json))
        } else {
            Ok(BenchScore::partial(composite, details_json))
        }
    }

    async fn next_user_message(
        &self,
        task: &BenchTask,
        conversation: &[ConversationTurn],
    ) -> Result<Option<String>, BenchError> {
        let scenario: WorkplaceScenario =
            serde_json::from_value(task.metadata["scenario"].clone()).map_err(|e| {
                BenchError::Scoring {
                    task_id: task.id.clone(),
                    reason: format!("failed to deserialize scenario: {e}"),
                }
            })?;

        let user_turns_sent = conversation
            .iter()
            .filter(|t| matches!(t.role, crate::suite::TurnRole::User))
            .count();

        if user_turns_sent >= scenario.turns.len() {
            return Ok(None);
        }

        let next_turn = &scenario.turns[user_turns_sent];

        // Apply state injection if present (e.g., new Slack messages arriving)
        if let Some(ref injection) = next_turn.state_injection {
            if let Some(ref server) = *self.active_server.lock().await {
                server.inject_state(injection.clone()).await;
                tracing::info!(
                    "Applied state injection before turn {} of '{}'",
                    user_turns_sent,
                    task.id
                );
            }
        }

        Ok(Some(next_turn.user_input.clone()))
    }

    fn additional_tools(&self) -> Vec<Arc<dyn ironclaw::tools::Tool>> {
        // The workplace suite uses HTTP-based tools (skills + http tool),
        // not custom Rust tools. We still provide the http tool via ironclaw's builtins.
        // The agent should already have the http tool available.
        vec![]
    }

    fn http_interceptor(
        &self,
    ) -> Option<Arc<dyn ironclaw::llm::recording::HttpInterceptor>> {
        // try_lock: we're called synchronously from the runner after setup_task
        self.active_interceptor
            .try_lock()
            .ok()
            .and_then(|guard| {
                guard
                    .as_ref()
                    .map(|i| Arc::clone(i) as Arc<dyn ironclaw::llm::recording::HttpInterceptor>)
            })
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Load all `.md` files from a directory (filename -> content).
fn load_md_files(dir: &Path) -> HashMap<String, String> {
    let mut files = HashMap::new();
    if !dir.is_dir() {
        return files;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return files;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "md") {
            if let (Some(name), Ok(content)) = (
                path.file_name().and_then(|n| n.to_str()),
                std::fs::read_to_string(&path),
            ) {
                files.insert(name.to_string(), content);
            }
        }
    }
    files
}

/// Load company template JSON files (stem -> CompanyState).
fn load_templates(dir: &Path) -> HashMap<String, CompanyState> {
    let mut templates = HashMap::new();
    if !dir.is_dir() {
        return templates;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return templates;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "json") {
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();
            match std::fs::read_to_string(&path) {
                Ok(content) => match serde_json::from_str::<CompanyState>(&content) {
                    Ok(state) => {
                        templates.insert(stem, state);
                    }
                    Err(e) => {
                        tracing::warn!("Failed to parse template {}: {}", path.display(), e);
                    }
                },
                Err(e) => {
                    tracing::warn!("Failed to read template {}: {}", path.display(), e);
                }
            }
        }
    }
    templates
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scenario_deserializes() {
        let json = r##"{
            "name": "ceo-001-morning-briefing",
            "description": "CEO morning triage",
            "tags": ["ceo", "basic"],
            "persona": "CEO",
            "setup": {
                "template": "acme-startup",
                "skills": ["slack", "email", "calendar"],
                "identity": { "SOUL.md": "You are the CEO's assistant" }
            },
            "scoring": {
                "weights": { "decision_quality": 0.25, "tool_efficiency": 0.10 },
                "llm_judge_dimensions": ["decision_quality"]
            },
            "turns": [
                {
                    "user_input": "Good morning. What's on my plate?",
                    "assertions": {
                        "http_calls_contain": ["GET /api/slack/"],
                        "response_contains": ["morning"]
                    }
                }
            ],
            "expected_state": {
                "slack_messages_sent": [
                    { "channel": "#leadership", "content_contains": ["agenda"] }
                ]
            }
        }"##;
        let scenario: WorkplaceScenario = serde_json::from_str(json).unwrap();
        assert_eq!(scenario.name, "ceo-001-morning-briefing");
        assert_eq!(scenario.persona, "CEO");
        assert_eq!(scenario.turns.len(), 1);
        assert!(scenario.expected_state.is_some());
        assert_eq!(scenario.setup.skills.len(), 3);
    }

    #[test]
    fn test_turn_with_state_injection() {
        let json = r##"{
            "user_input": "Did you see the urgent message?",
            "state_injection": {
                "slack_channel_messages": {
                    "#leadership": [
                        {
                            "id": "inj-1",
                            "from": "cfo",
                            "from_name": "David",
                            "content": "URGENT: billing is down",
                            "timestamp": "2026-03-29T09:00:00Z"
                        }
                    ]
                }
            },
            "assertions": {
                "response_contains": ["billing", "urgent"]
            }
        }"##;
        let turn: WorkplaceTurn = serde_json::from_str(json).unwrap();
        assert!(turn.state_injection.is_some());
        let inj = turn.state_injection.unwrap();
        assert_eq!(inj.slack_channel_messages.len(), 1);
    }
}
