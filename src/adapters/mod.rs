pub mod custom;
pub mod gaia;
pub mod pinchbench;
pub mod spot;
pub mod swe_bench;
pub mod tau_bench;
pub mod terminal_bench;
pub mod trajectory;

use crate::config::BenchConfig;
use crate::error::BenchError;
use crate::suite::BenchSuite;

const DEFAULT_JUDGE_MODEL: &str = "openrouter/anthropic/claude-haiku-4.5";

/// List of all known suite IDs.
pub const KNOWN_SUITES: &[(&str, &str)] = &[
    ("custom", "Custom JSONL tasks"),
    ("gaia", "GAIA benchmark (knowledge & reasoning)"),
    ("pinchbench", "PinchBench (skill-based agent evaluation)"),
    ("spot", "Spot checks (end-to-end user workflows)"),
    ("tau_bench", "Tau-bench (multi-turn tool use)"),
    ("swe_bench", "SWE-bench Pro (software engineering)"),
    (
        "terminal_bench",
        "Terminal Bench (containerized terminal tasks)",
    ),
    ("trajectory", "Multi-turn trajectory scenarios"),
];

/// Create a suite adapter by name.
pub fn create_suite(name: &str, config: &BenchConfig) -> Result<Box<dyn BenchSuite>, BenchError> {
    let suite_map = config.suite_config_map();
    match name {
        "custom" => {
            let dataset_path = suite_map
                .get("dataset_path")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .ok_or_else(|| {
                    BenchError::Config(
                        "suite_config.dataset_path is required for 'custom' suite".to_string(),
                    )
                })?;
            Ok(Box::new(custom::CustomSuite::new(dataset_path)))
        }
        "gaia" => {
            let dataset_path = suite_map
                .get("dataset_path")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .ok_or_else(|| {
                    BenchError::Config(
                        "suite_config.dataset_path is required for 'gaia' suite".to_string(),
                    )
                })?;
            let attachments_dir = suite_map
                .get("attachments_dir")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            Ok(Box::new(gaia::GaiaSuite::new(
                dataset_path,
                attachments_dir,
            )))
        }
        "pinchbench" => {
            let dataset_path = suite_map
                .get("dataset_path")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .ok_or_else(|| {
                    BenchError::Config(
                        "suite_config.dataset_path is required for 'pinchbench' suite".to_string(),
                    )
                })?;
            let judge_model = suite_map
                .get("judge_model")
                .and_then(|v| v.as_str())
                .unwrap_or(DEFAULT_JUDGE_MODEL)
                .to_string();
            let hybrid_auto_weight = suite_map
                .get("hybrid_auto_weight")
                .and_then(|v| v.as_float())
                .unwrap_or(0.6);
            Ok(Box::new(pinchbench::PinchBenchSuite::new(
                dataset_path,
                judge_model,
                hybrid_auto_weight,
            )))
        }
        "spot" => {
            let dataset_path = suite_map
                .get("dataset_path")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .ok_or_else(|| {
                    BenchError::Config(
                        "suite_config.dataset_path is required for 'spot' suite".to_string(),
                    )
                })?;
            Ok(Box::new(spot::SpotSuite::new(dataset_path)))
        }
        "tau_bench" => {
            let dataset_path = suite_map
                .get("dataset_path")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .ok_or_else(|| {
                    BenchError::Config(
                        "suite_config.dataset_path is required for 'tau_bench' suite".to_string(),
                    )
                })?;
            let domain = suite_map
                .get("domain")
                .and_then(|v| v.as_str())
                .unwrap_or("retail")
                .to_string();
            Ok(Box::new(tau_bench::TauBenchSuite::new(
                dataset_path,
                domain,
            )))
        }
        "swe_bench" => {
            let dataset_path = suite_map
                .get("dataset_path")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .ok_or_else(|| {
                    BenchError::Config(
                        "suite_config.dataset_path is required for 'swe_bench' suite".to_string(),
                    )
                })?;
            let workspace_dir = suite_map
                .get("workspace_dir")
                .and_then(|v| v.as_str())
                .unwrap_or("/tmp/swe-bench")
                .to_string();
            let use_docker = suite_map
                .get("use_docker")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            Ok(Box::new(swe_bench::SweBenchSuite::new(
                dataset_path,
                workspace_dir,
                use_docker,
            )))
        }
        "terminal_bench" => {
            let dataset_path = suite_map
                .get("dataset_path")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .ok_or_else(|| {
                    BenchError::Config(
                        "suite_config.dataset_path is required for 'terminal_bench' suite"
                            .to_string(),
                    )
                })?;
            let upstream_repo = suite_map
                .get("upstream_repo")
                .and_then(|v| v.as_str())
                .unwrap_or("https://github.com/harbor-framework/terminal-bench")
                .to_string();
            let upstream_tasks_subdir = suite_map
                .get("upstream_tasks_subdir")
                .and_then(|v| v.as_str())
                .unwrap_or("original-tasks")
                .to_string();
            let rebuild_images = suite_map
                .get("rebuild_images")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let verifier_timeout = suite_map
                .get("verifier_timeout")
                .and_then(|v| v.as_str())
                .and_then(|s| crate::config::parse_duration(s).ok())
                .unwrap_or(std::time::Duration::from_secs(300));
            Ok(Box::new(terminal_bench::TerminalBenchSuite::new(
                dataset_path,
                upstream_repo,
                upstream_tasks_subdir,
                rebuild_images,
                verifier_timeout,
            )))
        }
        "trajectory" => {
            let dataset_path = suite_map
                .get("dataset_path")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .ok_or_else(|| {
                    BenchError::Config(
                        "suite_config.dataset_path is required for 'trajectory' suite".to_string(),
                    )
                })?;
            let workspace_path = suite_map
                .get("workspace_path")
                .and_then(|v| v.as_str())
                .map(std::path::PathBuf::from);
            Ok(Box::new(trajectory::TrajectorySuite::new(
                dataset_path,
                workspace_path,
            )))
        }
        _ => {
            let available = KNOWN_SUITES
                .iter()
                .map(|(id, _)| *id)
                .collect::<Vec<_>>()
                .join(", ");
            Err(BenchError::SuiteNotFound {
                name: name.to_string(),
                available,
            })
        }
    }
}
