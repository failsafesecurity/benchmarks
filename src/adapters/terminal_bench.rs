//! Terminal Bench adapter — containerized terminal task evaluation.
//!
//! Each task runs inside its own Docker container. The agent interacts via
//! shell commands (routed through `docker exec`), then a test script verifies
//! success by writing a reward file.
//!
//! Supports both Harbor/TB2 format (`task.toml` + `instruction.md`) and legacy
//! Terminal Bench format (`task.yaml` with inline instruction).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use serde::Deserialize;
use uuid::Uuid;

use ironclaw::context::JobContext;
use ironclaw::tools::{ApprovalRequirement, Tool, ToolDomain, ToolError, ToolOutput};

/// Extract a required string parameter from a JSON object.
fn require_str<'a>(params: &'a serde_json::Value, name: &str) -> Result<&'a str, ToolError> {
    params
        .get(name)
        .and_then(|v| v.as_str())
        .ok_or_else(|| ToolError::InvalidParameters(format!("missing '{name}' parameter")))
}

use crate::docker;
use crate::error::BenchError;
use crate::suite::{BenchScore, BenchSuite, BenchTask, TaskSubmission};

// ---------------------------------------------------------------------------
// Task config parsing (Harbor/TB2 format)
// ---------------------------------------------------------------------------

/// Parsed from `task.toml` (Harbor/TB2 format).
#[derive(Debug, Deserialize)]
struct TbTaskToml {
    #[serde(default)]
    task: TbTaskSection,
    #[serde(default)]
    metadata: TbMetadata,
    #[serde(default)]
    agent: TbAgentConfig,
    #[serde(default)]
    verifier: TbVerifierConfig,
    #[serde(default)]
    environment: TbEnvironment,
}

#[derive(Debug, Default, Deserialize)]
struct TbTaskSection {
    #[serde(default)]
    #[allow(dead_code)]
    name: Option<String>,
    #[serde(default)]
    description: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct TbMetadata {
    #[serde(default)]
    difficulty: Option<String>,
    #[serde(default)]
    category: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct TbAgentConfig {
    #[serde(default = "default_agent_timeout")]
    timeout_sec: f64,
}

impl Default for TbAgentConfig {
    fn default() -> Self {
        Self {
            timeout_sec: default_agent_timeout(),
        }
    }
}

fn default_agent_timeout() -> f64 {
    600.0
}

#[derive(Debug, Deserialize)]
struct TbVerifierConfig {
    #[serde(default = "default_verifier_timeout")]
    timeout_sec: f64,
}

impl Default for TbVerifierConfig {
    fn default() -> Self {
        Self {
            timeout_sec: default_verifier_timeout(),
        }
    }
}

fn default_verifier_timeout() -> f64 {
    600.0
}

#[derive(Debug, Default, Deserialize)]
struct TbEnvironment {
    #[serde(default)]
    cpus: Option<f64>,
    #[serde(default)]
    memory_mb: Option<u64>,
}

/// Parsed from `task.yaml` (legacy Terminal Bench format).
#[derive(Debug, Deserialize)]
struct TbTaskYaml {
    instruction: String,
    #[serde(default)]
    difficulty: Option<String>,
    #[serde(default)]
    category: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default = "default_agent_timeout")]
    max_agent_timeout_sec: f64,
    #[serde(default = "default_verifier_timeout")]
    max_test_timeout_sec: f64,
}

// ---------------------------------------------------------------------------
// Per-task runtime state stored in metadata
// ---------------------------------------------------------------------------

/// Metadata stored on BenchTask.metadata for runtime use.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct TbTaskMeta {
    task_dir: String,
    verifier_timeout_sec: f64,
    #[serde(default)]
    cpus: Option<f64>,
    #[serde(default)]
    memory_mb: Option<u64>,
}

// ---------------------------------------------------------------------------
// DockerExecTool — routes shell commands into the task container
// ---------------------------------------------------------------------------

/// A tool that executes shell commands inside a Docker container.
///
/// Named `"shell"` so the agent uses it naturally. Looks up the container ID
/// from the shared map using the task ID extracted from `JobContext.title`.
pub struct DockerExecTool {
    containers: Arc<tokio::sync::Mutex<HashMap<String, String>>>,
    timeout: Duration,
}

#[async_trait]
impl Tool for DockerExecTool {
    fn name(&self) -> &str {
        "shell"
    }

    fn description(&self) -> &str {
        "Execute a shell command. The command runs inside an isolated container environment."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The shell command to execute"
                }
            },
            "required": ["command"]
        })
    }

    async fn execute(
        &self,
        params: serde_json::Value,
        ctx: &JobContext,
    ) -> Result<ToolOutput, ToolError> {
        let command = require_str(&params, "command")?;
        let container_id = resolve_container_id(&self.containers, &ctx.title).await?;

        let start = std::time::Instant::now();
        let result = docker::exec_in_container(&container_id, command, self.timeout).await;

        match result {
            Ok(output) => {
                let duration = start.elapsed();
                let text = if output.stderr.is_empty() {
                    output.stdout
                } else if output.stdout.is_empty() {
                    output.stderr
                } else {
                    format!("{}\n{}", output.stdout, output.stderr)
                };
                let mut tool_output = ToolOutput::text(text, duration);
                if output.exit_code != 0 {
                    tool_output.raw = Some(format!("exit code: {}", output.exit_code));
                }
                Ok(tool_output)
            }
            Err(e) => Err(ToolError::ExecutionFailed(e.to_string())),
        }
    }

    fn requires_approval(&self, _params: &serde_json::Value) -> ApprovalRequirement {
        ApprovalRequirement::Never
    }

    fn domain(&self) -> ToolDomain {
        ToolDomain::Container
    }

    fn execution_timeout(&self) -> Duration {
        self.timeout
    }

    fn requires_sanitization(&self) -> bool {
        true
    }
}

/// DockerReadFileTool — reads files from inside the task container.
pub struct DockerReadFileTool {
    containers: Arc<tokio::sync::Mutex<HashMap<String, String>>>,
}

#[async_trait]
impl Tool for DockerReadFileTool {
    fn name(&self) -> &str {
        "read_file"
    }

    fn description(&self) -> &str {
        "Read the contents of a file at the given path."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Absolute path to the file to read"
                }
            },
            "required": ["path"]
        })
    }

    async fn execute(
        &self,
        params: serde_json::Value,
        ctx: &JobContext,
    ) -> Result<ToolOutput, ToolError> {
        let path = require_str(&params, "path")?;
        let container_id = resolve_container_id(&self.containers, &ctx.title).await?;

        let start = std::time::Instant::now();
        let cmd = format!("cat {}", shell_escape(path));
        match docker::exec_in_container(&container_id, &cmd, Duration::from_secs(30)).await {
            Ok(output) if output.exit_code == 0 => {
                Ok(ToolOutput::text(output.stdout, start.elapsed()))
            }
            Ok(output) => Err(ToolError::ExecutionFailed(format!(
                "failed to read {path}: {}",
                output.stderr
            ))),
            Err(e) => Err(ToolError::ExecutionFailed(e.to_string())),
        }
    }

    fn requires_approval(&self, _params: &serde_json::Value) -> ApprovalRequirement {
        ApprovalRequirement::Never
    }

    fn domain(&self) -> ToolDomain {
        ToolDomain::Container
    }

    fn requires_sanitization(&self) -> bool {
        true
    }
}

/// DockerWriteFileTool — writes files inside the task container.
pub struct DockerWriteFileTool {
    containers: Arc<tokio::sync::Mutex<HashMap<String, String>>>,
}

#[async_trait]
impl Tool for DockerWriteFileTool {
    fn name(&self) -> &str {
        "write_file"
    }

    fn description(&self) -> &str {
        "Write content to a file at the given path, creating it if it doesn't exist."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Absolute path to write the file"
                },
                "content": {
                    "type": "string",
                    "description": "Content to write to the file"
                }
            },
            "required": ["path", "content"]
        })
    }

    async fn execute(
        &self,
        params: serde_json::Value,
        ctx: &JobContext,
    ) -> Result<ToolOutput, ToolError> {
        let path = require_str(&params, "path")?;
        let content = require_str(&params, "content")?;
        let container_id = resolve_container_id(&self.containers, &ctx.title).await?;

        let start = std::time::Instant::now();
        // Use heredoc to avoid quoting issues
        let cmd = format!(
            "mkdir -p $(dirname {path_escaped}) && cat > {path_escaped} << 'TBEOF'\n{content}\nTBEOF",
            path_escaped = shell_escape(path),
            content = content,
        );
        match docker::exec_in_container(&container_id, &cmd, Duration::from_secs(30)).await {
            Ok(output) if output.exit_code == 0 => {
                Ok(ToolOutput::text(format!("wrote {path}"), start.elapsed()))
            }
            Ok(output) => Err(ToolError::ExecutionFailed(format!(
                "failed to write {path}: {}",
                output.stderr
            ))),
            Err(e) => Err(ToolError::ExecutionFailed(e.to_string())),
        }
    }

    fn requires_approval(&self, _params: &serde_json::Value) -> ApprovalRequirement {
        ApprovalRequirement::Never
    }

    fn domain(&self) -> ToolDomain {
        ToolDomain::Container
    }

    fn requires_sanitization(&self) -> bool {
        false
    }
}

// ---------------------------------------------------------------------------
// TerminalBenchSuite
// ---------------------------------------------------------------------------

pub struct TerminalBenchSuite {
    dataset_path: PathBuf,
    /// Upstream git repo to clone task definitions from when `dataset_path`
    /// is missing or empty. Default: harbor-framework/terminal-bench.
    upstream_repo: String,
    /// Subdirectory within the upstream repo containing one task per dir.
    upstream_tasks_subdir: String,
    rebuild_images: bool,
    verifier_timeout: Duration,
    /// task_id -> container_id (shared with DockerExecTool instances)
    containers: Arc<tokio::sync::Mutex<HashMap<String, String>>>,
    /// task_id -> reward value (populated in teardown, read in score)
    rewards: Arc<tokio::sync::Mutex<HashMap<String, f64>>>,
}

impl TerminalBenchSuite {
    pub fn new(
        dataset_path: impl Into<PathBuf>,
        upstream_repo: impl Into<String>,
        upstream_tasks_subdir: impl Into<String>,
        rebuild_images: bool,
        verifier_timeout: Duration,
    ) -> Self {
        Self {
            dataset_path: dataset_path.into(),
            upstream_repo: upstream_repo.into(),
            upstream_tasks_subdir: upstream_tasks_subdir.into(),
            rebuild_images,
            verifier_timeout,
            containers: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            rewards: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        }
    }

    /// Clone the upstream repo (shallow) and copy the tasks subdirectory into
    /// `dataset_path`. Idempotent: skips if `dataset_path` already has tasks.
    async fn ensure_dataset(&self) -> Result<(), BenchError> {
        if dataset_has_tasks(&self.dataset_path) {
            return Ok(());
        }

        tracing::info!(
            "Terminal Bench dataset is empty; cloning {} into {}",
            self.upstream_repo,
            self.dataset_path.display()
        );

        let tmp = tempfile::tempdir()
            .map_err(|e| BenchError::Config(format!("failed to create temp dir: {e}")))?;
        let clone_dir = tmp.path().join("upstream");

        let status = tokio::process::Command::new("git")
            .args([
                "clone",
                "--depth",
                "1",
                "--quiet",
                &self.upstream_repo,
                clone_dir.to_str().unwrap(),
            ])
            .status()
            .await
            .map_err(|e| BenchError::Config(format!("git clone failed to spawn: {e}")))?;
        if !status.success() {
            return Err(BenchError::Config(format!(
                "git clone {} failed (exit {})",
                self.upstream_repo,
                status.code().unwrap_or(-1)
            )));
        }

        let src = clone_dir.join(&self.upstream_tasks_subdir);
        if !src.is_dir() {
            return Err(BenchError::Config(format!(
                "upstream subdir '{}' not found in {}",
                self.upstream_tasks_subdir,
                self.upstream_repo
            )));
        }

        std::fs::create_dir_all(&self.dataset_path)?;
        copy_dir_recursive(&src, &self.dataset_path)?;

        tracing::info!(
            "Cloned {} task dirs into {}",
            std::fs::read_dir(&self.dataset_path)?.count(),
            self.dataset_path.display()
        );
        Ok(())
    }
}

/// True if `dir` exists and contains at least one subdirectory with a
/// `task.toml` or `task.yaml` (the markers we use to identify a task).
fn dataset_has_tasks(dir: &Path) -> bool {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return false;
    };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() && (p.join("task.toml").exists() || p.join("task.yaml").exists()) {
            return true;
        }
    }
    false
}

/// Recursively copy `src` directory contents into `dst`.
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), BenchError> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if from.is_dir() {
            copy_dir_recursive(&from, &to)?;
        } else {
            std::fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

#[async_trait]
impl BenchSuite for TerminalBenchSuite {
    fn name(&self) -> &str {
        "Terminal Bench"
    }

    fn id(&self) -> &str {
        "terminal_bench"
    }

    async fn load_tasks(&self) -> Result<Vec<BenchTask>, BenchError> {
        self.ensure_dataset().await?;

        let mut tasks = Vec::new();
        discover_tasks(&self.dataset_path, &self.dataset_path, &mut tasks)?;
        if tasks.is_empty() {
            return Err(BenchError::Config(format!(
                "no Terminal Bench tasks found in {}",
                self.dataset_path.display()
            )));
        }
        tracing::info!("Loaded {} Terminal Bench tasks", tasks.len());
        Ok(tasks)
    }

    async fn setup_task(&self, task: &BenchTask) -> Result<(), BenchError> {
        let meta: TbTaskMeta = serde_json::from_value(task.metadata.clone()).map_err(|e| {
            BenchError::Config(format!(
                "invalid terminal_bench metadata for {}: {e}",
                task.id
            ))
        })?;

        let task_dir = PathBuf::from(&meta.task_dir);

        // Resolve Dockerfile: prefer environment/Dockerfile, fall back to task root
        let dockerfile = if task_dir.join("environment/Dockerfile").exists() {
            task_dir.join("environment/Dockerfile")
        } else if task_dir.join("Dockerfile").exists() {
            task_dir.join("Dockerfile")
        } else {
            return Err(BenchError::Docker(format!(
                "no Dockerfile found for task {}",
                task.id
            )));
        };

        // Build context is the task directory (or environment/ if Dockerfile is there)
        let build_context = if task_dir.join("environment/Dockerfile").exists() {
            task_dir.join("environment")
        } else {
            task_dir.clone()
        };

        let image_tag = format!("tb-{}", sanitize_for_docker(&task.id));
        docker::build_image(&image_tag, &dockerfile, &build_context, self.rebuild_images).await?;

        let container_name = format!("tb-{}-{}", sanitize_for_docker(&task.id), short_uuid());

        let container_id = docker::start_container(
            &image_tag,
            &container_name,
            meta.cpus,
            meta.memory_mb,
            &[],
            &[],
        )
        .await?;

        // Create /logs/verifier/ inside the container for reward files
        docker::exec_in_container(
            &container_id,
            "mkdir -p /logs/verifier /logs/agent",
            Duration::from_secs(10),
        )
        .await?;

        // Copy test files into the container if they exist
        let tests_dir = task_dir.join("tests");
        if tests_dir.exists() {
            copy_dir_to_container(&container_id, &tests_dir, "/tests").await?;
        }

        // Copy test.sh / run-tests.sh if at task root
        let test_sh = task_dir.join("test.sh");
        if test_sh.exists() {
            copy_file_to_container(&container_id, &test_sh, "/tests/test.sh").await?;
        }
        let run_tests_sh = task_dir.join("run-tests.sh");
        if run_tests_sh.exists() {
            copy_file_to_container(&container_id, &run_tests_sh, "/tests/run-tests.sh").await?;
        }

        // Copy solution if present (for oracle runs)
        let solution_dir = task_dir.join("solution");
        if solution_dir.exists() {
            copy_dir_to_container(&container_id, &solution_dir, "/solution").await?;
        }

        self.containers
            .lock()
            .await
            .insert(task.id.clone(), container_id);

        Ok(())
    }

    async fn teardown_task(&self, task: &BenchTask) -> Result<(), BenchError> {
        let container_id = {
            let mut map = self.containers.lock().await;
            match map.remove(&task.id) {
                Some(id) => id,
                None => {
                    tracing::warn!("no container found for task {} during teardown", task.id);
                    return Ok(());
                }
            }
        };

        // Run the test/verifier script inside the container
        let reward = run_verifier(&container_id, self.verifier_timeout).await;

        // Cache the reward for score()
        self.rewards.lock().await.insert(task.id.clone(), reward);

        // Clean up the container
        docker::stop_and_remove(&container_id).await?;

        Ok(())
    }

    async fn score(
        &self,
        task: &BenchTask,
        _submission: &TaskSubmission,
    ) -> Result<BenchScore, BenchError> {
        let reward = self.rewards.lock().await.remove(&task.id).unwrap_or(0.0);

        if reward >= 1.0 {
            Ok(BenchScore::pass())
        } else if reward <= 0.0 {
            Ok(BenchScore::fail("task not solved"))
        } else {
            Ok(BenchScore::partial(reward, format!("reward: {reward:.2}")))
        }
    }

    fn additional_tools(&self) -> Vec<Arc<dyn Tool>> {
        vec![
            Arc::new(DockerExecTool {
                containers: Arc::clone(&self.containers),
                timeout: Duration::from_secs(120),
            }),
            Arc::new(DockerReadFileTool {
                containers: Arc::clone(&self.containers),
            }),
            Arc::new(DockerWriteFileTool {
                containers: Arc::clone(&self.containers),
            }),
        ]
    }

    async fn openclaw_task_metadata(
        &self,
        task: &BenchTask,
    ) -> Option<serde_json::Map<String, serde_json::Value>> {
        let containers = self.containers.lock().await;
        let container_id = containers.get(&task.id)?;
        let mut map = serde_json::Map::new();
        map.insert(
            "tb_container_id".to_string(),
            serde_json::Value::String(container_id.clone()),
        );
        Some(map)
    }
}

// ---------------------------------------------------------------------------
// Task discovery
// ---------------------------------------------------------------------------

/// Recursively walk `dir` looking for task definitions.
fn discover_tasks(root: &Path, dir: &Path, tasks: &mut Vec<BenchTask>) -> Result<(), BenchError> {
    let entries = std::fs::read_dir(dir).map_err(|e| {
        BenchError::Config(format!("failed to read directory {}: {e}", dir.display()))
    })?;

    for entry in entries {
        let entry = entry.map_err(BenchError::Io)?;
        let path = entry.path();

        if path.is_dir() {
            // Check for task.toml (Harbor/TB2 format)
            if path.join("task.toml").exists() {
                match load_tb2_task(root, &path) {
                    Ok(task) => tasks.push(task),
                    Err(e) => tracing::warn!("Skipping task at {}: {e}", path.display()),
                }
            }
            // Check for task.yaml (legacy format)
            else if path.join("task.yaml").exists() {
                match load_legacy_task(root, &path) {
                    Ok(task) => tasks.push(task),
                    Err(e) => tracing::warn!("Skipping task at {}: {e}", path.display()),
                }
            }
            // Otherwise recurse
            else {
                discover_tasks(root, &path, tasks)?;
            }
        }
    }

    Ok(())
}

/// Load a task from Harbor/TB2 format (task.toml + instruction.md).
fn load_tb2_task(root: &Path, task_dir: &Path) -> Result<BenchTask, BenchError> {
    let toml_path = task_dir.join("task.toml");
    let toml_str = std::fs::read_to_string(&toml_path).map_err(BenchError::Io)?;
    let config: TbTaskToml = toml::from_str(&toml_str).map_err(BenchError::Toml)?;

    let instruction = if task_dir.join("instruction.md").exists() {
        std::fs::read_to_string(task_dir.join("instruction.md")).map_err(BenchError::Io)?
    } else {
        config
            .task
            .description
            .clone()
            .unwrap_or_else(|| "No instruction provided".to_string())
    };

    let task_id = derive_task_id(root, task_dir);

    let mut tags = config.metadata.tags.clone();
    if let Some(ref diff) = config.metadata.difficulty {
        tags.push(format!("difficulty:{diff}"));
    }
    if let Some(ref cat) = config.metadata.category {
        tags.push(format!("category:{cat}"));
    }

    let meta = TbTaskMeta {
        task_dir: task_dir.to_string_lossy().to_string(),
        verifier_timeout_sec: config.verifier.timeout_sec,
        cpus: config.environment.cpus,
        memory_mb: config.environment.memory_mb,
    };

    Ok(BenchTask {
        id: task_id,
        prompt: instruction,
        context: None,
        resources: vec![],
        tags,
        expected_turns: None,
        timeout: Some(Duration::from_secs_f64(config.agent.timeout_sec)),
        metadata: serde_json::to_value(meta).map_err(BenchError::Json)?,
    })
}

/// Load a task from legacy Terminal Bench format (task.yaml).
fn load_legacy_task(root: &Path, task_dir: &Path) -> Result<BenchTask, BenchError> {
    let yaml_path = task_dir.join("task.yaml");
    let yaml_str = std::fs::read_to_string(&yaml_path).map_err(BenchError::Io)?;
    let config: TbTaskYaml = serde_yaml::from_str(&yaml_str)
        .map_err(|e| BenchError::Config(format!("failed to parse {}: {e}", yaml_path.display())))?;

    let task_id = derive_task_id(root, task_dir);

    let mut tags = config.tags.clone();
    if let Some(ref diff) = config.difficulty {
        tags.push(format!("difficulty:{diff}"));
    }
    if let Some(ref cat) = config.category {
        tags.push(format!("category:{cat}"));
    }

    let meta = TbTaskMeta {
        task_dir: task_dir.to_string_lossy().to_string(),
        verifier_timeout_sec: config.max_test_timeout_sec,
        cpus: None,
        memory_mb: None,
    };

    Ok(BenchTask {
        id: task_id,
        prompt: config.instruction,
        context: None,
        resources: vec![],
        tags,
        expected_turns: None,
        timeout: Some(Duration::from_secs_f64(config.max_agent_timeout_sec)),
        metadata: serde_json::to_value(meta).map_err(BenchError::Json)?,
    })
}

// ---------------------------------------------------------------------------
// Verifier / reward parsing
// ---------------------------------------------------------------------------

/// Run the verifier (test script) inside the container and return the reward.
///
/// Reward sources, in order of preference:
/// 1. `/logs/verifier/reward.txt` — single numeric value (custom suites)
/// 2. `/logs/verifier/reward.json` — `{"reward": float}` or numeric (custom suites)
/// 3. The verifier's own exit code — 0 ⇒ 1.0, non-zero ⇒ 0.0 (real terminal-bench
///    tasks rely on pytest exit status; they don't write reward files).
async fn run_verifier(container_id: &str, timeout: Duration) -> f64 {
    // Try test.sh first, then run-tests.sh
    let test_script = if docker::exec_in_container(
        container_id,
        "test -f /tests/test.sh",
        Duration::from_secs(5),
    )
    .await
    .map(|o| o.exit_code == 0)
    .unwrap_or(false)
    {
        "TEST_DIR=/tests bash /tests/test.sh"
    } else if docker::exec_in_container(
        container_id,
        "test -f /tests/run-tests.sh",
        Duration::from_secs(5),
    )
    .await
    .map(|o| o.exit_code == 0)
    .unwrap_or(false)
    {
        "TEST_DIR=/tests bash /tests/run-tests.sh"
    } else {
        tracing::warn!("no test script found in container {container_id}");
        return 0.0;
    };

    tracing::info!("Running verifier: {test_script}");
    let verifier_exit_code = match docker::exec_in_container(container_id, test_script, timeout)
        .await
    {
        Ok(output) => {
            if !output.stderr.is_empty() {
                tracing::debug!("verifier stderr: {}", output.stderr.trim());
            }
            output.exit_code
        }
        Err(e) => {
            tracing::warn!("verifier execution failed: {e}");
            return 0.0;
        }
    };

    // Read reward.txt first, fall back to reward.json
    if let Ok(output) = docker::exec_in_container(
        container_id,
        "cat /logs/verifier/reward.txt",
        Duration::from_secs(5),
    )
    .await
    {
        if output.exit_code == 0 {
            if let Ok(reward) = output.stdout.trim().parse::<f64>() {
                return reward;
            }
        }
    }

    if let Ok(output) = docker::exec_in_container(
        container_id,
        "cat /logs/verifier/reward.json",
        Duration::from_secs(5),
    )
    .await
    {
        if output.exit_code == 0 {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(output.stdout.trim()) {
                // Support both {"reward": 1.0} and plain numeric values
                if let Some(r) = json.get("reward").and_then(|v| v.as_f64()) {
                    return r;
                }
                if let Some(r) = json.as_f64() {
                    return r;
                }
            }
        }
    }

    // Real terminal-bench tasks rely on the verifier exit code (pytest).
    tracing::info!(
        "no reward file in {container_id}; using verifier exit code {verifier_exit_code} as reward"
    );
    if verifier_exit_code == 0 { 1.0 } else { 0.0 }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Derive a task ID from the relative path under the dataset root.
fn derive_task_id(root: &Path, task_dir: &Path) -> String {
    task_dir
        .strip_prefix(root)
        .unwrap_or(task_dir)
        .to_string_lossy()
        .replace(['/', '\\'], "-")
        .trim_matches('-')
        .to_string()
}

/// Extract the task ID from a `JobContext.title` like `"bench-{task_id}"`.
fn extract_task_id(title: &str) -> &str {
    title.strip_prefix("bench-").unwrap_or(title)
}

/// Resolve the container ID for the current tool invocation.
///
/// In ironclaw, `JobContext.title` is set to "chat" (not the agent name), so we
/// can't recover the task ID from the title. Since the bench harness runs one
/// task at a time per agent (and the container map is per-suite), if the map
/// has exactly one entry it must be ours.
async fn resolve_container_id(
    containers: &tokio::sync::Mutex<HashMap<String, String>>,
    title: &str,
) -> Result<String, ToolError> {
    let map = containers.lock().await;
    let task_id = extract_task_id(title);
    if let Some(id) = map.get(task_id) {
        return Ok(id.clone());
    }
    if map.len() == 1 {
        return Ok(map.values().next().unwrap().clone());
    }
    Err(ToolError::ExecutionFailed(format!(
        "no container available (title='{title}', map size={})",
        map.len()
    )))
}

/// Sanitize a string for use as a Docker image/container name.
fn sanitize_for_docker(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '.' {
                c
            } else {
                '-'
            }
        })
        .collect::<String>()
        .to_lowercase()
}

/// Generate a short UUID suffix for container names.
fn short_uuid() -> String {
    Uuid::new_v4().to_string()[..8].to_string()
}

/// Basic shell escaping — wraps in single quotes.
fn shell_escape(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

/// Copy a local file into a container.
async fn copy_file_to_container(
    container_id: &str,
    local_path: &Path,
    container_path: &str,
) -> Result<(), BenchError> {
    // Ensure parent directory exists
    if let Some(parent) = Path::new(container_path).parent() {
        docker::exec_in_container(
            container_id,
            &format!("mkdir -p {}", parent.display()),
            Duration::from_secs(10),
        )
        .await?;
    }

    let output = tokio::process::Command::new("docker")
        .args([
            "cp",
            &local_path.to_string_lossy(),
            &format!("{container_id}:{container_path}"),
        ])
        .output()
        .await
        .map_err(|e| BenchError::Docker(format!("docker cp failed: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(BenchError::Docker(format!("docker cp failed: {stderr}")));
    }
    Ok(())
}

/// Copy a local directory into a container.
async fn copy_dir_to_container(
    container_id: &str,
    local_dir: &Path,
    container_path: &str,
) -> Result<(), BenchError> {
    docker::exec_in_container(
        container_id,
        &format!("mkdir -p {container_path}"),
        Duration::from_secs(10),
    )
    .await?;

    // docker cp src/. container:dest copies contents of src into dest
    let src = format!("{}/.", &local_dir.to_string_lossy());
    let output = tokio::process::Command::new("docker")
        .args(["cp", &src, &format!("{container_id}:{container_path}")])
        .output()
        .await
        .map_err(|e| BenchError::Docker(format!("docker cp failed: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        tracing::warn!("docker cp dir failed: {stderr}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_task_id() {
        let root = Path::new("/data/terminal-bench/tasks");
        let task_dir = Path::new("/data/terminal-bench/tasks/hello-world");
        assert_eq!(derive_task_id(root, task_dir), "hello-world");

        let nested = Path::new("/data/terminal-bench/tasks/category/my-task");
        assert_eq!(derive_task_id(root, nested), "category-my-task");
    }

    #[test]
    fn test_extract_task_id() {
        assert_eq!(extract_task_id("bench-hello-world"), "hello-world");
        assert_eq!(
            extract_task_id("bench-category-my-task"),
            "category-my-task"
        );
        assert_eq!(extract_task_id("no-prefix"), "no-prefix");
    }

    #[test]
    fn test_sanitize_for_docker() {
        assert_eq!(sanitize_for_docker("hello-world"), "hello-world");
        assert_eq!(sanitize_for_docker("My/Task_v2"), "my-task-v2");
    }

    #[test]
    fn test_parse_tb2_toml() {
        let toml_str = r#"
[task]
name = "test/hello"

[metadata]
difficulty = "easy"
tags = ["basic"]

[agent]
timeout_sec = 300.0

[verifier]
timeout_sec = 60.0

[environment]
cpus = 2.0
memory_mb = 4096
"#;
        let config: TbTaskToml = toml::from_str(toml_str).unwrap();
        assert_eq!(config.agent.timeout_sec, 300.0);
        assert_eq!(config.verifier.timeout_sec, 60.0);
        assert_eq!(config.environment.cpus, Some(2.0));
        assert_eq!(config.metadata.tags, vec!["basic"]);
    }
}
