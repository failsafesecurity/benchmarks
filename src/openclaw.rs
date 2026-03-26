use std::collections::HashMap;
use std::io::Write;
use std::process::Command;
use std::time::{Duration, Instant};

use chrono::Utc;

use crate::config::BenchConfig;
use crate::error::BenchError;
use crate::results::{TaskResult, Trace};
use crate::suite::{BenchScore, BenchTask};

/// Runs benchmark tasks against an OpenClaw gateway Docker container.
pub struct OpenClawRunner {
    image: String,
    gateway_token: String,
    model: Option<String>,
    http: reqwest::Client,
}

/// Handle to a running OpenClaw container. Stops the container on drop.
pub struct ContainerHandle {
    container_id: String,
    port: u16,
    _workspace_dir: tempfile::TempDir,
    _config_dir: tempfile::TempDir,
}

impl Drop for ContainerHandle {
    fn drop(&mut self) {
        let _ = Command::new("docker")
            .args(["rm", "-f", &self.container_id])
            .output();
    }
}

impl OpenClawRunner {
    pub fn new(config: &BenchConfig, model: Option<String>) -> Result<Self, BenchError> {
        let image = config
            .openclaw
            .as_ref()
            .map(|oc| oc.image.clone())
            .unwrap_or_else(|| "openclaw:local".to_string());

        let gateway_token = config
            .openclaw
            .as_ref()
            .and_then(|oc| oc.gateway_token.clone())
            .or_else(|| std::env::var("OPENCLAW_GATEWAY_TOKEN").ok())
            .unwrap_or_else(|| "bench-token".to_string());

        Ok(Self {
            image,
            gateway_token,
            model,
            http: reqwest::Client::new(),
        })
    }

    /// Start an OpenClaw container with identity files from the task.
    pub async fn start_container(&self, task: &BenchTask) -> Result<ContainerHandle, BenchError> {
        let workspace_dir = tempfile::tempdir()
            .map_err(|e| BenchError::OpenClaw(format!("failed to create temp dir: {e}")))?;

        let identity: HashMap<String, String> = task
            .metadata
            .get("setup")
            .and_then(|s| s.get("identity"))
            .and_then(|i| serde_json::from_value(i.clone()).ok())
            .unwrap_or_default();

        for (filename, content) in &identity {
            let path = workspace_dir.path().join(filename);
            let mut file = std::fs::File::create(&path)
                .map_err(|e| BenchError::OpenClaw(format!("failed to create {filename}: {e}")))?;
            file.write_all(content.as_bytes())
                .map_err(|e| BenchError::OpenClaw(format!("failed to write {filename}: {e}")))?;
        }

        let config_dir = tempfile::tempdir()
            .map_err(|e| BenchError::OpenClaw(format!("failed to create config dir: {e}")))?;

        let mut oc_config = serde_json::json!({
            "gateway": {
                "auth": {
                    "mode": "token",
                    "token": &self.gateway_token
                },
                "http": {
                    "endpoints": {
                        "chatCompletions": { "enabled": true }
                    }
                },
                "controlUi": {
                    "dangerouslyAllowHostHeaderOriginFallback": true
                }
            }
        });

        if let Some(ref model) = self.model {
            oc_config["agents"] = serde_json::json!({
                "defaults": { "model": model }
            });
        }

        let config_path = config_dir.path().join("openclaw.json");
        std::fs::write(&config_path, serde_json::to_string_pretty(&oc_config).unwrap())
            .map_err(|e| BenchError::OpenClaw(format!("failed to write openclaw.json: {e}")))?;

        let agent_dir = config_dir.path().join("agents/main/agent");
        std::fs::create_dir_all(&agent_dir)
            .map_err(|e| BenchError::OpenClaw(format!("failed to create agent dir: {e}")))?;

        let mut profiles = serde_json::Map::new();
        let api_key_sources: &[(&str, &str)] = &[
            ("ANTHROPIC_API_KEY", "anthropic"),
            ("OPENAI_API_KEY", "openai"),
            ("OPENROUTER_API_KEY", "openrouter"),
            ("GEMINI_API_KEY", "gemini"),
            ("LLM_API_KEY", "openrouter"), // fallback: LLM_API_KEY often targets openrouter
        ];
        for (env_var, provider) in api_key_sources {
            if let Ok(key) = std::env::var(env_var) {
                if !key.is_empty() && !profiles.contains_key(*provider) {
                    profiles.insert(
                        format!("{provider}-bench"),
                        serde_json::json!({
                            "type": "api_key",
                            "provider": provider,
                            "key": key
                        }),
                    );
                }
            }
        }

        let auth_store = serde_json::json!({
            "version": 1,
            "profiles": profiles
        });
        let auth_path = agent_dir.join("auth-profiles.json");
        std::fs::write(&auth_path, serde_json::to_string_pretty(&auth_store).unwrap())
            .map_err(|e| BenchError::OpenClaw(format!("failed to write auth-profiles.json: {e}")))?;

        let forwarded_env_vars = [
            "OPENAI_API_KEY",
            "ANTHROPIC_API_KEY",
            "GEMINI_API_KEY",
            "OPENROUTER_API_KEY",
        ];

        let mut docker_args = vec![
            "run".to_string(),
            "-d".to_string(),
            "--rm".to_string(),
            "-p".to_string(),
            "0:18789".to_string(),
            "-v".to_string(),
            format!("{}:/home/node/.openclaw", config_dir.path().display()),
            "-v".to_string(),
            format!(
                "{}:/home/node/.openclaw/workspace",
                workspace_dir.path().display()
            ),
            "-e".to_string(),
            format!("OPENCLAW_GATEWAY_TOKEN={}", self.gateway_token),
        ];

        for var in &forwarded_env_vars {
            if let Ok(val) = std::env::var(var) {
                docker_args.push("-e".to_string());
                docker_args.push(format!("{var}={val}"));
            }
        }

        docker_args.extend([
            self.image.clone(),
            "node".to_string(),
            "dist/index.js".to_string(),
            "gateway".to_string(),
            "--bind".to_string(),
            "lan".to_string(),
            "--port".to_string(),
            "18789".to_string(),
            "--allow-unconfigured".to_string(),
        ]);

        let output = Command::new("docker")
            .args(&docker_args)
            .output()
            .map_err(|e| BenchError::OpenClaw(format!("failed to run docker: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(BenchError::OpenClaw(format!(
                "docker run failed: {stderr}"
            )));
        }

        let container_id = String::from_utf8_lossy(&output.stdout).trim().to_string();

        let port_output = Command::new("docker")
            .args(["port", &container_id, "18789/tcp"])
            .output()
            .map_err(|e| BenchError::OpenClaw(format!("docker port failed: {e}")))?;

        let port_str = String::from_utf8_lossy(&port_output.stdout);
        let port = parse_docker_port(&port_str).ok_or_else(|| {
            // Clean up container on port parse failure
            let _ = Command::new("docker")
                .args(["rm", "-f", &container_id])
                .output();
            BenchError::OpenClaw(format!("failed to parse port from: {port_str}"))
        })?;

        let handle = ContainerHandle {
            container_id,
            port,
            _workspace_dir: workspace_dir,
            _config_dir: config_dir,
        };

        let deadline = Instant::now() + Duration::from_secs(30);
        let url = format!("http://127.0.0.1:{}/healthz", handle.port);

        loop {
            if Instant::now() > deadline {
                return Err(BenchError::OpenClaw(
                    "container health check timed out after 30s".to_string(),
                ));
            }

            match self.http.get(&url).send().await {
                Ok(resp) if resp.status().is_success() => break,
                _ => tokio::time::sleep(Duration::from_millis(500)).await,
            }
        }

        tracing::debug!(
            "OpenClaw container {} ready on port {}",
            handle.container_id,
            handle.port
        );

        Ok(handle)
    }

    /// Send a chat completion request to the running container.
    pub async fn chat_completion(
        &self,
        handle: &ContainerHandle,
        prompt: &str,
        timeout: Duration,
    ) -> Result<OpenClawResponse, BenchError> {
        let url = format!(
            "http://127.0.0.1:{}/v1/chat/completions",
            handle.port
        );

        let mut body = serde_json::json!({
            "messages": [{"role": "user", "content": prompt}],
            "stream": false
        });

        if let Some(ref model) = self.model {
            body["model"] = serde_json::Value::String(model.clone());
        }

        let resp = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.gateway_token))
            .header("Content-Type", "application/json")
            .timeout(timeout)
            .json(&body)
            .send()
            .await
            .map_err(|e| BenchError::OpenClaw(format!("chat completion request failed: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(BenchError::OpenClaw(format!(
                "chat completion returned {status}: {body}"
            )));
        }

        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| BenchError::OpenClaw(format!("failed to parse response: {e}")))?;

        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let prompt_tokens = json["usage"]["prompt_tokens"].as_u64().unwrap_or(0) as u32;
        let completion_tokens = json["usage"]["completion_tokens"].as_u64().unwrap_or(0) as u32;

        Ok(OpenClawResponse {
            content,
            prompt_tokens,
            completion_tokens,
        })
    }
}

pub struct OpenClawResponse {
    pub content: String,
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
}

/// Run a single task against an OpenClaw container.
pub async fn run_task_openclaw(
    task: &BenchTask,
    suite_id: &str,
    config_label: &str,
    timeout: Duration,
    runner: &OpenClawRunner,
) -> TaskResult {
    let started_at = Utc::now();
    let start = Instant::now();

    let handle = match runner.start_container(task).await {
        Ok(h) => h,
        Err(e) => {
            return make_error_result(task, suite_id, config_label, started_at, &e.to_string());
        }
    };

    let full_prompt = if let Some(ref ctx) = task.context {
        format!("{}\n\nContext:\n{}", task.prompt, ctx)
    } else {
        task.prompt.clone()
    };

    let result = match runner.chat_completion(&handle, &full_prompt, timeout).await {
        Ok(r) => r,
        Err(e) => {
            return make_error_result(task, suite_id, config_label, started_at, &e.to_string());
        }
    };

    let wall_time = start.elapsed();

    TaskResult {
        task_id: task.id.clone(),
        suite_id: suite_id.to_string(),
        score: BenchScore {
            value: 0.0,
            label: "pending".to_string(),
            details: None,
        },
        trace: Trace {
            wall_time_ms: wall_time.as_millis() as u64,
            llm_calls: 1,
            input_tokens: result.prompt_tokens,
            output_tokens: result.completion_tokens,
            estimated_cost_usd: 0.0,
            tool_calls: vec![],
            turns: 1,
            hit_iteration_limit: false,
            hit_timeout: false,
        },
        response: result.content,
        started_at,
        finished_at: Utc::now(),
        config_label: config_label.to_string(),
        error: None,
    }
}

use crate::runner::make_error_result;

/// Parse host port from `docker port` output like "0.0.0.0:32768" or ":::32768".
fn parse_docker_port(output: &str) -> Option<u16> {
    output
        .trim()
        .lines()
        .next()?
        .rsplit(':')
        .next()?
        .trim()
        .parse()
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_docker_port_ipv4() {
        assert_eq!(parse_docker_port("0.0.0.0:32768\n"), Some(32768));
    }

    #[test]
    fn test_parse_docker_port_ipv6() {
        assert_eq!(parse_docker_port(":::32768\n"), Some(32768));
    }

    #[test]
    fn test_parse_docker_port_multiline() {
        assert_eq!(
            parse_docker_port("0.0.0.0:32768\n:::32768\n"),
            Some(32768)
        );
    }

    #[test]
    fn test_parse_docker_port_empty() {
        assert_eq!(parse_docker_port(""), None);
    }
}
