use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::BenchError;
use crate::suite::{BenchScore, BenchSuite, BenchTask, ConversationTurn, TaskSubmission, TurnRole};

// ---------------------------------------------------------------------------
// Data structures parsed from task .md files
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PinchTaskMeta {
    id: String,
    name: String,
    #[serde(default)]
    category: String,
    grading_type: GradingType,
    #[serde(default = "default_timeout")]
    timeout_seconds: u64,
    #[serde(default)]
    workspace_files: Vec<WorkspaceFile>,
    #[serde(default)]
    multi_session: bool,
    #[serde(default)]
    sessions: Vec<SessionDef>,
}

fn default_timeout() -> u64 {
    120
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
enum GradingType {
    Automated,
    LlmJudge,
    Hybrid,
}

impl std::fmt::Display for GradingType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GradingType::Automated => write!(f, "automated"),
            GradingType::LlmJudge => write!(f, "llm_judge"),
            GradingType::Hybrid => write!(f, "hybrid"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WorkspaceFile {
    #[serde(default)]
    path: Option<String>,
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    source: Option<String>,
    #[serde(default)]
    dest: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SessionDef {
    id: String,
    prompt: String,
    #[serde(default)]
    new_session: bool,
}

/// Parsed sections from the markdown body.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PinchTaskContent {
    prompt: String,
    expected_behavior: String,
    #[serde(default)]
    automated_checks: Option<String>,
    #[serde(default)]
    llm_judge_rubric: Option<String>,
}

// ---------------------------------------------------------------------------
// PinchBenchSuite
// ---------------------------------------------------------------------------

pub struct PinchBenchSuite {
    dataset_path: PathBuf,
    judge_model: String,
    hybrid_auto_weight: f64,
    /// Base directory for all task workspaces.
    /// Initially `/tmp/pinchbench-workspaces`, overridden by the runner
    /// to `results/{run_id}/workspaces/` so files persist for scoring.
    workspace_base: std::sync::RwLock<PathBuf>,
}

impl PinchBenchSuite {
    pub fn new(
        dataset_path: impl Into<PathBuf>,
        judge_model: String,
        hybrid_auto_weight: f64,
    ) -> Self {
        Self {
            dataset_path: dataset_path.into(),
            judge_model,
            hybrid_auto_weight,
            workspace_base: std::sync::RwLock::new(PathBuf::from("/tmp/pinchbench-workspaces")),
        }
    }

    /// Get the deterministic workspace path for a task.
    fn workspace_path_for(&self, task_id: &str) -> PathBuf {
        self.workspace_base.read().unwrap().join(task_id)
    }

    /// Find all .md task files (excluding TASK_TEMPLATE.md).
    fn find_task_files(dir: &Path) -> Result<Vec<PathBuf>, BenchError> {
        if !dir.is_dir() {
            return Err(BenchError::Config(format!(
                "pinchbench tasks directory does not exist: {}",
                dir.display()
            )));
        }
        let mut files = Vec::new();
        for entry in std::fs::read_dir(dir)? {
            let path = entry?.path();
            if path.extension().is_some_and(|e| e == "md") {
                let name = path.file_name().unwrap_or_default().to_string_lossy();
                if name.starts_with("task_") {
                    files.push(path);
                }
            }
        }
        files.sort();
        Ok(files)
    }

    /// Parse a PinchBench task from a markdown file.
    fn parse_task_file(path: &Path) -> Result<(PinchTaskMeta, PinchTaskContent), BenchError> {
        let raw = std::fs::read_to_string(path).map_err(|e| {
            BenchError::Config(format!("failed to read {}: {e}", path.display()))
        })?;

        let (meta, body) = parse_frontmatter(&raw).map_err(|e| {
            BenchError::Config(format!("bad frontmatter in {}: {e}", path.display()))
        })?;

        let content = parse_sections(&body);
        Ok((meta, content))
    }

    /// Seed workspace files for a task using a deterministic path.
    fn seed_workspace(
        &self,
        task_id: &str,
        workspace_files: &[WorkspaceFile],
    ) -> Result<PathBuf, BenchError> {
        let ws_path = self.workspace_path_for(task_id);

        // Clean and recreate
        if ws_path.exists() {
            std::fs::remove_dir_all(&ws_path).ok();
        }
        std::fs::create_dir_all(&ws_path)?;

        for wf in workspace_files {
            if let (Some(path), Some(content)) = (&wf.path, &wf.content) {
                // Inline content
                let dest = ws_path.join(path);
                if let Some(parent) = dest.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(&dest, content)?;
            } else if let (Some(source), Some(dest_name)) = (&wf.source, &wf.dest) {
                // Asset reference
                let src = self.dataset_path.join("assets").join(source);
                let dest = ws_path.join(dest_name);
                if let Some(parent) = dest.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                if src.exists() {
                    std::fs::copy(&src, &dest).map_err(|e| {
                        BenchError::PinchBench(format!(
                            "failed to copy asset {} -> {}: {e}",
                            src.display(),
                            dest.display()
                        ))
                    })?;
                } else {
                    tracing::warn!(
                        "Asset not found: {} (needed by task {})",
                        src.display(),
                        task_id
                    );
                }
            }
        }

        Ok(ws_path)
    }
}

// ---------------------------------------------------------------------------
// BenchSuite implementation
// ---------------------------------------------------------------------------

#[async_trait]
impl BenchSuite for PinchBenchSuite {
    fn name(&self) -> &str {
        "PinchBench"
    }

    fn id(&self) -> &str {
        "pinchbench"
    }

    async fn load_tasks(&self) -> Result<Vec<BenchTask>, BenchError> {
        let tasks_dir = self.dataset_path.join("tasks");
        let files = Self::find_task_files(&tasks_dir)?;
        let mut tasks = Vec::new();

        for path in files {
            let (meta, content) = Self::parse_task_file(&path)?;

            // For multi-session tasks, the prompt is the first session's prompt.
            let raw_prompt = if meta.multi_session && !meta.sessions.is_empty() {
                meta.sessions[0].prompt.clone()
            } else {
                content.prompt.clone()
            };

            // Prepend workspace instructions with a placeholder that the runner
            // substitutes at runtime: IronClaw → host temp dir, OpenClaw →
            // container workspace path. See runner::WORKSPACE_PLACEHOLDER.
            let ws = crate::runner::WORKSPACE_PLACEHOLDER;
            let prompt = if meta.workspace_files.is_empty() {
                format!(
                    "IMPORTANT: Your workspace directory is {ws}\n\
                     You MUST use absolute paths for ALL file operations.\n\
                     For example, to create output.txt, write to {ws}/output.txt\n\n\
                     {prompt}",
                    prompt = raw_prompt
                )
            } else {
                let file_list: Vec<String> = meta
                    .workspace_files
                    .iter()
                    .filter_map(|wf| {
                        wf.path
                            .as_deref()
                            .or(wf.dest.as_deref())
                            .map(|p| format!("  - {ws}/{p}"))
                    })
                    .collect();
                format!(
                    "IMPORTANT: Your workspace directory is {ws}\n\
                     You MUST use absolute paths for ALL file operations.\n\
                     The following files are already present:\n{files}\n\n\
                     {prompt}",
                    files = file_list.join("\n"),
                    prompt = raw_prompt
                )
            };

            let expected_turns = if meta.multi_session {
                Some(meta.sessions.len())
            } else {
                Some(1)
            };

            let metadata = serde_json::json!({
                "meta": {
                    "id": meta.id,
                    "name": meta.name,
                    "category": meta.category,
                    "grading_type": meta.grading_type,
                    "timeout_seconds": meta.timeout_seconds,
                    "workspace_files": meta.workspace_files,
                    "multi_session": meta.multi_session,
                    "sessions": meta.sessions,
                    "_dataset_path": self.dataset_path.to_string_lossy(),
                    "_workspace_base": self.workspace_base.read().unwrap().to_string_lossy().to_string(),
                },
                "content": content,
            });

            tasks.push(BenchTask {
                id: meta.id.clone(),
                prompt,
                context: None,
                resources: vec![],
                tags: vec![
                    meta.category.clone(),
                    meta.grading_type.to_string(),
                ],
                expected_turns,
                timeout: Some(Duration::from_secs(meta.timeout_seconds)),
                metadata,
            });
        }

        Ok(tasks)
    }

    fn set_run_workspace_base(&self, path: std::path::PathBuf) {
        *self.workspace_base.write().unwrap() = path;
    }

    async fn setup_task(&self, task: &BenchTask) -> Result<(), BenchError> {
        let workspace_files: Vec<WorkspaceFile> = task
            .metadata
            .get("meta")
            .and_then(|m| m.get("workspace_files"))
            .and_then(|wf| serde_json::from_value(wf.clone()).ok())
            .unwrap_or_default();

        self.seed_workspace(&task.id, &workspace_files)?;
        Ok(())
    }

    async fn teardown_task(&self, _task: &BenchTask) -> Result<(), BenchError> {
        // Don't clean up workspace here — scoring happens after teardown.
        // Workspaces are cleaned and recreated in setup_task() on next run.
        Ok(())
    }

    async fn score(
        &self,
        task: &BenchTask,
        submission: &TaskSubmission,
    ) -> Result<BenchScore, BenchError> {
        let meta: PinchTaskMeta = serde_json::from_value(
            task.metadata.get("meta").cloned().unwrap_or_default(),
        )
        .map_err(|e| BenchError::Scoring {
            task_id: task.id.clone(),
            reason: format!("failed to parse task meta: {e}"),
        })?;

        let content: PinchTaskContent = serde_json::from_value(
            task.metadata.get("content").cloned().unwrap_or_default(),
        )
        .map_err(|e| BenchError::Scoring {
            task_id: task.id.clone(),
            reason: format!("failed to parse task content: {e}"),
        })?;

        let workspace_path = self.workspace_path_for(&task.id);

        let transcript_json = normalize_transcript(submission);
        let transcript_str = serde_json::to_string(&transcript_json).unwrap_or_default();

        // For judge-scored tasks, build an enriched transcript that includes
        // workspace file contents so the judge can evaluate actual output.
        // Keep the clean JSON for the Python grader (which needs valid JSON).
        let judge_transcript_str = if meta.grading_type == GradingType::LlmJudge
            || meta.grading_type == GradingType::Hybrid
        {
            let workspace_summary = summarize_workspace(&workspace_path);
            if workspace_summary.is_empty() {
                transcript_str.clone()
            } else {
                format!(
                    "{transcript_str}\n\n--- Workspace files created by the agent ---\n{workspace_summary}"
                )
            }
        } else {
            transcript_str.clone()
        };

        match meta.grading_type {
            GradingType::Automated => {
                let grade_code = extract_python_code(&content.automated_checks);
                if grade_code.is_empty() {
                    return Ok(BenchScore::fail("no automated grading code found"));
                }
                let scores =
                    run_python_grader(&grade_code, &transcript_str, &workspace_path).await?;
                Ok(scores_to_bench_score(&scores))
            }
            GradingType::LlmJudge => {
                let rubric = content.llm_judge_rubric.as_deref().unwrap_or("");
                if rubric.is_empty() {
                    return Ok(BenchScore::fail("no LLM judge rubric found"));
                }
                let scores = run_llm_judge(
                    &self.judge_model,
                    &content.prompt,
                    &content.expected_behavior,
                    &judge_transcript_str,
                    rubric,
                )
                .await?;
                Ok(scores_to_bench_score(&scores))
            }
            GradingType::Hybrid => {
                let mut auto_scores = HashMap::new();
                let grade_code = extract_python_code(&content.automated_checks);
                if !grade_code.is_empty() {
                    auto_scores =
                        run_python_grader(&grade_code, &transcript_str, &workspace_path).await?;
                }

                let mut judge_scores = HashMap::new();
                let rubric = content.llm_judge_rubric.as_deref().unwrap_or("");
                if !rubric.is_empty() {
                    judge_scores = run_llm_judge(
                        &self.judge_model,
                        &content.prompt,
                        &content.expected_behavior,
                        &transcript_str,
                        rubric,
                    )
                    .await?;
                }

                let auto_avg = avg_scores(&auto_scores);
                let judge_avg = avg_scores(&judge_scores);

                let w = self.hybrid_auto_weight;
                let combined = if auto_scores.is_empty() {
                    judge_avg
                } else if judge_scores.is_empty() {
                    auto_avg
                } else {
                    auto_avg * w + judge_avg * (1.0 - w)
                };

                let mut detail_parts = Vec::new();
                if !auto_scores.is_empty() {
                    detail_parts.push(format!("automated={auto_avg:.2}"));
                }
                if !judge_scores.is_empty() {
                    detail_parts.push(format!("judge={judge_avg:.2}"));
                }
                let detail = detail_parts.join(", ");

                if combined >= 0.95 {
                    Ok(BenchScore::pass())
                } else if combined <= 0.05 {
                    Ok(BenchScore::fail(detail))
                } else {
                    Ok(BenchScore::partial(combined, detail))
                }
            }
        }
    }

    async fn next_user_message(
        &self,
        task: &BenchTask,
        conversation: &[ConversationTurn],
    ) -> Result<Option<String>, BenchError> {
        let meta: PinchTaskMeta = serde_json::from_value(
            task.metadata.get("meta").cloned().unwrap_or_default(),
        )
        .map_err(|e| BenchError::Scoring {
            task_id: task.id.clone(),
            reason: format!("failed to parse task meta: {e}"),
        })?;

        if !meta.multi_session || meta.sessions.is_empty() {
            return Ok(None);
        }

        let user_turns_sent = conversation
            .iter()
            .filter(|t| matches!(t.role, TurnRole::User))
            .count();

        if user_turns_sent < meta.sessions.len() {
            Ok(Some(meta.sessions[user_turns_sent].prompt.clone()))
        } else {
            Ok(None)
        }
    }

    fn additional_tools(&self) -> Vec<Arc<dyn ironclaw::tools::Tool>> {
        vec![
            Arc::new(ironclaw::tools::builtin::ShellTool::new()),
            Arc::new(ironclaw::tools::builtin::ReadFileTool::new()),
            Arc::new(ironclaw::tools::builtin::WriteFileTool::new()),
            Arc::new(ironclaw::tools::builtin::ListDirTool::new()),
            Arc::new(ironclaw::tools::builtin::ApplyPatchTool::new()),
        ]
    }
}

// ---------------------------------------------------------------------------
// Frontmatter + markdown section parsing
// ---------------------------------------------------------------------------

/// Parse YAML frontmatter delimited by `---` lines.
fn parse_frontmatter(raw: &str) -> Result<(PinchTaskMeta, String), String> {
    let trimmed = raw.trim_start_matches('\u{feff}'); // strip BOM
    if !trimmed.starts_with("---") {
        return Err("file does not start with ---".to_string());
    }

    let after_first = &trimmed[3..];
    let end = after_first
        .find("\n---")
        .ok_or_else(|| "no closing --- for frontmatter".to_string())?;

    let yaml_str = &after_first[..end];
    let body = &after_first[end + 4..]; // skip "\n---"

    let meta: PinchTaskMeta =
        serde_yaml::from_str(yaml_str).map_err(|e| format!("YAML parse error: {e}"))?;

    Ok((meta, body.to_string()))
}

/// Extract sections from markdown body by `## ` headers.
fn parse_sections(body: &str) -> PinchTaskContent {
    let mut sections: HashMap<String, String> = HashMap::new();
    let mut current_header: Option<String> = None;
    let mut current_lines: Vec<&str> = Vec::new();

    for line in body.lines() {
        if let Some(header) = line.strip_prefix("## ") {
            if let Some(h) = current_header.take() {
                sections.insert(h, current_lines.join("\n").trim().to_string());
            }
            current_header = Some(header.trim().to_lowercase());
            current_lines.clear();
        } else {
            current_lines.push(line);
        }
    }
    if let Some(h) = current_header {
        sections.insert(h, current_lines.join("\n").trim().to_string());
    }

    PinchTaskContent {
        prompt: sections.remove("prompt").unwrap_or_default(),
        expected_behavior: sections.remove("expected behavior").unwrap_or_default(),
        automated_checks: sections.remove("automated checks"),
        llm_judge_rubric: sections.remove("llm judge rubric"),
    }
}

// ---------------------------------------------------------------------------
// Transcript normalization
// ---------------------------------------------------------------------------

/// Convert `TaskSubmission` into PinchBench's transcript format.
///
/// PinchBench graders expect:
/// ```json
/// [{"type": "message", "message": {"role": "assistant", "content": [...]}}]
/// ```
fn normalize_transcript(submission: &TaskSubmission) -> serde_json::Value {
    let mut entries = Vec::new();

    for turn in &submission.conversation {
        let role = match turn.role {
            TurnRole::User => "user",
            TurnRole::Assistant => "assistant",
            TurnRole::System => "system",
        };
        entries.push(serde_json::json!({
            "type": "message",
            "message": {
                "role": role,
                "content": [{"type": "text", "text": &turn.content}]
            }
        }));
    }

    // Append tool calls as assistant messages with toolCall content items.
    // Use trace_tool_calls (rich data with arguments) when available,
    // falling back to plain tool_calls (names only).
    if !submission.trace_tool_calls.is_empty() {
        for tc in &submission.trace_tool_calls {
            let mut item = serde_json::json!({
                "type": "toolCall",
                "name": &tc.name,
            });
            if let Some(ref args) = tc.arguments {
                item["params"] = args.clone();
            }
            entries.push(serde_json::json!({
                "type": "message",
                "message": {
                    "role": "assistant",
                    "content": [item],
                }
            }));
            // Append tool result if available
            if let Some(ref preview) = tc.result_preview {
                entries.push(serde_json::json!({
                    "type": "message",
                    "message": {
                        "role": "toolResult",
                        "content": [preview],
                    }
                }));
            }
        }
    } else if !submission.tool_calls.is_empty() {
        let tool_items: Vec<serde_json::Value> = submission
            .tool_calls
            .iter()
            .map(|name| {
                serde_json::json!({
                    "type": "toolCall",
                    "name": name,
                })
            })
            .collect();

        entries.push(serde_json::json!({
            "type": "message",
            "message": {
                "role": "assistant",
                "content": tool_items,
            }
        }));
    }

    // If conversation is empty, synthesize from the response.
    if submission.conversation.is_empty() && !submission.response.is_empty() {
        entries.push(serde_json::json!({
            "type": "message",
            "message": {
                "role": "assistant",
                "content": [{"type": "text", "text": &submission.response}]
            }
        }));
    }

    serde_json::Value::Array(entries)
}

// ---------------------------------------------------------------------------
// Python grading
// ---------------------------------------------------------------------------

/// Extract the Python code block from the Automated Checks section.
fn extract_python_code(section: &Option<String>) -> String {
    let Some(text) = section else {
        return String::new();
    };
    // Find ```python ... ``` block
    let start = text.find("```python");
    let Some(start) = start else {
        return String::new();
    };
    let after_marker = &text[start + 9..]; // skip "```python"
    let end = after_marker.find("```");
    let Some(end) = end else {
        return String::new();
    };
    after_marker[..end].trim().to_string()
}

/// Run a Python grade() function via `uv run`.
async fn run_python_grader(
    grade_code: &str,
    transcript_json: &str,
    workspace_path: &Path,
) -> Result<HashMap<String, f64>, BenchError> {
    let grade_file = tempfile::NamedTempFile::new()
        .map_err(|e| BenchError::PinchBench(format!("temp file: {e}")))?;
    std::fs::write(grade_file.path(), grade_code)?;

    let transcript_file = tempfile::NamedTempFile::new()
        .map_err(|e| BenchError::PinchBench(format!("temp file: {e}")))?;
    std::fs::write(transcript_file.path(), transcript_json)?;

    let output = tokio::process::Command::new("uv")
        .args([
            "run",
            "--no-project",
            "scripts/pinchbench_grade.py",
            "--grade-code",
            &grade_file.path().display().to_string(),
            "--transcript",
            &transcript_file.path().display().to_string(),
            "--workspace",
            &workspace_path.display().to_string(),
        ])
        .output()
        .await
        .map_err(|e| BenchError::PinchBench(format!("uv run failed: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(BenchError::PinchBench(format!(
            "grade() failed: {stderr}"
        )));
    }

    let scores: HashMap<String, f64> = serde_json::from_slice(&output.stdout).map_err(|e| {
        let stdout = String::from_utf8_lossy(&output.stdout);
        BenchError::PinchBench(format!("bad grade output: {e}\nraw: {stdout}"))
    })?;

    if let Some(err) = scores.get("_error") {
        return Err(BenchError::PinchBench(format!("grade() error: {err}")));
    }

    Ok(scores)
}

// ---------------------------------------------------------------------------
// LLM judge
// ---------------------------------------------------------------------------

/// Run the LLM judge via a direct OpenAI-compatible API call.
///
/// Design decision: the judge does not need tools, file access, or any agentic
/// behavior. It is a single prompt → structured JSON response. Using a direct
/// HTTP call is simpler and more portable than wiring through an agent framework.
async fn run_llm_judge(
    model: &str,
    task_prompt: &str,
    expected_behavior: &str,
    transcript_json: &str,
    rubric: &str,
) -> Result<HashMap<String, f64>, BenchError> {
    let judge_prompt = format!(
        "You are a grading function. Your ONLY job is to output a single JSON object.\n\n\
         CRITICAL RULES:\n\
         - Do NOT use any tools\n\
         - Do NOT create files or run commands\n\
         - Do NOT write any prose, explanation, or commentary outside the JSON\n\
         - Respond with ONLY a JSON object — nothing else\n\n\
         Be a strict evaluator. Reserve 1.0 for genuinely excellent performance. \
         An average acceptable completion should score around 0.6-0.7. \
         Deduct points for unnecessary steps, verbose output, and inefficient tool usage.\n\n\
         ## Task\n{task_prompt}\n\n\
         ## Expected Behavior\n{expected_behavior}\n\n\
         ## Agent Transcript\n{transcript_json}\n\n\
         ## Grading Rubric\n{rubric}\n\n\
         Score each criterion from 0.0 to 1.0.\n\
         The \"total\" field must be between 0.0 and 1.0, as the arithmetic mean of criterion scores.\n\n\
         Respond with ONLY this JSON structure (no markdown, no code fences, no extra text):\n\
         {{\"scores\": {{\"criterion_name\": 0.0}}, \"total\": 0.0, \"notes\": \"brief justification\"}}"
    );

    let (base_url, api_key, api_model) = resolve_judge_endpoint(model)?;

    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "model": api_model,
        "messages": [{"role": "user", "content": judge_prompt}],
        "temperature": 0.0,
        "max_tokens": 2000,
    });

    let resp = client
        .post(format!("{base_url}/chat/completions"))
        .header("Authorization", format!("Bearer {api_key}"))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| BenchError::PinchBench(format!("judge API request failed: {e}")))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(BenchError::PinchBench(format!(
            "judge API returned {status}: {body}"
        )));
    }

    let resp_json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| BenchError::PinchBench(format!("judge API bad response: {e}")))?;

    let content = resp_json["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("");

    parse_judge_response(content)
}

/// Resolve the API base URL, key, and model ID for the judge.
///
/// The model string may have a provider prefix (e.g. `openrouter/z-ai/glm-5`)
/// which selects the API endpoint. The prefix is stripped from the model ID
/// sent to the API.
fn resolve_judge_endpoint(model: &str) -> Result<(String, String, String), BenchError> {
    if let Some(api_model) = model.strip_prefix("openrouter/") {
        let key = std::env::var("OPENROUTER_API_KEY")
            .or_else(|_| std::env::var("OPENAI_API_KEY"))
            .map_err(|_| {
                BenchError::PinchBench(
                    "OPENROUTER_API_KEY or OPENAI_API_KEY required for judge model".to_string(),
                )
            })?;
        Ok((
            "https://openrouter.ai/api/v1".to_string(),
            key,
            api_model.to_string(),
        ))
    } else if let Some(api_model) = model.strip_prefix("anthropic/") {
        let key = std::env::var("ANTHROPIC_API_KEY").map_err(|_| {
            BenchError::PinchBench("ANTHROPIC_API_KEY required for judge model".to_string())
        })?;
        Ok((
            "https://api.anthropic.com/v1".to_string(),
            key,
            api_model.to_string(),
        ))
    } else {
        let base = std::env::var("LLM_BASE_URL")
            .unwrap_or_else(|_| "https://api.openai.com/v1".to_string());
        let key = std::env::var("OPENAI_API_KEY")
            .or_else(|_| std::env::var("LLM_API_KEY"))
            .map_err(|_| {
                BenchError::PinchBench("OPENAI_API_KEY or LLM_API_KEY required for judge".to_string())
            })?;
        Ok((base, key, model.to_string()))
    }
}

/// Parse the judge's JSON response, handling common variations.
fn parse_judge_response(content: &str) -> Result<HashMap<String, f64>, BenchError> {
    let content = content.trim();

    // Strip markdown code fences if present
    let json_str = content
        .strip_prefix("```json")
        .or_else(|| content.strip_prefix("```"))
        .unwrap_or(content);
    let json_str = json_str
        .strip_suffix("```")
        .unwrap_or(json_str)
        .trim();

    let parsed: serde_json::Value = serde_json::from_str(json_str).map_err(|e| {
        BenchError::PinchBench(format!(
            "judge returned bad JSON: {e}\nraw: {content}"
        ))
    })?;

    let mut scores = HashMap::new();

    // Extract from "scores" or "criteria_scores"
    let scores_obj = parsed
        .get("scores")
        .or_else(|| parsed.get("criteria_scores"));

    if let Some(obj) = scores_obj.and_then(|v| v.as_object()) {
        for (k, v) in obj {
            if let Some(score) = v.as_f64() {
                scores.insert(k.clone(), score.clamp(0.0, 1.0));
            } else if let Some(inner) = v.get("score").and_then(|s| s.as_f64()) {
                scores.insert(k.clone(), inner.clamp(0.0, 1.0));
            }
        }
    }

    // If we got a "total" directly but no individual scores, use that.
    if scores.is_empty() {
        if let Some(total) = parsed
            .get("total")
            .or_else(|| parsed.get("score"))
            .or_else(|| parsed.get("overall_score"))
            .and_then(|v| v.as_f64())
        {
            scores.insert("total".to_string(), total.clamp(0.0, 1.0));
        }
    }

    if scores.is_empty() {
        return Err(BenchError::PinchBench(format!(
            "judge response contained no scores: {content}"
        )));
    }

    Ok(scores)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Summarize text files in the workspace for the LLM judge.
/// Reads all non-binary files, truncating large ones, so the judge can
/// evaluate the agent's actual output rather than just its response text.
fn summarize_workspace(workspace_path: &Path) -> String {
    if !workspace_path.is_dir() {
        return String::new();
    }
    let mut parts = Vec::new();
    let skip_prefixes = ["SOUL.md", "IDENTITY.md", "AGENTS.md", "TOOLS.md",
                          "KNOWLEDGE.md", "USER.md", "HEARTBEAT.md", "BOOTSTRAP.md",
                          "README.md"];
    if let Ok(entries) = std::fs::read_dir(workspace_path) {
        let mut files: Vec<_> = entries.flatten().map(|e| e.path()).collect();
        files.sort();
        for path in &files {
            if path.is_dir() {
                continue;
            }
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            // Skip identity/bootstrap files and hidden files
            if name.starts_with('.') || skip_prefixes.iter().any(|s| *s == name.as_ref()) {
                continue;
            }
            // Skip binary files
            let ext = path.extension().unwrap_or_default().to_string_lossy();
            if ["pdf", "xlsx", "xls", "png", "jpg", "gif", "zip", "tar", "gz"].contains(&ext.as_ref()) {
                parts.push(format!("### {name}\n(binary file, {:.1}KB)",
                    std::fs::metadata(path).map(|m| m.len() as f64 / 1024.0).unwrap_or(0.0)));
                continue;
            }
            if let Ok(content) = std::fs::read_to_string(path) {
                let truncated = if content.len() > 2000 {
                    format!("{}... (truncated, {} chars total)", &content[..2000], content.len())
                } else {
                    content
                };
                parts.push(format!("### {name}\n```\n{truncated}\n```"));
            }
        }
    }
    // Also check subdirectories one level deep
    if let Ok(entries) = std::fs::read_dir(workspace_path) {
        for entry in entries.flatten() {
            let subdir = entry.path();
            if !subdir.is_dir() || subdir.file_name().unwrap_or_default().to_string_lossy().starts_with('.') {
                continue;
            }
            if let Ok(sub_entries) = std::fs::read_dir(&subdir) {
                for sub_entry in sub_entries.flatten() {
                    let path = sub_entry.path();
                    if path.is_dir() { continue; }
                    let name = format!("{}/{}",
                        subdir.file_name().unwrap_or_default().to_string_lossy(),
                        path.file_name().unwrap_or_default().to_string_lossy());
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        let truncated = if content.len() > 1000 {
                            format!("{}... (truncated)", &content[..1000])
                        } else {
                            content
                        };
                        parts.push(format!("### {name}\n```\n{truncated}\n```"));
                    }
                }
            }
        }
    }
    parts.join("\n\n")
}

fn avg_scores(scores: &HashMap<String, f64>) -> f64 {
    if scores.is_empty() {
        return 0.0;
    }
    let sum: f64 = scores.values().sum();
    sum / scores.len() as f64
}

fn scores_to_bench_score(scores: &HashMap<String, f64>) -> BenchScore {
    if scores.is_empty() {
        return BenchScore::fail("no scores returned");
    }
    let avg = avg_scores(scores);
    let detail: String = scores
        .iter()
        .map(|(k, v)| format!("{k}={v:.2}"))
        .collect::<Vec<_>>()
        .join(", ");

    if avg >= 0.95 {
        BenchScore::pass()
    } else if avg <= 0.05 {
        BenchScore::fail(detail)
    } else {
        BenchScore::partial(avg, detail)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_TASK: &str = r#"---
id: task_00_sanity
name: Sanity Check
category: basic
grading_type: automated
timeout_seconds: 60
workspace_files: []
---

## Prompt

Say "Hello, I'm ready!" to confirm you can respond.

## Expected Behavior

The agent should respond with a greeting.

## Grading Criteria

- [ ] Agent responded successfully

## Automated Checks

```python
def grade(transcript: list, workspace_path: str) -> dict:
    has_response = False
    for entry in transcript:
        if entry.get("type") == "message":
            message = entry.get("message", {})
            if message.get("role") == "assistant":
                content = message.get("content", [])
                if content and len(content) > 0:
                    has_response = True
                    break
    return {"agent_responded": 1.0 if has_response else 0.0}
```
"#;

    const SAMPLE_HYBRID_TASK: &str = r#"---
id: task_10_workflow
name: Multi-step API Workflow
category: complex
grading_type: hybrid
timeout_seconds: 300
workspace_files:
  - path: "config.json"
    content: |
      {"api": {"endpoint": "https://api.example.com"}}
---

## Prompt

Read config.json and create a Python script.

## Expected Behavior

The agent should read config and write a script.

## Automated Checks

```python
def grade(transcript: list, workspace_path: str) -> dict:
    from pathlib import Path
    ws = Path(workspace_path)
    return {"script_exists": 1.0 if list(ws.glob("*.py")) else 0.0}
```

## LLM Judge Rubric

### Criterion 1: Script Quality (Weight: 50%)
**Score 1.0**: Excellent script.
**Score 0.0**: No script.

### Criterion 2: Documentation (Weight: 50%)
**Score 1.0**: Excellent docs.
**Score 0.0**: No docs.
"#;

    const SAMPLE_MULTI_SESSION: &str = r#"---
id: task_22_second_brain
name: Second Brain
category: memory
grading_type: hybrid
timeout_seconds: 300
multi_session: true
sessions:
  - id: store
    prompt: "Save this info to memory/MEMORY.md: my name is Alice."
  - id: recall
    prompt: "What is my name? Check memory/MEMORY.md."
workspace_files: []
---

## Prompt

This is a multi-session task.

## Expected Behavior

Agent stores and recalls.
"#;

    #[test]
    fn test_parse_frontmatter() {
        let (meta, body) = parse_frontmatter(SAMPLE_TASK).unwrap();
        assert_eq!(meta.id, "task_00_sanity");
        assert_eq!(meta.name, "Sanity Check");
        assert_eq!(meta.category, "basic");
        assert_eq!(meta.grading_type, GradingType::Automated);
        assert_eq!(meta.timeout_seconds, 60);
        assert!(meta.workspace_files.is_empty());
        assert!(body.contains("## Prompt"));
    }

    #[test]
    fn test_parse_frontmatter_hybrid_with_workspace_files() {
        let (meta, _body) = parse_frontmatter(SAMPLE_HYBRID_TASK).unwrap();
        assert_eq!(meta.id, "task_10_workflow");
        assert_eq!(meta.grading_type, GradingType::Hybrid);
        assert_eq!(meta.workspace_files.len(), 1);
        assert_eq!(
            meta.workspace_files[0].path.as_deref(),
            Some("config.json")
        );
        assert!(meta.workspace_files[0].content.is_some());
    }

    #[test]
    fn test_parse_frontmatter_multi_session() {
        let (meta, _body) = parse_frontmatter(SAMPLE_MULTI_SESSION).unwrap();
        assert!(meta.multi_session);
        assert_eq!(meta.sessions.len(), 2);
        assert_eq!(meta.sessions[0].id, "store");
        assert!(meta.sessions[0].prompt.contains("Alice"));
    }

    #[test]
    fn test_parse_sections() {
        let (_, body) = parse_frontmatter(SAMPLE_TASK).unwrap();
        let content = parse_sections(&body);
        assert!(content.prompt.contains("Hello, I'm ready!"));
        assert!(content.expected_behavior.contains("greeting"));
        assert!(content.automated_checks.is_some());
        assert!(content.llm_judge_rubric.is_none());
    }

    #[test]
    fn test_parse_sections_hybrid() {
        let (_, body) = parse_frontmatter(SAMPLE_HYBRID_TASK).unwrap();
        let content = parse_sections(&body);
        assert!(content.automated_checks.is_some());
        assert!(content.llm_judge_rubric.is_some());
        assert!(content.llm_judge_rubric.as_ref().unwrap().contains("Script Quality"));
    }

    #[test]
    fn test_extract_python_code() {
        let section = Some("Some text\n```python\ndef grade(t, w):\n    return {\"ok\": 1.0}\n```\nmore text".to_string());
        let code = extract_python_code(&section);
        assert!(code.contains("def grade"));
        assert!(code.contains("return"));
    }

    #[test]
    fn test_extract_python_code_none() {
        assert!(extract_python_code(&None).is_empty());
        assert!(extract_python_code(&Some("no code block".to_string())).is_empty());
    }

    #[test]
    fn test_normalize_transcript() {
        let submission = TaskSubmission {
            response: "Hello!".to_string(),
            conversation: vec![
                ConversationTurn {
                    role: TurnRole::User,
                    content: "Hi".to_string(),
                },
                ConversationTurn {
                    role: TurnRole::Assistant,
                    content: "Hello!".to_string(),
                },
            ],
            tool_calls: vec!["read_file".to_string()],
            trace_tool_calls: vec![],
            error: None,
        };
        let transcript = normalize_transcript(&submission);
        let arr = transcript.as_array().unwrap();
        assert_eq!(arr.len(), 3); // user, assistant, tool calls
        assert_eq!(arr[0]["message"]["role"], "user");
        assert_eq!(arr[1]["message"]["role"], "assistant");
        assert_eq!(arr[2]["message"]["content"][0]["type"], "toolCall");
    }

    #[test]
    fn test_normalize_transcript_empty_conversation() {
        let submission = TaskSubmission {
            response: "Hello!".to_string(),
            conversation: vec![],
            tool_calls: vec![],
            trace_tool_calls: vec![],
            error: None,
        };
        let transcript = normalize_transcript(&submission);
        let arr = transcript.as_array().unwrap();
        assert_eq!(arr.len(), 1); // synthesized from response
        assert_eq!(arr[0]["message"]["role"], "assistant");
    }

    #[test]
    fn test_parse_judge_response_standard() {
        let json = r#"{"scores": {"quality": 0.8, "completeness": 0.6}, "total": 0.7, "notes": "ok"}"#;
        let scores = parse_judge_response(json).unwrap();
        assert_eq!(scores["quality"], 0.8);
        assert_eq!(scores["completeness"], 0.6);
    }

    #[test]
    fn test_parse_judge_response_code_fenced() {
        let json = "```json\n{\"scores\": {\"x\": 0.9}, \"total\": 0.9}\n```";
        let scores = parse_judge_response(json).unwrap();
        assert_eq!(scores["x"], 0.9);
    }

    #[test]
    fn test_parse_judge_response_total_only() {
        let json = r#"{"total": 0.75}"#;
        let scores = parse_judge_response(json).unwrap();
        assert_eq!(scores["total"], 0.75);
    }

    #[test]
    fn test_parse_judge_response_criteria_scores_alternate() {
        let json = r#"{"criteria_scores": {"a": {"score": 0.5}, "b": 0.8}, "total": 0.65}"#;
        let scores = parse_judge_response(json).unwrap();
        assert_eq!(scores["a"], 0.5);
        assert_eq!(scores["b"], 0.8);
    }

    #[test]
    fn test_scores_to_bench_score_pass() {
        let mut scores = HashMap::new();
        scores.insert("a".to_string(), 1.0);
        scores.insert("b".to_string(), 1.0);
        let bench = scores_to_bench_score(&scores);
        assert_eq!(bench.label, "pass");
        assert_eq!(bench.value, 1.0);
    }

    #[test]
    fn test_scores_to_bench_score_fail() {
        let mut scores = HashMap::new();
        scores.insert("a".to_string(), 0.0);
        let bench = scores_to_bench_score(&scores);
        assert_eq!(bench.label, "fail");
    }

    #[test]
    fn test_scores_to_bench_score_partial() {
        let mut scores = HashMap::new();
        scores.insert("a".to_string(), 0.5);
        scores.insert("b".to_string(), 0.8);
        let bench = scores_to_bench_score(&scores);
        assert_eq!(bench.label, "partial");
        assert!((bench.value - 0.65).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_load_tasks_from_directory() {
        let dir = tempfile::tempdir().unwrap();
        let tasks_dir = dir.path().join("tasks");
        std::fs::create_dir_all(&tasks_dir).unwrap();

        std::fs::write(tasks_dir.join("task_00_test.md"), SAMPLE_TASK).unwrap();
        std::fs::write(tasks_dir.join("task_01_hybrid.md"), SAMPLE_HYBRID_TASK).unwrap();

        let suite = PinchBenchSuite::new(dir.path(), "test-model".to_string(), 0.6);
        let tasks = suite.load_tasks().await.unwrap();
        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].id, "task_00_sanity");
        assert_eq!(tasks[1].id, "task_10_workflow");
        assert!(tasks[0].prompt.contains("Hello, I'm ready!"));
        assert!(tasks[0].tags.contains(&"automated".to_string()));
        assert!(tasks[1].tags.contains(&"hybrid".to_string()));
    }

    #[tokio::test]
    async fn test_load_multi_session_task() {
        let dir = tempfile::tempdir().unwrap();
        let tasks_dir = dir.path().join("tasks");
        std::fs::create_dir_all(&tasks_dir).unwrap();

        std::fs::write(tasks_dir.join("task_22_brain.md"), SAMPLE_MULTI_SESSION).unwrap();

        let suite = PinchBenchSuite::new(dir.path(), "test-model".to_string(), 0.6);
        let tasks = suite.load_tasks().await.unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].expected_turns, Some(2));
        assert!(tasks[0].prompt.contains("Alice"));
    }

    #[tokio::test]
    async fn test_setup_and_teardown_workspace() {
        let dir = tempfile::tempdir().unwrap();
        let tasks_dir = dir.path().join("tasks");
        let assets_dir = dir.path().join("assets");
        std::fs::create_dir_all(&tasks_dir).unwrap();
        std::fs::create_dir_all(&assets_dir).unwrap();

        // Create an asset file
        std::fs::write(assets_dir.join("test.txt"), "asset content").unwrap();

        let task_md = r#"---
id: task_ws_test
name: Workspace Test
category: test
grading_type: automated
timeout_seconds: 60
workspace_files:
  - path: "inline.txt"
    content: "hello inline"
  - source: "test.txt"
    dest: "copied.txt"
---

## Prompt

Test.

## Expected Behavior

Test.
"#;
        std::fs::write(tasks_dir.join("task_99_test.md"), task_md).unwrap();

        let suite = PinchBenchSuite::new(dir.path(), "test-model".to_string(), 0.6);
        let tasks = suite.load_tasks().await.unwrap();
        assert_eq!(tasks.len(), 1);

        suite.setup_task(&tasks[0]).await.unwrap();

        // Check workspace was created with files
        let ws_path = suite.workspace_path_for("task_ws_test");
        assert!(ws_path.join("inline.txt").exists());
        assert_eq!(
            std::fs::read_to_string(ws_path.join("inline.txt")).unwrap(),
            "hello inline"
        );
        assert!(ws_path.join("copied.txt").exists());
        assert_eq!(
            std::fs::read_to_string(ws_path.join("copied.txt")).unwrap(),
            "asset content"
        );

        // Teardown is a no-op (scoring happens after teardown in runner)
        suite.teardown_task(&tasks[0]).await.unwrap();
        // Workspace still exists (cleaned on next setup_task)
        assert!(ws_path.exists());
        // Manual cleanup
        std::fs::remove_dir_all(&ws_path).ok();
    }

    /// Validate all 23 real PinchBench tasks parse correctly.
    /// Ignored when dataset isn't downloaded.
    #[tokio::test]
    #[ignore]
    async fn test_load_real_pinchbench_tasks() {
        let dataset_path = PathBuf::from("datasets/pinchbench/v1");
        if !dataset_path.join("tasks").is_dir() {
            eprintln!("Skipping: datasets/pinchbench/v1/tasks not found. Run scripts/pinchbench_download.sh first.");
            return;
        }
        let suite = PinchBenchSuite::new(&dataset_path, "test".to_string(), 0.6);
        let tasks = suite.load_tasks().await.unwrap();
        assert!(tasks.len() >= 23, "Expected 23+ tasks, got {}", tasks.len());

        for task in &tasks {
            assert!(!task.id.is_empty(), "Task has empty id");
            assert!(!task.prompt.is_empty(), "Task {} has empty prompt", task.id);
            assert!(task.timeout.is_some(), "Task {} has no timeout", task.id);

            // Verify metadata round-trips
            let meta: PinchTaskMeta = serde_json::from_value(
                task.metadata.get("meta").cloned().unwrap(),
            )
            .unwrap_or_else(|e| panic!("Task {} meta failed: {e}", task.id));

            let content: PinchTaskContent = serde_json::from_value(
                task.metadata.get("content").cloned().unwrap(),
            )
            .unwrap_or_else(|e| panic!("Task {} content failed: {e}", task.id));

            // Automated and hybrid tasks must have grade code
            if meta.grading_type == GradingType::Automated || meta.grading_type == GradingType::Hybrid {
                let code = extract_python_code(&content.automated_checks);
                assert!(
                    !code.is_empty(),
                    "Task {} is {:?} but has no Python grade code",
                    task.id,
                    meta.grading_type
                );
            }

            // LLM judge and hybrid tasks must have rubric
            if meta.grading_type == GradingType::LlmJudge || meta.grading_type == GradingType::Hybrid {
                assert!(
                    content.llm_judge_rubric.is_some(),
                    "Task {} is {:?} but has no LLM judge rubric",
                    task.id,
                    meta.grading_type
                );
            }
        }

        // Print summary
        let automated = tasks.iter().filter(|t| t.tags.contains(&"automated".to_string())).count();
        let llm_judge = tasks.iter().filter(|t| t.tags.contains(&"llm_judge".to_string())).count();
        let hybrid = tasks.iter().filter(|t| t.tags.contains(&"hybrid".to_string())).count();
        eprintln!("Loaded {} tasks: {} automated, {} llm_judge, {} hybrid",
            tasks.len(), automated, llm_judge, hybrid);
    }

    #[tokio::test]
    async fn test_next_user_message_multi_session() {
        let dir = tempfile::tempdir().unwrap();
        let tasks_dir = dir.path().join("tasks");
        std::fs::create_dir_all(&tasks_dir).unwrap();
        std::fs::write(tasks_dir.join("task_22_brain.md"), SAMPLE_MULTI_SESSION).unwrap();

        let suite = PinchBenchSuite::new(dir.path(), "test-model".to_string(), 0.6);
        let tasks = suite.load_tasks().await.unwrap();

        // No conversation yet -> first session prompt
        let msg = suite.next_user_message(&tasks[0], &[]).await.unwrap();
        assert!(msg.as_ref().unwrap().contains("Alice"));

        // After one user turn -> second session prompt
        let conv = vec![ConversationTurn {
            role: TurnRole::User,
            content: "first".to_string(),
        }];
        let msg = suite.next_user_message(&tasks[0], &conv).await.unwrap();
        assert!(msg.as_ref().unwrap().contains("What is my name"));

        // After two user turns -> done
        let conv = vec![
            ConversationTurn {
                role: TurnRole::User,
                content: "first".to_string(),
            },
            ConversationTurn {
                role: TurnRole::User,
                content: "second".to_string(),
            },
        ];
        let msg = suite.next_user_message(&tasks[0], &conv).await.unwrap();
        assert!(msg.is_none());
    }
}
