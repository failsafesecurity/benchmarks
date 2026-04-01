mod adapters;
mod channel;
mod config;
mod error;
mod instrumented_llm;
#[allow(dead_code)]
mod mission;
mod openclaw;
#[allow(dead_code)]
mod post_mortem_mission;
mod results;
mod runner;
mod scoring;
mod suite;

use std::path::PathBuf;
use std::sync::Arc;

use clap::{Parser, Subcommand};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

use crate::config::BenchConfig;

#[derive(Parser)]
#[command(name = "nearai-bench", about = "NEAR AI benchmarking harness")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a benchmark suite.
    Run {
        /// Suite to run (custom, gaia, spot, tau_bench, swe_bench).
        #[arg(long)]
        suite: String,

        /// Path to bench config TOML.
        #[arg(long)]
        config: Option<PathBuf>,

        /// Override model for all matrix entries.
        #[arg(long)]
        model: Option<String>,

        /// Max tasks to run in parallel.
        #[arg(long)]
        parallelism: Option<usize>,

        /// Sample N tasks from the suite (for quick testing).
        #[arg(long)]
        sample: Option<usize>,

        /// Only run these task IDs (comma-separated).
        #[arg(long, value_delimiter = ',')]
        task_ids: Option<Vec<String>>,

        /// Only run tasks with these tags (comma-separated).
        #[arg(long, value_delimiter = ',')]
        tags: Option<Vec<String>>,

        /// Per-task timeout in seconds.
        #[arg(long)]
        timeout_secs: Option<u64>,

        /// Override results directory.
        #[arg(long)]
        results_dir: Option<PathBuf>,

        /// Agent framework being benchmarked (default: "ironclaw").
        #[arg(long, default_value = "ironclaw")]
        framework: String,

        /// Version of the agent framework (semver or git SHA).
        #[arg(long, default_value = "")]
        framework_version: String,

        /// Build against a specific ironclaw git ref (branch, tag, or commit SHA).
        ///
        /// When set, patches Cargo.toml, rebuilds the harness, and re-executes
        /// the run with the new binary. The original Cargo.toml is restored after
        /// the build. framework_version is auto-set to the resolved SHA.
        #[arg(long)]
        ironclaw_rev: Option<String>,

        /// Resume a previous run by ID.
        #[arg(long)]
        resume: Option<Uuid>,
    },

    /// Show results for a run.
    Results {
        /// Run ID or "latest".
        #[arg(default_value = "latest")]
        run_id: String,

        /// Output format.
        #[arg(long, default_value = "table")]
        format: ResultsFormat,

        /// Override results directory.
        #[arg(long)]
        results_dir: Option<PathBuf>,
    },

    /// Compare two runs.
    Compare {
        /// Baseline run ID.
        baseline: Uuid,

        /// Comparison run ID.
        comparison: Uuid,

        /// Override results directory.
        #[arg(long)]
        results_dir: Option<PathBuf>,
    },

    /// List available benchmark suites.
    List,

    /// Manage deployment monitoring missions.
    Mission {
        #[command(subcommand)]
        mission_command: MissionCommands,
    },
}

#[derive(Subcommand)]
enum MissionCommands {
    /// Create a new deployment monitoring mission (checks every 2 hours by default).
    Create {
        /// Mission ID (auto-generated if not provided).
        #[arg(long)]
        id: Option<String>,

        /// Deployment URL to monitor.
        #[arg(long)]
        url: String,

        /// Check interval in hours (default: 2).
        #[arg(long, default_value = "2")]
        interval_hours: f64,

        /// Alert on deployment failure.
        #[arg(long, default_value = "true")]
        alert_on_failure: bool,

        /// Webhook URL for alerts.
        #[arg(long)]
        webhook: Option<String>,
    },

    /// Pause an active mission.
    Pause {
        /// Mission ID to pause.
        #[arg(long)]
        id: String,
    },

    /// Resume a paused mission.
    Resume {
        /// Mission ID to resume.
        #[arg(long)]
        id: String,
    },

    /// Stop a mission permanently.
    Stop {
        /// Mission ID to stop.
        #[arg(long)]
        id: String,
    },

    /// Get status of a mission.
    Status {
        /// Mission ID to check.
        #[arg(long)]
        id: String,
    },

    /// List all missions.
    List,

    /// Start monitoring a mission (runs in foreground).
    Start {
        /// Mission ID to start.
        #[arg(long)]
        id: String,
    },
}

#[derive(Clone, Debug, clap::ValueEnum)]
enum ResultsFormat {
    Table,
    Json,
    Csv,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env from current directory before anything else.
    // This ensures our env vars take precedence over ~/.ironclaw/.env
    // (dotenvy never overwrites existing vars).
    let _ = dotenvy::dotenv();

    let cli = Cli::parse();

    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("nearai_bench=info,ironclaw=warn")),
        )
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .init();

    match cli.command {
        Commands::List => {
            println!("Available benchmark suites:\n");
            for (id, desc) in adapters::KNOWN_SUITES {
                println!("  {:<15} {}", id, desc);
            }
            println!();
        }
        Commands::Mission { mission_command } => {
            handle_mission_command(mission_command).await?;
        }
        Commands::Run {
            suite,
            config: config_path,
            model,
            parallelism,
            sample,
            task_ids,
            tags,
            timeout_secs,
            results_dir,
            framework,
            framework_version,
            ironclaw_rev,
            resume,
        } => {
            // If --ironclaw-rev is set, rebuild with that ref and re-exec.
            if let Some(ref rev) = ironclaw_rev {
                return rebuild_and_exec(rev);
            }

            // Load or create config
            let mut bench_config = if let Some(ref path) = config_path {
                BenchConfig::from_file(path)?
            } else {
                BenchConfig::minimal(model.clone())
            };

            // Apply CLI overrides
            if let Some(p) = parallelism {
                bench_config.parallelism = p;
            }
            if let Some(t) = timeout_secs {
                bench_config.task_timeout = std::time::Duration::from_secs(t);
            }
            bench_config.framework = framework;
            if !framework_version.is_empty() {
                bench_config.framework_version = framework_version;
            } else if bench_config.framework == "ironclaw" {
                // Auto-populate from the compile-time resolved git SHA.
                let sha = env!("IRONCLAW_GIT_SHA");
                if !sha.is_empty() {
                    bench_config.framework_version = sha.to_string();
                }
            }
            if let Some(ref dir) = results_dir {
                bench_config.results_dir = dir.clone();
            } else {
                bench_config.results_dir =
                    PathBuf::from(format!("./results/{}", bench_config.framework));
            }

            // If model override specified and we have matrix entries, update them
            if let Some(ref m) = model {
                for entry in &mut bench_config.matrix {
                    entry.model = Some(m.clone());
                }
            }

            // Create suite
            let bench_suite = adapters::create_suite(&suite, &bench_config)?;

            // Set up framework-specific dependencies.
            let framework = if bench_config.framework == "openclaw" {
                runner::FrameworkDeps::OpenClaw
            } else {
                // Bridge common API key env vars to ironclaw's config format.
                bridge_provider_env_vars();
                // Prevent ironclaw from reading ~/.ironclaw/ (production instance).
                isolate_from_ironclaw_home();

                let ironclaw_config = ironclaw::Config::from_env().await.map_err(|e| {
                    anyhow::anyhow!(
                        "Failed to load LLM config: {e}\n\n\
                         Set one of:\n  \
                           OPENAI_API_KEY=sk-...           (uses OpenAI)\n  \
                           ANTHROPIC_API_KEY=sk-ant-...    (uses Anthropic via OpenRouter)\n  \
                           LLM_BACKEND + LLM_BASE_URL + LLM_API_KEY  (any OpenAI-compatible provider)\n\n\
                         See .env.example for details."
                    )
                })?;

                let session =
                    ironclaw::llm::create_session_manager(ironclaw::llm::SessionConfig::default())
                        .await;

                let is_nearai = ironclaw_config.llm.backend == "nearai"
                    || ironclaw_config.llm.backend == "near_ai";
                if is_nearai && ironclaw_config.llm.nearai.api_key.is_none() {
                    session.ensure_authenticated().await?;
                }

                let llm = ironclaw::llm::create_llm_provider(&ironclaw_config.llm, session)
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to create LLM provider: {e}"))?;
                let safety = Arc::new(ironclaw_safety::SafetyLayer::new(&ironclaw_config.safety));
                runner::FrameworkDeps::Ironclaw { llm, safety }
            };

            let runner = runner::BenchRunner::new(bench_suite, bench_config.clone(), framework);

            // Run for each matrix entry
            for matrix_entry in &bench_config.matrix {
                let run_id = runner
                    .run(
                        matrix_entry,
                        sample,
                        task_ids.as_deref(),
                        tags.as_deref(),
                        resume,
                    )
                    .await?;
                println!("Run complete: {}", run_id);
            }
        }
        Commands::Results {
            run_id,
            format,
            results_dir,
        } => {
            let base = results_dir.unwrap_or_else(|| find_results_base("./results"));
            let uuid = if run_id == "latest" {
                results::find_latest_run(&base)?
                    .ok_or_else(|| anyhow::anyhow!("No runs found in {}", base.display()))?
            } else {
                Uuid::parse_str(&run_id)?
            };

            let json_path = results::run_json_path(&base, uuid);
            let jsonl_path = results::tasks_jsonl_path(&base, uuid);

            let run = results::read_run_result(&json_path)?;
            let tasks = results::read_task_results(&jsonl_path)?;

            match format {
                ResultsFormat::Table => {
                    results::print_results_table(&tasks, &run);
                }
                ResultsFormat::Json => {
                    let output = serde_json::json!({
                        "run": run,
                        "tasks": tasks,
                    });
                    println!("{}", serde_json::to_string_pretty(&output)?);
                }
                ResultsFormat::Csv => {
                    println!("task_id,score,label,tokens,cost,turns,time_s");
                    for task in &tasks {
                        println!(
                            "{},{:.3},{},{},{:.4},{},{:.1}",
                            task.task_id,
                            task.score.value,
                            task.score.label,
                            task.trace.input_tokens + task.trace.output_tokens,
                            task.trace.estimated_cost_usd,
                            task.trace.turns,
                            task.trace.wall_time_ms as f64 / 1000.0,
                        );
                    }
                }
            }
        }
        Commands::Compare {
            baseline,
            comparison,
            results_dir,
        } => {
            let base = results_dir.unwrap_or_else(|| find_results_base("./results"));

            let baseline_run = results::read_run_result(&results::run_json_path(&base, baseline))?;
            let comparison_run =
                results::read_run_result(&results::run_json_path(&base, comparison))?;

            println!("\nComparison: {} vs {}\n", baseline, comparison);
            println!(
                "{:<20} {:>12} {:>12} {:>10}",
                "Metric", "Baseline", "Comparison", "Delta"
            );
            println!("{}", "-".repeat(58));

            let pass_delta = comparison_run.pass_rate - baseline_run.pass_rate;
            println!(
                "{:<20} {:>11.1}% {:>11.1}% {:>+9.1}%",
                "Pass rate",
                baseline_run.pass_rate * 100.0,
                comparison_run.pass_rate * 100.0,
                pass_delta * 100.0,
            );

            let score_delta = comparison_run.avg_score - baseline_run.avg_score;
            println!(
                "{:<20} {:>12.3} {:>12.3} {:>+10.3}",
                "Avg score", baseline_run.avg_score, comparison_run.avg_score, score_delta,
            );

            let cost_delta = comparison_run.total_cost_usd - baseline_run.total_cost_usd;
            println!(
                "{:<20} {:>11.4}$ {:>11.4}$ {:>+9.4}$",
                "Total cost",
                baseline_run.total_cost_usd,
                comparison_run.total_cost_usd,
                cost_delta,
            );

            let time_b = baseline_run.total_wall_time_ms as f64 / 1000.0;
            let time_c = comparison_run.total_wall_time_ms as f64 / 1000.0;
            println!(
                "{:<20} {:>11.1}s {:>11.1}s {:>+9.1}s",
                "Total time",
                time_b,
                time_c,
                time_c - time_b,
            );

            println!(
                "{:<20} {:>12} {:>12}",
                "Model", baseline_run.model, comparison_run.model,
            );

            // Show framework versions if they differ (useful for A/B testing)
            let base_ver = short_version(&baseline_run.framework_version);
            let comp_ver = short_version(&comparison_run.framework_version);
            if !base_ver.is_empty() || !comp_ver.is_empty() {
                println!("{:<20} {:>12} {:>12}", "Framework ver", base_ver, comp_ver,);
            }

            // Per-category comparison
            let all_cats: std::collections::BTreeSet<String> = baseline_run
                .categories
                .keys()
                .chain(comparison_run.categories.keys())
                .cloned()
                .collect();

            if !all_cats.is_empty() {
                println!();
                println!(
                    "{:<25} {:>8} {:>8} {:>8}",
                    "Category", "Base%", "Comp%", "Delta"
                );
                println!("{}", "-".repeat(52));
                for cat in &all_cats {
                    let b = baseline_run.categories.get(cat);
                    let c = comparison_run.categories.get(cat);
                    let bp = b.map_or(0.0, |r| r.pass_rate * 100.0);
                    let cp = c.map_or(0.0, |r| r.pass_rate * 100.0);
                    let cat_display = if cat.len() > 23 {
                        let truncated: String = cat.chars().take(20).collect();
                        format!("{truncated}...")
                    } else {
                        cat.clone()
                    };
                    println!(
                        "{:<25} {:>7.1}% {:>7.1}% {:>+7.1}%",
                        cat_display,
                        bp,
                        cp,
                        cp - bp,
                    );
                }
            }
            println!();
        }
    }

    Ok(())
}

/// Patch Cargo.toml to use a specific ironclaw git ref, rebuild, and re-exec.
///
/// 1. Reads the current Cargo.toml and saves a backup.
/// 2. Replaces the ironclaw git dependency line with `rev = "<ref>"`.
/// 3. Runs `cargo build --release`.
/// 4. Restores the original Cargo.toml.
/// 5. Re-executes the freshly built binary with the same args, minus `--ironclaw-rev`.
fn rebuild_and_exec(rev: &str) -> anyhow::Result<()> {
    let cargo_toml = PathBuf::from("Cargo.toml");
    let original = std::fs::read_to_string(&cargo_toml)?;

    // Replace both ironclaw dependency lines (ironclaw and ironclaw_safety)
    let ironclaw_re = regex::Regex::new(r#"(?m)^(ironclaw(?:_safety)?)\s*=\s*\{[^}]+\}\s*$"#)
        .expect("valid regex");

    // Build replacement that preserves the crate name
    let patched = ironclaw_re
        .replace_all(&original, |caps: &regex::Captures| {
            let crate_name = &caps[1];
            if rev.len() >= 7 && rev.chars().all(|c| c.is_ascii_hexdigit()) {
                format!(
                    r#"{crate_name} = {{ git = "https://github.com/nearai/ironclaw.git", rev = "{rev}" }}"#
                )
            } else if rev.starts_with("v") && rev[1..].contains('.') {
                format!(
                    r#"{crate_name} = {{ git = "https://github.com/nearai/ironclaw.git", tag = "{rev}" }}"#
                )
            } else {
                format!(
                    r#"{crate_name} = {{ git = "https://github.com/nearai/ironclaw.git", branch = "{rev}" }}"#
                )
            }
        })
        .to_string();

    if patched == original {
        anyhow::bail!(
            "Could not find ironclaw dependency line in Cargo.toml to patch.\n\
             Expected a line like: ironclaw = {{ git = \"...\", ... }}"
        );
    }

    eprintln!("Patching Cargo.toml to use ironclaw @ {rev}");
    std::fs::write(&cargo_toml, &patched)?;

    // Build (debug profile to reduce memory pressure; release builds OOM on constrained machines)
    eprintln!("Building with ironclaw @ {rev} ...");
    let build_status = std::process::Command::new("cargo").args(["build"]).status();

    // Always restore the original Cargo.toml, even if build fails
    std::fs::write(&cargo_toml, &original)?;
    eprintln!("Restored original Cargo.toml");

    let status = build_status?;
    if !status.success() {
        anyhow::bail!("cargo build failed for ironclaw @ {rev}");
    }

    // Re-exec with the freshly built binary, removing --ironclaw-rev from args
    let args: Vec<String> = std::env::args().collect();
    let mut new_args: Vec<&str> = Vec::new();
    let mut skip_next = false;
    for (i, arg) in args.iter().enumerate() {
        if skip_next {
            skip_next = false;
            continue;
        }
        if arg == "--ironclaw-rev" {
            // Skip this flag and its value
            skip_next = true;
            continue;
        }
        if arg.starts_with("--ironclaw-rev=") {
            continue;
        }
        if i == 0 {
            continue; // Skip the binary name, we'll use the release binary
        }
        new_args.push(arg);
    }

    let binary = PathBuf::from("target/debug/nearai-bench");
    eprintln!("Re-executing: {} {}", binary.display(), new_args.join(" "));

    let status = std::process::Command::new(&binary)
        .args(&new_args)
        .status()?;

    std::process::exit(status.code().unwrap_or(1));
}

/// Handle mission management commands
async fn handle_mission_command(cmd: MissionCommands) -> anyhow::Result<()> {
    use mission::{DeploymentMission, MissionConfig};

    match cmd {
        MissionCommands::Create {
            id,
            url,
            interval_hours,
            alert_on_failure,
            webhook,
        } => {
            let mission_id =
                id.unwrap_or_else(|| format!("deploy-{}", chrono::Utc::now().timestamp()));

            let config = MissionConfig {
                mission_id: mission_id.clone(),
                deployment_url: url,
                check_interval_hours: interval_hours,
                alert_on_failure,
                webhook_url: webhook,
            };

            let _mission = mission::create_mission_from_config(&config);

            println!("Created mission: {}", mission_id);
            println!("  URL: {}", config.deployment_url);
            println!("  Interval: {} hours", config.check_interval_hours);
            println!("  Alert on failure: {}", config.alert_on_failure);
            if let Some(webhook) = config.webhook_url {
                println!("  Webhook: {}", webhook);
            }
            println!(
                "\nTo start monitoring: nearai-bench mission start --id {}",
                mission_id
            );
            println!("To pause: nearai-bench mission pause --id {}", mission_id);
        }

        MissionCommands::Pause { id } => {
            // In a real implementation, this would look up the mission from persistent storage
            let mission = DeploymentMission::new(&id, "placeholder", 2.0);
            mission.pause();
            println!("Mission {} paused", id);
        }

        MissionCommands::Resume { id } => {
            let mission = DeploymentMission::new(&id, "placeholder", 2.0);
            mission.resume();
            println!("Mission {} resumed", id);
        }

        MissionCommands::Stop { id } => {
            let mission = DeploymentMission::new(&id, "placeholder", 2.0);
            mission.stop();
            println!("Mission {} stopped", id);
        }

        MissionCommands::Status { id } => {
            let mission = DeploymentMission::new(&id, "placeholder", 2.0);
            let state = mission.state();
            println!("Mission: {}", id);
            println!("State: {:?}", state);
            println!("Active: {}", mission.is_active());
        }

        MissionCommands::List => {
            println!("Active missions:");
            println!("  (Mission persistence not implemented in this demo)");
            println!("\nExample usage:");
            println!(
                "  Create: nearai-bench mission create --url https://deploy.example.com --interval-hours 2"
            );
            println!("  Start:  nearai-bench mission start --id <mission-id>");
            println!("  Pause:  nearai-bench mission pause --id <mission-id>");
            println!("  Resume: nearai-bench mission resume --id <mission-id>");
        }

        MissionCommands::Start { id } => {
            println!("Starting mission {} (runs until Ctrl+C)...", id);
            println!("Monitoring deployment every 2 hours");
            println!("\nTo pause: nearai-bench mission pause --id {}", id);
            println!("To resume: nearai-bench mission resume --id {}", id);

            let mission = DeploymentMission::new(&id, "https://deploy.example.com", 2.0);

            // Run in background so we can demonstrate pause/resume
            let mission_clone = mission.clone();
            let handle = tokio::spawn(async move {
                if let Err(e) = mission_clone.run().await {
                    eprintln!("Mission error: {}", e);
                }
            });

            // Wait for Ctrl+C
            tokio::signal::ctrl_c().await?;
            println!("\nReceived shutdown signal, stopping mission...");
            mission.stop();
            handle.abort();
        }
    }

    Ok(())
}

/// Truncate a git SHA or version string to a short display form.
fn short_version(v: &str) -> String {
    if v.is_empty() {
        return String::new();
    }
    // If it looks like a full 40-char hex SHA, truncate to 10
    if v.len() >= 40 && v.chars().all(|c| c.is_ascii_hexdigit()) {
        return v[..10].to_string();
    }
    // Already short or a semver — keep as-is
    v.to_string()
}

/// Find the results base directory containing the most recent run.
///
/// Scans all subdirectories of `root` (e.g. `./results/ironclaw/`,
/// `./results/openclaw/`) and returns the one with the latest run.
/// Falls back to `{root}/ironclaw` if nothing is found.
fn find_results_base(root: &str) -> PathBuf {
    let root_path = PathBuf::from(root);
    if !root_path.is_dir() {
        return root_path.join("ironclaw");
    }
    let mut best: Option<(PathBuf, std::time::SystemTime)> = None;
    if let Ok(entries) = std::fs::read_dir(&root_path) {
        for entry in entries.flatten() {
            let fw_dir = entry.path();
            if !fw_dir.is_dir() {
                continue;
            }
            if let Some(uuid) = results::find_latest_run(&fw_dir).ok().flatten() {
                let run_json = results::run_json_path(&fw_dir, uuid);
                if let Ok(meta) = std::fs::metadata(&run_json) {
                    if let Ok(modified) = meta.modified() {
                        if best.as_ref().is_none_or(|(_, t)| modified > *t) {
                            best = Some((fw_dir, modified));
                        }
                    }
                }
            }
        }
    }
    best.map(|(p, _)| p)
        .unwrap_or_else(|| root_path.join("ironclaw"))
}

/// Redirect ironclaw away from `~/.ironclaw/` (production instance).
///
/// ironclaw resolves all state paths via `dirs::home_dir().join(".ironclaw")`.
/// We point `HOME` at a temporary directory so the entire `~/.ironclaw/`
/// tree — `.env`, `settings.json`, `session.json`, `ironclaw.db` — resolves
/// to an empty, isolated location instead of the user's production install.
///
/// We also set the minimal env vars needed for `Config::from_env()` to
/// succeed without running `ironclaw onboard`.
fn isolate_from_ironclaw_home() {
    let bench_home = std::env::temp_dir().join("nearai-bench-home");
    std::fs::create_dir_all(bench_home.join(".ironclaw")).ok();

    // SAFETY: called before any threads are spawned (single-threaded main init).
    unsafe {
        std::env::set_var("HOME", &bench_home);

        // Use libsql so DatabaseConfig doesn't require DATABASE_URL.
        if std::env::var("DATABASE_BACKEND").is_err() {
            std::env::set_var("DATABASE_BACKEND", "libsql");
        }
    }
    tracing::debug!("Redirected ironclaw home to {}", bench_home.display());
}

/// Bridge common provider env vars to ironclaw's config format.
///
/// If the user has set `OPENAI_API_KEY` or `ANTHROPIC_API_KEY` but has NOT
/// already set `LLM_BACKEND`, we auto-configure ironclaw's env vars so
/// `Config::from_env()` picks them up without needing the onboarding wizard.
fn bridge_provider_env_vars() {
    // Don't override if the user already configured ironclaw directly.
    if std::env::var("LLM_BACKEND").is_ok() {
        return;
    }

    // SAFETY: called before any threads are spawned (single-threaded main init).
    unsafe {
        if let Ok(key) = std::env::var("OPENAI_API_KEY") {
            std::env::set_var("LLM_BACKEND", "openai_compatible");
            std::env::set_var("LLM_BASE_URL", "https://api.openai.com/v1");
            std::env::set_var("LLM_API_KEY", &key);
            if std::env::var("LLM_MODEL").is_err() {
                std::env::set_var("LLM_MODEL", "gpt-4o");
            }
            tracing::info!("Using OpenAI provider (from OPENAI_API_KEY)");
        } else if let Ok(key) = std::env::var("ANTHROPIC_API_KEY") {
            std::env::set_var("LLM_BACKEND", "anthropic");
            std::env::set_var("LLM_API_KEY", &key);
            if std::env::var("LLM_MODEL").is_err() {
                std::env::set_var("LLM_MODEL", "claude-sonnet-4-20250514");
            }
            tracing::info!("Using Anthropic provider (from ANTHROPIC_API_KEY)");
        }
    }
}
