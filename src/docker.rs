//! Shared Docker container management utilities.
//!
//! Provides helpers for building images, starting/stopping containers,
//! and executing commands inside containers. Used by the Terminal Bench
//! adapter and potentially other container-based suites.

use std::process::Stdio;
use std::time::Duration;

use crate::error::BenchError;

/// Output from executing a command inside a container.
#[derive(Debug)]
pub struct ExecOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

/// Build a Docker image from a Dockerfile.
///
/// Skips the build if the image already exists and `force_rebuild` is false.
pub async fn build_image(
    tag: &str,
    dockerfile: &std::path::Path,
    context: &std::path::Path,
    force_rebuild: bool,
) -> Result<(), BenchError> {
    if !force_rebuild && image_exists(tag).await {
        tracing::debug!("Docker image {} already exists, skipping build", tag);
        return Ok(());
    }

    tracing::info!("Building Docker image: {}", tag);
    let output = tokio::process::Command::new("docker")
        .args([
            "build",
            "-t",
            tag,
            "-f",
            &dockerfile.to_string_lossy(),
            &context.to_string_lossy(),
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| BenchError::Docker(format!("failed to run docker build: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(BenchError::Docker(format!(
            "docker build failed for {tag}: {stderr}"
        )));
    }

    Ok(())
}

/// Start a Docker container and return its container ID.
pub async fn start_container(
    image: &str,
    name: &str,
    cpus: Option<f64>,
    memory_mb: Option<u64>,
    env_vars: &[(String, String)],
    volumes: &[(String, String)],
) -> Result<String, BenchError> {
    let mut args = vec![
        "run".to_string(),
        "-d".to_string(),
        "--name".to_string(),
        name.to_string(),
    ];

    if let Some(cpus) = cpus {
        args.push(format!("--cpus={cpus}"));
    }
    if let Some(mem) = memory_mb {
        args.push(format!("--memory={mem}m"));
    }
    for (key, val) in env_vars {
        args.push("-e".to_string());
        args.push(format!("{key}={val}"));
    }
    for (host, container) in volumes {
        args.push("-v".to_string());
        args.push(format!("{host}:{container}"));
    }

    args.push(image.to_string());
    // Keep container alive with a long sleep
    args.push("sleep".to_string());
    args.push("infinity".to_string());

    let output = tokio::process::Command::new("docker")
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| BenchError::Docker(format!("failed to run docker run: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(BenchError::Docker(format!(
            "docker run failed for {name}: {stderr}"
        )));
    }

    let container_id = String::from_utf8_lossy(&output.stdout).trim().to_string();
    tracing::info!("Started container {} ({})", name, &container_id[..12]);
    Ok(container_id)
}

/// Execute a command inside a running container.
pub async fn exec_in_container(
    container_id: &str,
    command: &str,
    timeout: Duration,
) -> Result<ExecOutput, BenchError> {
    let result = tokio::time::timeout(
        timeout,
        tokio::process::Command::new("docker")
            .args(["exec", container_id, "bash", "-c", command])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output(),
    )
    .await;

    match result {
        Ok(Ok(output)) => Ok(ExecOutput {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code().unwrap_or(-1),
        }),
        Ok(Err(e)) => Err(BenchError::Docker(format!("docker exec failed: {e}"))),
        Err(_) => Err(BenchError::Docker(format!(
            "docker exec timed out after {timeout:?}"
        ))),
    }
}

/// Stop and remove a container.
pub async fn stop_and_remove(container_id: &str) -> Result<(), BenchError> {
    let output = tokio::process::Command::new("docker")
        .args(["rm", "-f", container_id])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| BenchError::Docker(format!("docker rm -f failed: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        tracing::warn!("docker rm -f {} failed: {}", container_id, stderr);
    }

    Ok(())
}

/// Check if a Docker image exists locally.
async fn image_exists(tag: &str) -> bool {
    tokio::process::Command::new("docker")
        .args(["image", "inspect", tag])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await
        .map(|s| s.success())
        .unwrap_or(false)
}
