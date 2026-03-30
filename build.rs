//! Extract the resolved ironclaw git SHA from Cargo.lock at compile time.
//!
//! Sets `IRONCLAW_GIT_SHA` so `main.rs` can auto-populate `framework_version`
//! without the user having to pass `--framework-version` manually.

fn main() {
    println!("cargo::rerun-if-changed=Cargo.lock");

    let sha = ironclaw_sha_from_lockfile().unwrap_or_default();
    println!("cargo::rustc-env=IRONCLAW_GIT_SHA={sha}");
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
