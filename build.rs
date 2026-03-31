//! Extract the resolved ironclaw git SHA from Cargo.lock at compile time
//! and detect API-level differences between ironclaw branches.
//!
//! Sets `IRONCLAW_GIT_SHA` so `main.rs` can auto-populate `framework_version`
//! without the user having to pass `--framework-version` manually.
//!
//! Also probes for optional fields/types that differ between ironclaw branches
//! (e.g. `engine_v2` in v2-architecture) and sets cfg flags accordingly.

fn main() {
    println!("cargo::rerun-if-changed=Cargo.lock");

    let sha = ironclaw_sha_from_lockfile().unwrap_or_default();
    println!("cargo::rustc-env=IRONCLAW_GIT_SHA={sha}");

    // Detect ironclaw API surface by checking if AgentConfig has engine_v2.
    // We probe the checkout source directly via the Cargo git cache.
    if ironclaw_has_engine_v2(&sha) {
        println!("cargo::rustc-cfg=ironclaw_engine_v2");
    }
}

/// Parse Cargo.lock to find ironclaw's resolved git commit SHA.
///
/// The source line looks like:
///   `source = "git+https://...#<full-sha>"`
fn ironclaw_sha_from_lockfile() -> Option<String> {
    let lock = std::fs::read_to_string("Cargo.lock").ok()?;
    let mut in_ironclaw = false;

    for line in lock.lines() {
        let trimmed = line.trim();
        if trimmed == r#"name = "ironclaw""# {
            in_ironclaw = true;
            continue;
        }
        if in_ironclaw {
            if trimmed.starts_with("source = ") {
                // source = "git+https://github.com/nearai/ironclaw.git?branch=staging#8acdd080..."
                if let Some(hash_pos) = trimmed.rfind('#') {
                    let sha = trimmed[hash_pos + 1..].trim_end_matches('"');
                    return Some(sha.to_string());
                }
                return None;
            }
            // If we hit an empty line or next package block, stop
            if trimmed.is_empty() || trimmed == "[[package]]" {
                return None;
            }
        }
    }
    None
}

/// Check if the resolved ironclaw checkout has the `engine_v2` field in AgentConfig.
///
/// Scans `~/.cargo/git/checkouts/ironclaw-*/` for the matching SHA prefix.
fn ironclaw_has_engine_v2(sha: &str) -> bool {
    if sha.len() < 7 {
        return false;
    }
    let short = &sha[..7];
    let cargo_home = std::env::var("CARGO_HOME")
        .unwrap_or_else(|_| format!("{}/.cargo", std::env::var("HOME").unwrap_or_default()));
    let checkouts = std::path::Path::new(&cargo_home).join("git/checkouts");

    // Find ironclaw checkout directories
    let Ok(entries) = std::fs::read_dir(&checkouts) else {
        return false;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.starts_with("ironclaw-") {
            continue;
        }
        // Look for the SHA-prefixed subdirectory
        let checkout_dir = entry.path().join(short);
        let agent_config = checkout_dir.join("src/config/agent.rs");
        if agent_config.exists() {
            if let Ok(content) = std::fs::read_to_string(&agent_config) {
                return content.contains("engine_v2");
            }
        }
        // Also check non-modular config.rs layout
        let config_rs = checkout_dir.join("src/config.rs");
        if config_rs.exists() {
            if let Ok(content) = std::fs::read_to_string(&config_rs) {
                return content.contains("engine_v2");
            }
        }
    }
    false
}
